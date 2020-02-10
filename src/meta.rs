use pest::Parser;
use std::num::NonZeroU32;

use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Kind};
use crate::model::Model;
use crate::suggestion::get_suggestions;

use crate::coord;

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

pub const DEFN_NAME_STR: &str = "defn_subrule_name";
pub const DEFN_SUBRULE_NAME_STR: &str = "defn_subrule_name";
pub const DEFN_SUBRULE_GRAMMAR_STR: &str = "defn_subrule_grammar";

pub fn transform_from_meta_representation(
    m: &Model,
    coord: Coordinate,
) -> (Kind, Vec<(Coordinate, Grammar)>) {
    /* this is a utility function for transforming grammars from their meta
     * representation to their representation within the root.
     *
     * specifically, two types of transformations/augmentations need to happen when a
     * definition grammar is copied from the meta into the root
     * - grammars representing special definition rules ("defn_variant" and "defn_repitition")
     *   are into regular input grammars (by changing their Kind)
     * - input grammars transformed from "defn_variants" include completion suggestions for all
     *   the specified variants
     *
     * This involves modifying the `grammar.kind` and the list of suggestions passed into
     * `view_input_grammar`. Hence returning a tuple of (Kind, Vec<(Coordinate, Grammar)>)
     */

    let grammar = m.get_session().grammars.get(&coord).unwrap();
    let default = (grammar.kind.clone(), get_suggestions(m, coord.clone()));
    if coord.row_cols.len() < 3 {
        return default;
    }

    let truncated_coord = coord.truncate(3).unwrap();
    if coord!("meta").is_n_parent(&coord).unwrap_or(0) >= 3 && truncated_coord.col().get() == 2 {
        let defn_grammar_root = truncated_coord.parent().unwrap();
        let mut suggestions = get_suggestions(m, coord.clone());
        if let Some(Grammar {
            kind: Kind::Grid(sub_coords),
            ..
        }) = m.get_session().grammars.get(&defn_grammar_root)
        {
            info! {"TODO"};
            for c in sub_coords {
                if (*c).1.get() == 1 {
                    // first col of defn grammar, which is the definition names
                    let full_coord = Coordinate::child_of(&defn_grammar_root, *c);
                    if let Some(g) = m.get_session().grammars.get(&full_coord) {
                        if g.name == DEFN_SUBRULE_NAME_STR {
                            suggestions.push((full_coord, g.clone()));
                        }
                    }
                }
            }
        }
        (Kind::Input(String::new()), suggestions)
    } else {
        default
    }
}
