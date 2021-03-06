extern crate shio;

use shio::prelude::*;

// redirect the current
fn redirect_to(_: Context) -> Response {
    Response::build()
        .status(StatusCode::SeeOther)
        .header(shio::header::Location::new("/redirected"))
        .into()
}

fn redirected(_: Context) -> &'static str {
    "You has been redirected!\n"
}

fn index(_: Context) -> &'static str {
    "Hello World!\n"
}

fn main() {
    Shio::default()
        .route((Method::Get, "/", index))
        .route((Method::Get, "/redirect", redirect_to))
        .route((Method::Get, "/redirected", redirected))
        .run(":7878")
        .unwrap();
}
