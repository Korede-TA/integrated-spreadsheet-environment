use pest::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::option::Option;
use std::string::String;
use std::vec::Vec;

use crate::coordinate;
use crate::coordinate::*;
use crate::grammar;
use crate::grammar::{Grammar, Interactive, Kind, Lookup};
use crate::model::Model;
use crate::util::non_zero_u32_tuple;
use crate::{coord, coord_col, coord_row, row_col_vec};
use yew::html::Component;

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

// Style contains the relevant CSS properties for styling
// a grammar Cell or Grid
#[derive(Deserialize, Debug, Clone)]
pub struct Style {
    pub width: f64,            // CSS: width
    pub height: f64,           // CSS: height
    pub border_color: String,  // CSS: border-color
    pub border_collapse: bool, // CSS: border-collapse
    pub font_weight: i32,      // CSS: font-weight
    pub font_color: String,    // CSS: font-color
    pub col_span: (u32, u32),
    pub row_span: (u32, u32),
    pub display: bool,
}
js_serializable!(Style);
js_deserializable!(Style);

impl Style {
    pub fn default() -> Style {
        Style {
            width: 90.00,
            height: 30.00,
            border_color: "grey".to_string(),
            border_collapse: false,
            font_weight: 400,
            font_color: "black".to_string(),
            col_span: (0, 0),
            row_span: (0, 0),
            display: true,
        }
    }

    pub fn to_string(&self) -> String {
        format! {
        "/* border: 1px; NOTE: ignoring Style::border_* for now */
border-collapse: {};
font-weight: {};
color: {};
\n",
        // self.border_color,
        if self.border_collapse { "collapse" } else { "inherit" },
        self.font_weight,
        self.font_color,
        }
    }
}

pub fn get_style(
    model_grammar: &Grammar,
    model_col_widths: &HashMap<coordinate::Col, f64>,
    model_row_heights: &HashMap<coordinate::Row, f64>,
    coord: &Coordinate,
) -> String {
    let grammar = model_grammar;
    // ignore root or meta

    if coord.row_cols.len() == 1 {
        return grammar.style(coord);
    }
    let (col_span, row_span, mut col_width, mut row_height) = {
        let s = &model_grammar.style;
        (s.col_span, s.row_span, s.width, s.height)
    };
    let mut s_col_span = String::new();
    let mut s_row_span = String::new();
    let n_col_span = col_span.1 - col_span.0;
    let n_row_span = row_span.1 - row_span.0;
    col_width = col_width + n_col_span as f64;
    row_height = row_height + n_row_span as f64;

    if n_col_span != 0 || n_row_span != 0 {
        if n_col_span != 0 {
            s_col_span = format! {
                "\ngrid-column-start: {}; grid-column: {} / span {};",
                col_span.0.to_string(), col_span.0.to_string(), col_span.1.to_string(),
            };
        }
        if n_row_span != 0 {
            s_row_span = format! {
                "\ngrid-row-start: {}; grid-row: {} / span {};",
                row_span.0.to_string(), row_span.0.to_string(), row_span.1.to_string(),
            };
        }
        return format! {
            "{}\nwidth: {}px;\nheight: {}px;{} {}",
            grammar.style(coord), col_width, row_height,
            s_col_span, s_row_span,
        };
    }

    if let Kind::Grid(_) = grammar.kind {
        return format! {
            "{}\nwidth: fit-content;\nheight: fit-content;\n",
            grammar.style(coord),
        };
    }
    let col_width = model_col_widths.get(&coord.full_col()).unwrap_or(&90.0);
    let row_height = model_row_heights.get(&coord.full_row()).unwrap_or(&30.0);
    format! {
        "{}\nwidth: {}px;\nheight: {}px;\n",
        grammar.style(coord), col_width, row_height,
    }
}

pub enum Dimension {
    MaxContent,
    MinContent,
    FitContent,
    Px(f64),
    Percentage(f64),
}

impl Dimension {
    fn to_string(&self) -> String {
        match self {
            Dimension::MaxContent => "max-content".to_string(),
            Dimension::MinContent => "min-content".to_string(),
            Dimension::FitContent => "fit-content".to_string(),
            Dimension::Px(x) => format! {"{}px", x},
            Dimension::Percentage(x) => format! {"{}%", x},
        }
    }
}


mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_style_to_string() {
        assert_eq!(Style::default().to_string(),  String::from("/* border: 1px; NOTE: ignoring Style::border_* for now */\nborder-collapse: inherit;\nfont-weight: 400;\ncolor: black;\n\n"));
        // assert_ne!(Style::default().to_string(),  String::from("/* border: 1px; NOTE: ignoring Style::border_* for now */\n    border-collapse: inherit;\n    font-weight: 400;\n    color: black;\n" ));
    }

    #[test]
    fn test_get_style() {
        //Test type Grid
        assert_eq!(get_style(&grammar::Grammar {name: "root".to_string(), style: Style::default(), kind: Kind::Grid(row_col_vec![(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)]),}, &hashmap! { coord_col!("root","A") => 90.0, coord_col!("root","B") => 90.0, coord_col!("meta","A") => 180.0, coord_col!("meta-A3","A") => 90.0, coord_col!("meta-A3","B") => 180.0,}, &hashmap! {coord_row!("root","1") => 30.0, coord_row!("root","2") => 30.0, coord_row!("root","3") => 30.0,coord_row!("meta","1") => 180.0,}, &coord!("root-A1") ),
        String::from("display: grid;\ngrid-area: cell-root-A1;\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n\"cell-root-A1-A1 cell-root-A1-B1\"\n\"cell-root-A1-A2 cell-root-A1-B2\"\n\"cell-root-A1-A3 cell-root-A1-B3\";\n\nwidth: fit-content;\nheight: fit-content;\n"));
        assert_ne!(get_style(&grammar::Grammar {name: "root".to_string(), style: Style::default(), kind: Kind::Grid(row_col_vec![(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)]),}, &hashmap! { coord_col!("root","A") => 90.0, coord_col!("root","B") => 90.0, coord_col!("meta","A") => 180.0, coord_col!("meta-A3","A") => 90.0, coord_col!("meta-A3","B") => 180.0,}, &hashmap! {coord_row!("root","1") => 30.0, coord_row!("root","2") => 30.0, coord_row!("root","3") => 30.0,coord_row!("meta","1") => 180.0,}, &coord!("root-A1") ),
        String::from("display: grid;\ngrid-area: cell-root-B1;\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n\"cell-root-A1-A1 cell-root-A1-C1\"\n\"cell-root-A1-A2 cell-root-A1-B2\"\n\"cell-root-A1-A3 cell-root-A1-B3\";\n\nwidth: fit-content;\nheight: fit-content;\n"));

        //Test Row_cols length == 1
        assert_eq!(get_style(&grammar::Grammar {name: "root".to_string(), style: Style::default(), kind: Kind::Grid(row_col_vec![(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)]),}, &hashmap! { coord_col!("root","A") => 90.0, coord_col!("root","B") => 90.0, coord_col!("meta","A") => 180.0, coord_col!("meta-A3","A") => 90.0, coord_col!("meta-A3","B") => 180.0,}, &hashmap! {coord_row!("root","1") => 30.0, coord_row!("root","2") => 30.0, coord_row!("root","3") => 30.0,coord_row!("meta","1") => 180.0,}, &coord!("root") ),
        String::from("display: grid;\ngrid-area: cell-root;\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n\"cell-root-A1 cell-root-B1\"\n\"cell-root-A2 cell-root-B2\"\n\"cell-root-A3 cell-root-B3\";\n"));

        //Test Kind input
        assert_eq!(get_style(&grammar::Grammar {name: "root".to_string(), style: Style::default(), kind: Kind::Input(String::default())}, &hashmap! { coord_col!("root","A") => 90.0, coord_col!("root","B") => 90.0, coord_col!("meta","A") => 180.0, coord_col!("meta-A3","A") => 90.0, coord_col!("meta-A3","B") => 180.0,}, &hashmap! {coord_row!("root","1") => 30.0, coord_row!("root","2") => 30.0, coord_row!("root","3") => 30.0,coord_row!("meta","1") => 180.0,}, &coord!("root") ),
        String::from("/* border: 1px; NOTE: ignoring Style::border_* for now */\nborder-collapse: inherit;\nfont-weight: 400;\ncolor: black;\n\ngrid-area: cell-root;\n"));

        //Test Type interractive =>  Button as exemple
        assert_eq!(get_style(&grammar::Grammar {name: "root".to_string(), style: Style::default(), kind: Kind::Interactive(String::from("Test"), Interactive::Button())}, &hashmap! { coord_col!("root","A") => 90.0, coord_col!("root","B") => 90.0, coord_col!("meta","A") => 180.0, coord_col!("meta-A3","A") => 90.0, coord_col!("meta-A3","B") => 180.0,}, &hashmap! {coord_row!("root","1") => 30.0, coord_row!("root","2") => 30.0, coord_row!("root","3") => 30.0,coord_row!("meta","1") => 180.0,}, &coord!("root") ),
        String::from("/* border: 1px; NOTE: ignoring Style::border_* for now */\nborder-collapse: inherit;\nfont-weight: 400;\ncolor: black;\n\ngrid-area: cell-root;\n"));

        // Test Type Lookup // Have to figureout the arguments
        assert_eq!(get_style(&grammar::Grammar {name: "root".to_string(), style: Style::default(), kind: Kind::Lookup(String::default(), std::option::Option::default())}, &hashmap! { coord_col!("root","A") => 90.0, coord_col!("root","B") => 90.0, coord_col!("meta","A") => 180.0, coord_col!("meta-A3","A") => 90.0, coord_col!("meta-A3","B") => 180.0,}, &hashmap! {coord_row!("root","1") => 30.0, coord_row!("root","2") => 30.0, coord_row!("root","3") => 30.0,coord_row!("meta","1") => 180.0,}, &coord!("root") ),
        String::from("/* border: 1px; NOTE: ignoring Style::border_* for now */\nborder-collapse: inherit;\nfont-weight: 400;\ncolor: black;\n\ndisplay: inline-flex; grid-area: cell-root; background: white;\n"));
    }

    #[test]
    fn test_dimensio_to_string() {
        assert_eq!(Dimension::FitContent.to_string(), "fit-content".to_string());
        assert_eq!(Dimension::MaxContent.to_string(), "max-content".to_string());
        assert_eq!(Dimension::MinContent.to_string(), "min-content".to_string());
        assert_eq!(Dimension::Percentage(2.0).to_string(), "2%".to_string());
        assert_eq!(Dimension::Px(2.0).to_string(), "2px".to_string());
    }
}