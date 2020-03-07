use serde::Deserialize;
use std::ops::Deref;
use std::option::Option;

use crate::coordinate::Coordinate;
use crate::grammar::Kind;
use crate::model::Model;
// use crate::style::Style;

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

pub fn get_style(model: &Model, coord: &Coordinate) -> String {
    let grammar = model
        .get_session()
        .grammars
        .get(coord)
        .expect("no grammar with this coordinate");
    // ignore root or meta
    if coord.row_cols.len() == 1 {
        info!(" gettt stylllleeee1 {:?}", grammar.style(coord));
        return grammar.style(coord);
        
    }
    if grammar.style.width > 90.0 || grammar.style.height > 30.0 {
        let (col_span, row_span, mut col_width, mut row_height) = {
            let s = &model
                .get_session()
                .grammars
                .get(&coord)
                .expect(format! {"grammar map should have coord {}", coord.to_string()}.deref())
                .style;
            (s.col_span, s.row_span, s.width, s.height)
        };
        let mut s_col_span = String::new();
        let mut s_row_span = String::new();
        let n_col_span = col_span.1 - col_span.0;
        let n_row_span = row_span.1 - row_span.0;
        col_width = col_width + (3 * n_col_span) as f64;
        row_height = row_height + (3 * n_row_span) as f64;
        info!("-------------------------------------------");
        info!("col_span {} - {}", col_span.0, col_span.1);
        info!("row_span {} - {}", row_span.0, row_span.1);
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
            "{}\nwidth: {}px;\nheight: {}px;
            {} {}",
            grammar.style(coord), col_width, row_height,
            s_col_span, s_row_span,
        };
    }
    if let Kind::Grid(_) = grammar.kind {
        info!(" gettt stylllleeee2 {:?}", format! {
            "{}\nwidth: fit-content;\nheight: fit-content;\n",
            grammar.style(coord),
        });
        return format! {
            "{}\nwidth: fit-content;\nheight: fit-content;\n",
            grammar.style(coord),
        };
    }
    let col_width = model.col_widths.get(&coord.full_col()).unwrap_or(&90.0);
    let row_height = model.row_heights.get(&coord.full_row()).unwrap_or(&30.0);
    info!(" gettt stylllleeee {:?}", format! {
        "{}\nwidth: {}px;\nheight: {}px;\n",
        grammar.style(coord), col_width, row_height,
    });
    format! {
        "{}\nwidth: {}px;\nheight: {}px;\n",
        grammar.style(coord), col_width, row_height,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_style_to_string() {
        assert_eq!(Style::default().to_string(),  
            "/* border: 1px; NOTE: ignoring Style::border_* for now */
    border-collapse: inherit;
    font-weight: 400;
    color: black;\n" )
}
}
