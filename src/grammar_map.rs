use pest::Parser;

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

#[derive(Clone)]
pub struct GrammarMap(HashMap<Coordinate, Grammar>);

#[derive(Clone)]
pub enum MapEntry {
    G(Grammar),
    // using `Box` here is necessary so Rust can infer a proper size of enum
    Grid(Vec<Vec<Box<MapEntry>>>),
}

pub fn build_grammar_map(
    map: &mut HashMap<Coordinate, Grammar>,
    root_coord: Coordinate,
    entry: MapEntry,
) {
    match entry {
        MapEntry::G(grammar) => {
            map.insert(root_coord, grammar);
        }
        MapEntry::Grid(entry_table) => {
            let mut sub_coords = vec![];
            let mut num_rows = 0;
            let mut num_cols = 0;
            for (row_i, entry_row) in entry_table.iter().enumerate() {
                num_rows = row_i + 1;
                for (col_i, entry) in entry_row.iter().enumerate() {
                    if col_i > num_cols {
                        num_cols = col_i + 1
                    }
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
                    style: {
                        let mut s = Style::default();
                        s.width = 90.0 * (num_cols as f64);
                        s.height = 30.0 * (num_rows as f64);
                        s
                    },
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
macro_rules! grid {
    [ $( [ $( $d:expr ),* ] ),* ] => {
        MapEntry::Grid(vec![
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
        let entry = grid![
            [
                g!(Grammar::text("", "A1")),
                g!(Grammar::text("", "B1")),
                g!(Grammar::text("", "C1"))
            ],
            [
                g!(Grammar::text("", "A2")),
                g!(Grammar::text("", "B2")),
                g!(Grammar::text("", "C2"))
            ],
            [
                g!(Grammar::text("", "A3")),
                g!(Grammar::text("", "B3")),
                grid![
                    [g!(Grammar::text("", "C3-A1")), g!(Grammar::text("", "C3-B1"))],
                    [g!(Grammar::text("", "C3-A2")), g!(Grammar::text("", "C3-B2"))]
                ]
            ]
        ];
        build_grammar_map(&mut map, coord!("root"), entry);
        assert_eq!(
            map.get(&(coord!("root-C3"))).unwrap().kind,
            Kind::Grid(vec![
                non_zero_u32_tuple((1, 1)),
                non_zero_u32_tuple((1, 2)),
                non_zero_u32_tuple((2, 1)),
                non_zero_u32_tuple((2, 2)),
            ])
        );
    }
}
