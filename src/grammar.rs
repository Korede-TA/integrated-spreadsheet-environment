use pest::Parser;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default::Default;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;
use std::string::String;
use std::vec::Vec;

use crate::coordinate::*;
use crate::coordinate::{Col, Coordinate, Row};
use crate::grammar;
use crate::style::Style;
use crate::util::non_zero_u32_tuple;
use crate::{coord, coord_col, coord_row, row_col_vec};

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

// Grammar is the main data-type representing
// the contents of a cell
#[derive(Deserialize, Debug, Clone)]
pub struct Grammar {
    pub name: String,
    pub style: Style,
    pub kind: Kind,
}
js_serializable!(Grammar);
js_deserializable!(Grammar);

// Kinds of grammars in the system.
// Since this is an Enum, a Grammar's kind field
// can only be set to one these variants at a time
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Kind {
    // Read-only text grammar
    Text(String),

    // Readable and writable text grammar
    Input(String),

    // Structural grammar that nests a grid of grammars
    Grid(Vec<(NonZeroU32, NonZeroU32)>),

    // Interactive Grammars
    Interactive(String, Interactive),

    // Lookup grammar
    // in the context of definitions, these bind to cell bindings
    Lookup(String, Option<Lookup>),

    // Definition grammar
    // sort of like a mirror to the meta-table that creates new grammars and
    // specifies valid completions
    Defn(
        /* binding name */ String,
        /* definition coord */ Coordinate,
        /* rule names and coordinates */ Vec<(String, Coordinate)>,
    ),

    Editor(/* content */ String),
}
js_serializable!(Kind);
js_deserializable!(Kind);

// Kinds of lookup grammars
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Lookup {
    Cell(Coordinate),
    Range {
        parent: Coordinate,
        start: (NonZeroU32, NonZeroU32),
        end: (NonZeroU32, NonZeroU32),
    },
    Row(Row),
    Col(Col),
}

// Kinds of interactive grammars
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Interactive {
    Button(),
    Slider(/*value*/ f64, /*min*/ f64, /*max*/ f64),
    Toggle(bool),
}

impl Default for Grammar {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            style: Style::default(),
            kind: Kind::Input("".to_string()),
        }
    }
}

impl Grammar {
    pub fn style(&self, coord: &Coordinate) -> String {
        match &self.kind {
            Kind::Grid(sub_coords) => {
                let mut grid_area_str = "\"".to_string();
                let mut prev_row = 1;
                let mut sub_coords = sub_coords.clone();
                sub_coords.sort_by(|(a_row, a_col), (b_row, b_col)| {
                    if a_row < b_row {
                        Ordering::Less
                    } else if a_row == b_row {
                        if a_col < b_col {
                            Ordering::Less
                        } else {
                            Ordering::Greater
                        }
                    } else {
                        Ordering::Greater
                    }
                });
                for (row, col) in sub_coords {
                    if row.get() > prev_row {
                        grid_area_str.pop();
                        grid_area_str += "\"\n\"";
                    }
                    let sub_coord = Coordinate::child_of(coord, (row.clone(), col.clone()));
                    grid_area_str += format! {"cell-{} ", sub_coord.to_string()}.deref();
                    prev_row = row.get();
                }
                grid_area_str.pop();
                grid_area_str += "\"";
                format! {
                    "display: grid;\ngrid-area: cell-{};\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n{};\n",
                    coord.to_string(),
                    grid_area_str,
                }
            }
            Kind::Lookup(_, _) => format! {
                "{}display: inline-flex; grid-area: cell-{}; background: white;\n", self.style.to_string(), coord.to_string()
            },
            _ => format! {"{}grid-area: cell-{};\n", self.style.to_string(), coord.to_string()},
        }
    }

    // NOTE: more info on this pattern here: https://hermanradtke.com/2015/05/06/creating-a-rust-function-that-accepts-string-or-str.html
    pub fn text<S>(name: S, value: S) -> Grammar
    where
        S: Into<String>,
    {
        Grammar {
            name: name.into(),
            style: Style::default(),
            kind: Kind::Text(value.into()),
        }
    }

    pub fn input<S>(name: S, value: S) -> Grammar
    where
        S: Into<String>,
    {
        Grammar {
            name: name.into(),
            style: Style::default(),
            kind: Kind::Input(value.into()),
        }
    }

    pub fn default_button() -> Grammar {
        Grammar {
            name: "button".to_string(),
            style: Style::default(),
            kind: Kind::Interactive("".to_string(), Interactive::Button()),
        }
    }

    pub fn default_slider() -> Grammar {
        Grammar {
            name: "slider".to_string(),
            style: Style::default(),
            kind: Kind::Interactive("".to_string(), Interactive::Slider(0.0, 0.0, 100.0)),
        }
    }

    pub fn default_toggle() -> Grammar {
        Grammar {
            name: "toggle".to_string(),
            style: Style::default(),
            kind: Kind::Interactive("".to_string(), Interactive::Toggle(false)),
        }
    }

