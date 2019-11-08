extern crate cargo_web;
extern crate env_logger;
#[macro_use]
extern crate log;

extern crate simple_server;

use simple_server::Server;
use cargo_web::{CargoWebOpts, StartOpts};
use structopt::StructOpt;
use failure::{bail};

fn main() {
    let res = cargo_web::run(CargoWebOpts::Start(
        StartOpts::from_iter_safe(&[
            "--target=wasm32-unknown-unknown",
            "--package=frontend",
        ]).expect("expected hardcoded cargo-web args to be valid"),
    ));

    env_logger::init().unwrap();

    let host = "127.0.0.1";
    let port = "7878";

    let server = Server::new(|request, mut response| {
        info!("Request received. {} {}", request.method(), request.uri());
        Ok(response.body("Hello Rust!".as_bytes().to_vec())?)
    });

    server.listen(host, port);
}
