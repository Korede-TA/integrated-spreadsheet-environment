use pest::Parser;
use std::char::from_u32;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;

use crate::coordinate::{Col, Coordinate, Row};
use crate::grammar::{Grammar, Kind};
use crate::model::Model;

use crate::{coord, coord_col, coord_row, row_col_vec};

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

pub const DEFN_NAME_STR: &str = "defn_subrule_name";
pub const DEFN_SUBRULE_NAME_STR: &str = "defn_subrule_name";
pub const DEFN_SUBRULE_GRAMMAR_STR: &str = "defn_subrule_grammar";

// get_suggestions proxies the Model.suggestions field to determine what kinds of
// completable values should be available within specific cells.
pub fn get_suggestions(m: &Model, coord: Coordinate) -> Vec<(Coordinate, Grammar)> {
    let DEFN_VARIANT_COORD: Coordinate = coord!("meta-A4");
    let DEFN_REPITITION_COORD: Coordinate = coord!("meta-A5");
    let DEFN_WILDCARD_COORD: Coordinate = coord!("meta-A5");

    let mut suggestion_list: Vec<(Coordinate, Grammar)> = Vec::new();
    if let Some(grammar) = m.get_session().grammars.get(&coord) {
        /*
         * - regular top-level grammars are suggested the top-level meta grammars
         * - nested grammars are suggested structures defined within nested meta grammas
         * - within the defn-grammar
         *      - the name portion is freeform
         *      - the grammar portion allows nested grids and, at any layer of nesting,
         *      permits either defn_variant, defn_repitition or sub-rule name grammars
         */
        match grammar {
            // no suggestions, free-form input,
            Grammar { name, .. } if name == DEFN_NAME_STR => (),
            Grammar { name, .. } if name == DEFN_SUBRULE_NAME_STR => (),

            // inside definition grammar (3+ levels deep in meta), and the second col
            Grammar { name, .. }
                if name == DEFN_SUBRULE_GRAMMAR_STR
                    || coord!("meta").is_n_parent(&coord).unwrap_or(0) >= 3 =>
            {
                info! {"getting suggestions for coord: {}", coord.to_string()};
                // include defn_variant and defn_repitition
                let defn_variant_grammar =
                    m.get_session().grammars.get(&DEFN_VARIANT_COORD).unwrap();
                suggestion_list.push((DEFN_VARIANT_COORD.clone(), defn_variant_grammar.clone()));
                // suggestion_list.push("repitition", DEFN_REPITITION_COORD.clone());

                let subrule_names_col = coord.neighbor_left().unwrap().full_col();
                for subrule_coord in m.query_col(subrule_names_col) {
                    if subrule_coord == coord {
                        continue; // exclude current coord
                    }
                    if let Some(suggestion_grammar) = m.get_session().grammars.get(&subrule_coord) {
                        suggestion_list.push((subrule_coord.clone(), suggestion_grammar.clone()));
                    }
                }
            }
            // TODO: figure out how to determine if grammar is nested within another grammar-def
            // perhaps, use name field to specify when grammar is pre-defined in meta
            _ => {
                for meta_coord in m.query_col(coord_col!("meta", "A")) {
                    if meta_coord == coord {
                        continue; // exclude current coord
                    }
                    if let Some(suggestion_grammar) = m.get_session().grammars.get(&meta_coord) {
                        suggestion_list.push((meta_coord.clone(), suggestion_grammar.clone()));
                    }
                }
            }
        }
    }
    return suggestion_list;
}