    pub fn as_grid(rows: NonZeroU32, cols: NonZeroU32) -> Grammar {
        let mut grid: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();
        for i in 1..(rows.get() + 1) {
            for j in 1..(cols.get() + 1) {
                grid.push((NonZeroU32::new(i).unwrap(), NonZeroU32::new(j).unwrap()));
            }
        }

        Grammar {
            name: "".to_string(),
            style: Style::default(),
            kind: Kind::Grid(grid),
        }
    }
}

#[macro_export]
macro_rules! grammar_table {
	($([$($content:tt)*]), *) => (
		HashMap::<Coordinate, Grammar>::from_iter(vec![$(vec![$($content)*]), *].into_iter().flatten().collect())
	);

    /*
    (@step $_idx:expr,) => {};

    (@step $idx:expr, $head:ident, $($tail:ident,)*) => {
        impl A {
            fn $head(&self) -> i32 {
                self.data[$idx]
            }
        }

        grammar_table!(@step $idx + 1usize, $($tail,)*);
    };

    ($($n:ident),*) => {
        grammar_table!(@step 0usize, $($n,)*);
    }
    */
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_default_grammar() {
        assert_eq!(Grammar::default().kind, Kind::Input("".to_string()));
        assert_ne!(Grammar::default().kind, Kind::Text("".to_string()));
        assert_eq!(Grammar::default().name, "".to_string());
        assert_ne!(Grammar::default().name, " ");
        assert_eq!(
            Grammar::default().style.to_string(),
            Style::default().to_string()
        );
    }

    #[test]
    fn test_grammar_style() {
        assert_eq!(
            Grammar::default().style(&coord!("root-A1")),
            format! {"/* border: 1px; NOTE: ignoring Style::border_* for now */\nborder-collapse: inherit;\nfont-weight: 400;\ncolor: black;\n\ngrid-area: cell-root-A1;\n"}
        );
        assert_ne!(
            Grammar::default().style(&coord!("root-A1")),
            format! {"display: grid;\ngrid-area: cell-root-A1;\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n\"cell-root-A1-A1 cell-root-A1-B1\";\n"}
        );
        // Type Grid
        assert_eq!(
            Grammar::as_grid(NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap())
                .style(&coord!("root-A1")),
            format! {"display: grid;\ngrid-area: cell-root-A1;\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n\"cell-root-A1-A1 cell-root-A1-B1\";\n"}
        );
        assert_ne!(
            Grammar::as_grid(NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap())
                .style(&coord!("root-A1")),
            format! {"/* border: 1px; NOTE: ignoring Style::border_* for now */\nborder-collapse: inherit;\nfont-weight: 400;\ncolor: black;\n\ngrid-area: cell-root-A1;\n"}
        );
    }

    #[test]
    fn test_grammar_text() {
        assert_eq!(
            Grammar::text("testing", "testing").name,
            "testing".to_string(),
        );

        assert_eq!(
            Grammar::text("testing", "testing").style.to_string(),
            Style::default().to_string()
        );
    }

    #[test]
    fn test_grammar_input() {
        assert_eq!(
            Grammar::input("testing", "testing").name,
            "testing".to_string(),
        );

        assert_eq!(
            Grammar::input("testing", "testing").style.to_string(),
            Style::default().to_string()
        );
        assert_ne!(
            Grammar::input("testing", "testing").kind,
            Kind::Input("testing ".to_string())
        );
    }

    #[test]
    fn test_default_button() {
        assert_eq!(Grammar::default_button().name, "button".to_string());

        assert_eq!(
            Grammar::default_button().style.to_string(),
            Style::default().to_string()
        );

        assert_ne!(
            Grammar::default_button().kind,
            Kind::Interactive(" ".to_string(), Interactive::Button())
        );
    }

    #[test]
    fn test_default_slider() {
        assert_eq!(Grammar::default_slider().name, "slider".to_string());

        assert_eq!(
            Grammar::default_slider().style.to_string(),
            Style::default().to_string()
        );

        assert_ne!(
            Grammar::default_slider().kind,
            Kind::Interactive(" ".to_string(), Interactive::Slider(0.0, 0.0, 100.0))
        );
    }

    #[test]
    fn test_default_toggle() {
        assert_eq!(Grammar::default_toggle().name, "toggle".to_string());

        assert_eq!(
            Grammar::default_toggle().style.to_string(),
            Style::default().to_string()
        );

        assert_ne!(
            Grammar::default_toggle().kind,
            Kind::Interactive(" ".to_string(), Interactive::Toggle(false))
        );
    }

    #[test]
    fn test_as_grid() {
        assert_eq!(
            Grammar::as_grid(NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap()).name,
            "".to_string()
        );

        assert_eq!(
            Grammar::as_grid(NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap())
                .style
                .to_string(),
            Style::default().to_string()
        );

        assert_eq!(
            Grammar::as_grid(NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap()).kind,
            Kind::Grid(vec![non_zero_u32_tuple((1, 1)), non_zero_u32_tuple((1, 2))])
        );
    }
}
