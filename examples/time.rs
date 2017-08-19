#![feature(conservative_impl_trait)]

extern crate salt;

use std::time;

use salt::{Stack, BoxHandler, Handler};
use salt::prelude::*;

fn hello(_: Context) -> Response {
    Response::new().body("Hello World!\n")
}

// Measure request time and print it out
fn timeit(next: BoxHandler) -> impl Handler {
    move |ctx: Context| {
        // Mark the time of the request
        let time_before = time::Instant::now();

        // Continue processing the request
        next.call(ctx).map(move |response| {
            // Find the elapsed time in μs
            let d = time::Instant::now().duration_since(time_before);
            let elapsed = (d.as_secs() * 1_000_000) + (d.subsec_nanos() as u64 / 1_000);
            println!("Request took {}μs", elapsed);

            // Forward the response
            response
        })
    }
}

fn main() {
    // Create a new _middleware_ stack, using `hello` as the _root_ handler
    let mut stack = Stack::new(hello);

    // Add `timeit` to the sequence of middleware to be executed
    stack.add(timeit);

    // Create and run a Salt service, using our configured stack as _its_ handler
    Salt::new(stack).run("0.0.0.0:7878");
}