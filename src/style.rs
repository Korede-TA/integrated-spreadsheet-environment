use serde::Deserialize;
use std::option::Option;

use crate::coordinate::Coordinate;
use crate::grammar::Kind;
use crate::model::Model;

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
    pub grid_column: String,   // CSS: span-color
    pub visibility: String,    //CSS visibility
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
            grid_column: "0".to_string(),
            visibility: "visible".to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        format! {
        "/* border: 1px; NOTE: ignoring Style::border_* for now */
border-collapse: {};
font-weight: {};
color: {};
visibility: {};
grid-column: {};\n",
        // self.border_color,
        if self.border_collapse { "collapse" } else { "inherit" },
        self.font_weight,
        self.font_color,
        self.visibility,
        self.grid_column,
        }
    }
}

pub fn get_style(model: &Model, coord: &Coordinate) -> String {
    let grammar = model
        .get_session()
        .grammars
        .get(coord)
        .expect("no grammar with this coordinate");
    // ignore root or meta
    if coord.row_cols.len() == 1 {
        return grammar.style(coord);
    }
    if grammar.style.width > 90.0 || grammar.style.height > 30.0 {
        return format!{
            "{}\nwidth: {}px;\nheight: {}px;\n",
            grammar.style(coord), grammar.style.width, grammar.style.height,
        };
    }
    if let Kind::Grid(_) = grammar.kind {
        return format! {
            "{}\nwidth: fit-content;\nheight: fit-content;\n",
            grammar.style(coord),
        };
    }
    let col_width = model.col_widths.get(&coord.full_col()).unwrap_or(&90.0);
    let row_height = model.row_heights.get(&coord.full_row()).unwrap_or(&30.0);
    format! {
        "{}\nwidth: {}px;\nheight: {}px;\n",
        grammar.style(coord), col_width, row_height,
    }
}
