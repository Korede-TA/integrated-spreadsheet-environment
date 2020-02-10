use std::char::from_u32;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;

use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Kind};
use crate::model::Model;


const DEFN_NAME_STR = "defn_subrule_name";
const DEFN_SUBRULE_NAME_STR = "defn_subrule_name";
const DEFN_SUBRULE_GRAMMAR_STR = "defn_subrule_grammar";

const DEFN_VARIANT_COORD = coord!("meta-A4");
const DEFN_REPITITION_COORD = coord!("meta-A5");
const DEFN_WILDCARD_COORD = coord!("meta-A5");

// get_suggestions proxies the Model.suggestions field to determine what kinds of
// completable values should be available within specific cells.
pub fn get_suggestions(m: &Model, coord: Coordinate) -> Vec<(Coordinate, Grammar) {
    let suggestion_list : Vec<(Coordinate, Grammar) = Vec::new();
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
        Grammar {name, ..} if name == DEFN_NAME_STR => (),
        Grammar {name, ..} if name == DEFN_SUBRULE_NAME_STR => (),

        Grammar {name, ..} if name == DEFN_SUBRULE_GRAMMAR_STR || coord!("meta").is_n_parent(coord) == Some(2) => {
            // include defn_variant and defn_repitition
            suggetion_list.append("variant", DEFN_VARIANT_COORD.clone());
            // suggetion_list.append("repitition", DEFN_REPITITION_COORD.clone());

            let subrule_names_col = coord.neighbor_left().unwrap();
            let subrule_name_suggestions = m.query_col(subrule_names_col)
                .filter_map(|suggestion_coord| {
                    if suggestion_coord == coord {
                        return None; // exclude current coord
                    }
                    if let Some(suggestion_grammar) =
                        m.get_session().grammars.get(&suggestion_coord)
                    {
                        Some((suggestion_coord.clone(), suggestion_grammar.clone()))
                    } else {
                        None
                    }
                });
            suggestion_list.extend(subrule_name_suggestions);
        },
        // TODO: figure out how to determine if grammar is nested within another grammar-def
        // perhaps, use name field to specify when grammar is pre-defined in meta
        _ => {
        let top_level_suggestions = m.query_col(coord_col!("meta", "A"))
            .filter_map(|suggestion_coord| {
                if let Some(suggestion_grammar) =
                    m.get_session().grammars.get(&suggestion_coord)
                {
                    Some((suggestion_coord.clone(), suggestion_grammar.clone()))
                } else {
                    None
                }
            });
        suggestion_list.extend(top_level_suggestions);
            }
    }
    }
    return suggestion_list;
}

