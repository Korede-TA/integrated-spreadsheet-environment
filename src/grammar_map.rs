use std::collections::HashMap;
use std::num::NonZeroU32;

use crate::coord;
use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Kind};
use crate::style::Style;
use crate::util::non_zero_u32_tuple;

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

type C = Coordinate;
type G = Grammar;

#[derive(Clone)]
pub struct GrammarMap(HashMap<C, G>);

#[derive(Clone)]
pub enum MapEntry {
    G(Grammar),
    // using `Box` here is necessary so Rust can infer a proper size of enum
    GG(Vec<Vec<Box<MapEntry>>>),
}

pub fn build_grammar_map(map: &mut HashMap<C, G>, root_coord: Coordinate, entry: MapEntry) {
    match entry {
        MapEntry::G(grammar) => {
            map.insert(root_coord, grammar);
        }
        MapEntry::GG(entry_table) => {
            let mut sub_coords = vec![];
            for (row_i, entry_row) in entry_table.iter().enumerate() {
                for (col_i, entry) in entry_row.iter().enumerate() {
                    let new_coord = Coordinate::child_of(
                        &root_coord,
                        non_zero_u32_tuple(((row_i + 1) as u32, (col_i + 1) as u32)),
                    );
                    build_grammar_map(map, new_coord, (**entry).clone());
                    sub_coords.push(non_zero_u32_tuple(((row_i + 1) as u32, (col_i + 1) as u32)));
                }
            }
            map.insert(
                root_coord,
                Grammar {
                    name: String::new(),
                    style: Style::default(),
                    kind: Kind::Grid(sub_coords),
                },
            );
        }
    }
}

#[macro_export]
macro_rules! g {
    ( $grammar:expr ) => {
        MapEntry::G($grammar)
    };
}

#[macro_export]
macro_rules! gg {
    [ $( [ $( $d:expr ),* ] ),* ] => {
        MapEntry::GG(vec![
            $(
                vec![$(Box::new($d)),*],
            )*
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_grammar_map() {
        let mut map = HashMap::new();
        let entry = gg![
            [
                g!(Grammar::text("A1")),
                g!(Grammar::text("B1")),
                g!(Grammar::text("C1"))
            ],
            [
                g!(Grammar::text("A2")),
                g!(Grammar::text("B2")),
                g!(Grammar::text("C2"))
            ],
            [
                g!(Grammar::text("A3")),
                g!(Grammar::text("B3")),
                gg![
                    [Grammar::text("C3-A1"), Grammar::text("C3-B1")],
                    [Grammar::text("C3-A2"), Grammar::text("C3-B2")]
                ],
            ]
        ];
        build_grammar_map(&mut map, &coord!("root"), entry);
        assert_eq!(map.keys().len(), 13);
    }
}
