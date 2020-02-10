use serde::Deserialize;
use std::ops::Deref;
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
        }
    }

    pub fn to_string(&self) -> String {
        format! {
        "/* border: 1px; NOTE: ignoring Style::border_* for now */
border-collapse: {};
font-weight: {};
color: {};\n",
        // self.border_color,
        if self.border_collapse { "collapse" } else { "inherit" },
        self.font_weight,
        self.font_color,
        }
    }
}

pub fn get_style(model: &Model, coord: &Coordinate) -> String {
    let grammar = model
        .get_session()
        .grammars
        .get(coord)
        .expect(format! {"no grammar with this coordinate {}", coord.to_string()}.deref());
    match grammar.kind {
        Kind::Grid(_) | Kind::Defn(_, _, _) => grammar.style(coord).to_string(),
        _ => {
            let col_width = model.col_widths.get(&coord.full_col()).unwrap_or(&90.0);
            let row_height = model.row_heights.get(&coord.full_row()).unwrap_or(&30.0);
            format! {
                "{}\nwidth: {}px;\nheight: {}px;\n",
                grammar.style(coord), col_width, row_height,
            }
        }
    }
}
