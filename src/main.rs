extern crate cargo_web;
extern crate simple_server;

use simple_server::Server;
use cargo_web::{CargoWebOpts, StartOpts};
use structopt::StructOpt;
use std::thread;

fn main() {
    let frontend_builder = thread::Builder::new().name("frontend_builder".to_string()).spawn(|| {
        let _res = cargo_web::run(CargoWebOpts::Start(
            StartOpts::from_iter_safe(&[
                "--target=wasm32-unknown-unknown",
                "--package=frontend",
                "--port=9000",
            ]).expect("expected hardcoded cargo-web args to be valid"),
        ));
    }).unwrap();

    let api_proxy = thread::Builder::new().name("api_proxy".to_string()).spawn(|| {
        let host = "127.0.0.1";
        let port = "7878";

        let server = Server::new(|request, mut response| {
            Ok(response.body("Hello Rust!".as_bytes().to_vec())?)
        });

        println!("starting api_proxy at http://127.0.0.1:7878");
        server.listen(host, port);
    }).unwrap();

    frontend_builder.join().unwrap();
    api_proxy.join().unwrap();
}
