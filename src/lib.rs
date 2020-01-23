#![recursion_limit = "512"]

use wasm_bindgen::prelude::*;
use std::panic;

extern crate console_error_panic_hook;
extern crate web_logger;
extern crate pest;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate stdweb;
#[macro_use] extern crate pest_derive;

pub mod coordinate;
pub mod grammar;
pub mod model;
pub mod session;
pub mod style;
pub mod util;
pub mod view;

use crate::model::Model;

/*
 * DATA MODEL:
 * is centered around the "grammars" map: HashMap<Coordinate, Grammar>
 * this is a linear-time accessible directory of every grammar in the system
 * as indexed by the grammar coordinate
 *
 */

/*
 * # Other Notes:
 *
 * Enums vs Structs: 
 * Structs are just a basic collection of fields like in a class.
 * Enums are used to represent a value that can take multiple forms.
 * For instance, 
 *
 * `#[derive()]`:
 * These is a macro provided in the Rust standard library for generating code 
 * to automatically implement certain traits (interfaces) in Rust
 *
 * NonZeroU32:
 * In a number of places in the application, we make use of integers that can be neither
 * negative (unsigned) nor zero, such as the coordinate values. We adapt the standard rust 
 * data type NonZeroU32 (non-zero unsigned 32-bit integer) as a type for such values
 */

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
//    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    yew::start_app::<Model>();
    Ok(())
}
