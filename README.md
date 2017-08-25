# Shio
![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Crates.io](https://img.shields.io/crates/v/shio.svg)](https://crates.io/crates/shio)
[![Crates.io](https://img.shields.io/crates/d/shio.svg)](https://crates.io/crates/shio)
[![Docs.rs](https://docs.rs/shio/badge.svg)](https://docs.rs/shio)
[![IRC](https://img.shields.io/badge/chat-%23shio-yellow.svg)](https://kiwiirc.com/client/irc.mozilla.org/#shio)
> Shio is a fast, simple, and asynchronous micro web-framework for Rust.

 - **Asynchronous**. Handlers are both handled _asynchronously_ and may be _asynchronous_ themselves. A `shio::Handler` receives a `tokio_core::reactor::Handle` which may be used to schedule additional work on the thread-local event loop.

 - **Multithreaded**. By default, requests are handled by multiple threads, each running an event loop powered by `tokio`.

### WARNING: Shio is at 0.0.x which means the API is highly unstable. Use at your own risk. See [#1](https://github.com/mehcode/shio-rs/issues/1) to discuss our general direction.

## Usage

```toml
[dependencies]
shio = "0.0.6"
```

```rust
extern crate shio;

use shio::prelude::*;

fn hello_world(_: Context) -> &'static str {
    "Hello World\n"
}

fn main() {
    Shio::default()
        .route((Method::Get, "/", hello_world))
        .run(":7878").unwrap();
}
```

## Examples

### [Proxy Request](examples/proxy/src/main.rs)

Handlers may return a value that implements `Responder` or a `BoxFuture<T, E>`
where `T` implements `Responder`.

For some concrete examples, you may return
a `String`, a `BoxFuture<StatusCode, _>`,
a `&'static str`, or a `BoxFuture<Response, _>`. `Responder` is implemented
for many other primitive types and is meant to serve as an integration point
for external crates like [Askama](https://github.com/djc/askama).

```rust
extern crate hyper;
extern crate shio;

use shio::prelude::*;
use hyper::Client;

fn proxy(ctx: Context) -> BoxFuture<Response, hyper::Error> {
    // Additional work can be scheduled on the thread-local event loop,
    // as each handler receives a reference to it
    Client::new(&ctx)
        .get("http://www.google.com".parse().unwrap())
        // Map the _streaming_ response from google into a _streaming_
        // response from us
        .map(|res| Response::build().body(res.body()))
        // Use `.into_box` to turn this future stream into a `BoxFuture`
        // that can be easily returned on stable Rust.
        //
        // When `impl Trait` becomes available on stable Rust, this
        // necessity will go away
        .into_box()
}

// fn main omitted [...]
```

### [Stateful](examples/state/src/main.rs)

Handlers are **not** cloned on each request and therefore may contain state.
Note that any fields must be `Send + Sync`.

```rust
extern crate shio;

use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use shio::prelude::*;

#[derive(Default)]
struct HandlerWithState {
    counter: AtomicUsize,
}

impl shio::Handler for HandlerWithState {
    type Result = String;

    fn call(&self, _: Context) -> Self::Result {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);

        format!(
            "Hi, #{} (from thread: {:?})\n",
            counter,
            thread::current().id()
        )
    }
}
```

### Even More Examples

Many more usage [examples/](https://github.com/mehcode/shio-rs/tree/master/examples) are included with Shio.

Examples may be ran with `cargo run -p <example name>`. For instance, to run the `hello` example, use:

```bash
$ cargo run -p hello
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
