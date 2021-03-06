use std::sync::Arc;
use std::fmt;
use std::panic::AssertUnwindSafe;

use hyper;
use tokio_core::reactor::Handle;
use futures::{future, Future, IntoFuture};

use request::Request;
use response::Response;
use handler::{default_catch, Handler};
use context::Context;
use ext::{BoxFuture};

// FIXME: Why does #[derive(Clone)] not work here? This _seems_ like a implementation that
//        should be auto-derived.

// #[derive(Clone)]
pub(crate) struct Service<H: Handler + 'static>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    handler: Arc<H>,
    handle: Handle,
}

impl<H: Handler + 'static> Service<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    pub(crate) fn new(handler: Arc<H>, handle: Handle) -> Self {
        Service { handler, handle }
    }
}

impl<H: Handler + 'static> Clone for Service<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    fn clone(&self) -> Self {
        Service {
            handler: self.handler.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl<H: Handler + 'static> hyper::server::Service for Service<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let request = Request::new(request.deconstruct());
        let ctx = Context::new(self.handle.clone(), request);
        let handler = self.handler.clone();

        Box::new(
            AssertUnwindSafe(future::lazy(move || handler.call(ctx).into_future()))
                .catch_unwind()
                .then(|result| -> BoxFuture<hyper::Response, hyper::Error> {
                    Box::new(future::ok(
                        (match result {
                            Err(err) => default_catch(err),
                            Ok(Err(err)) => default_catch(err),
                            Ok(Ok(response)) => response,
                        }).into_hyper_response(),
                    ))
                }),
        )
    }
}
