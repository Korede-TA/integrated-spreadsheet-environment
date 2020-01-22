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
#[macro_use] pub mod utils;

use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Kind};
use crate::model::Model;
use crate::style::Style;

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


// macro for easily defining a vector of non-zero tuples
// used in Coordinate::root() below
#[macro_export]
macro_rules! row_col_vec {
    ( $( $x:expr ), * ) => {
        {
            let mut v: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();
            $(
                v.push(non_zero_u32_tuple($x));
            )*
            v
        }
    };
}

// macro for easily defining a coordinate
// either absolutely or relative to it's parent coordinate
// TODO: this code is messy, can be optimized more later 
#[macro_export]
macro_rules! coord {
    ( $coord_str:tt ) => {
        {
            

            let mut fragments: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();

            let pairs = CoordinateParser::parse(Rule::coordinate, $coord_str).unwrap_or_else(|e| panic!("{}", e));

            for pair in pairs {
                match pair.as_rule() {
                    Rule::special if pair.as_str() == "root" => {
                        fragments.push(non_zero_u32_tuple((1, 1)));
                    }
                    Rule::special if pair.as_str() == "meta" => {
                        fragments.push(non_zero_u32_tuple((1, 2)));
                    }
                    Rule::fragment => {
                        let mut fragment: (u32, u32) = (0,0);
                        for inner_pair in pair.into_inner() {
                            match inner_pair.as_rule() {
                                // COLUMN
                                Rule::alpha => {
                                    let mut val: u32 = 0;
                                    for ch in inner_pair.as_str().to_string().chars() {
                                        val += (ch as u32) - 64;
                                    }
                                    fragment.1 = val;
                                }
                                // ROW
                                Rule::digit => {
                                    fragment.0 = inner_pair.as_str().parse::<u32>().unwrap();
                                }
                                _ => unreachable!()
                            };
                        }
                        fragments.push(non_zero_u32_tuple(fragment));
                    }
                    _ => unreachable!()
                }
            }

            Coordinate {
                row_cols: fragments,
            }
        }
    };

}

#[macro_export]
macro_rules! coord_col {
    ( $parent_str:tt, $col_str:tt ) => {
        {
            let mut col: u32 = 0;
            for ch in $col_str.to_string().chars() {
                col += (ch as u32) - 64;
            }

            Col(coord!($parent_str), NonZeroU32::new(col).unwrap())
        }
    };
}

#[macro_export]
macro_rules! coord_row {
    ( $parent_str:tt, $row_str:tt ) => {
        {
            let row: u32 = $row_str.parse::<u32>().unwrap();

            Row(coord!($parent_str), NonZeroU32::new(row).unwrap())
        }
    };
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
//    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    yew::start_app::<Model>();
    Ok(())
}
