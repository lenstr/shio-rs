use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::fmt;
use std::net::SocketAddr;

use num_cpus;
use futures::{future, IntoFuture, Stream};
use hyper::server::Http;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use net2::TcpBuilder;

#[cfg(unix)]
use net2::unix::UnixTcpBuilderExt;

use handler::Handler;
use router::{Route, Router};
use errors::ListenError;
use response::Response;
use ext::{ToSocketAddrsExt};
use service::Service;

pub struct Shio<H: Handler + 'static>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    handler: Arc<H>,
    threads: usize,
}

impl<H: Handler> Shio<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    pub fn new(handler: H) -> Self {
        Shio {
            handler: Arc::new(handler),
            threads: num_cpus::get(),
        }
    }

    /// Set the number of threads to use.
    pub fn threads(&mut self, threads: usize) {
        self.threads = threads;
    }

    #[cfg_attr(feature = "cargo-clippy", allow(use_debug, never_loop))]
    pub fn run<A: ToSocketAddrsExt>(&self, addr: A) -> Result<(), ListenError> {
        let addrs = addr.to_socket_addrs_ext()?.collect::<Vec<_>>();
        let mut children = Vec::new();

        let spawn = || -> JoinHandle<Result<(), ListenError>> {
            let addrs = addrs.clone();
            let handler = self.handler.clone();

            thread::spawn(move || -> Result<(), ListenError> {
                let mut core = Core::new()?;
                let mut work = Vec::new();
                let handle = core.handle();
                let service = Service::new(handler, handle.clone());

                for addr in &addrs {
                    let handle = handle.clone();
                    let builder = (match *addr {
                        SocketAddr::V4(_) => TcpBuilder::new_v4(),
                        SocketAddr::V6(_) => TcpBuilder::new_v6(),
                    })?;

                    // Set SO_REUSEADDR on the socket
                    builder.reuse_address(true)?;

                    // Set SO_REUSEPORT on the socket (in unix)
                    #[cfg(unix)]
                    builder.reuse_port(true)?;

                    builder.bind(&addr)?;

                    let listener = TcpListener::from_listener(
                        // TODO: Should this be configurable somewhere?
                        builder.listen(128)?,
                        addr,
                        &handle,
                    )?;

                    let protocol = Http::new();
                    let service = service.clone();

                    let srv = listener.incoming().for_each(move |(socket, addr)| {
                        protocol.bind_connection(&handle, socket, addr, service.clone());

                        Ok(())
                    });

                    work.push(srv);
                }

                core.run(future::join_all(work))?;

                Ok(())
            })
        };

        for _ in 0..self.threads {
            children.push(spawn());
        }

        while !children.is_empty() {
            let respawn = 'outer: loop {
                for child in children.drain(..) {
                    if child.join().is_err() {
                        // Thread panicked; spawn another one
                        // TODO: Should there be any sort of limit/backoff here?
                        break 'outer true;
                    }
                }

                break false;
            };

            if respawn {
                children.push(spawn());
            }
        }

        Ok(())
    }
}

impl Default for Shio<Router> {
    fn default() -> Self {
        Shio::new(Router::new())
    }
}

impl Shio<Router> {
    pub fn route<R: Into<Route>>(&mut self, route: R) -> &mut Self {
        Arc::get_mut(&mut self.handler).map(|router| router.add(route));

        self
    }
}
