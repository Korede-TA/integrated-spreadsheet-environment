extern crate cargo_web;

use cargo_web::{CargoWebOpts, StartOpts};
use structopt::StructOpt;
use failure::{bail};

fn main() {
    let res = cargo_web::run(CargoWebOpts::Start(
        StartOpts::from_iter_safe(&["--target=wasm32-unknown-unknown"])
            .expect("expected hardcoded cargo-web args to be valid"),
    ));

    /* run(CargoWebOpts::Start(StartOpts {
        build_args: Build {
            package: None,
            features: None,
            all_features: false,
            no_default_features: false,
            use_system_emscripten: false,
            release: false,
            target: Backend::WebAssembly,
            verbose: false,
        },
        build_target: Target {
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
    })); */
}
