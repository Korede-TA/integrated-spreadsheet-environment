mod frontend;
#[macro_use] extern crate maplit;
extern crate cargo_web;

use cargo_web::{run, CargoWebOpts, StartOpts, Target};
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    frontend::run();

    cargo_web::run(CargoWebOpts::Start(StartOpts {
        build_args: {
            package: None,
            features: None,
            all_features: false,
            no_default_features: false,
            use_system_emscripten: false,
            release: false,
            target: Backend::WebAssembly,
            verbose: false,
        },
        build_target: {
            lib: false,
            bin: None,
            example: None,
            test: None,
            bench: None,
        },
        auto_reload: true,
        open: true,
        host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 8000,
    }));

    // if let Err(error) = run(CargoWebOpts::from_iter(argv)) {
    //     eprintln!("error: {}", error);
    //     exit(101);
    // }
}
