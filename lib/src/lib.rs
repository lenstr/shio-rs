#![cfg_attr(feature = "cargo-clippy", warn(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(missing_docs_in_private_items))]

extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate net2;
extern crate num_cpus;
extern crate regex;
extern crate tokio_core;
extern crate tokio_io;
extern crate typemap;

pub mod context;
mod handler;
mod shio;
mod service;
pub mod ext;
pub mod response;
pub mod request;
pub mod errors;
pub mod router;

pub use hyper::{header, Method, StatusCode};

pub use response::Response;
pub use request::Request;
pub use shio::Shio;
pub use context::Context;
pub use handler::Handler;

/// Re-exports important traits and types. Meant to be glob imported when using Shio.
pub mod prelude {
    pub use super::{header, Context, Method, Request, Response, Shio, StatusCode};
    pub use super::router::Parameters;

    pub use futures::{Future, IntoFuture};
    pub use ext::{BoxFuture, FutureExt};
}
