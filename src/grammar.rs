use std::num::NonZeroU32;
use std::ops::Deref;
use std::cmp::Ordering;
use std::option::Option;
use serde::Deserialize;

use crate::coordinate::Coordinate;
use crate::model::Model;
use crate::style::Style;

// Grammar is the main data-type representing
// the contents of a cell
#[derive(Deserialize, Debug, Clone)]
pub struct Grammar {
    pub name: String,
    pub style: Style,
    pub kind: Kind,
}
js_serializable!( Grammar );
js_deserializable!( Grammar );

// Kinds of grammars in the system.
// Since this is an Enum, a Grammar's kind field
// can only be set to one these variants at a time
#[derive(Deserialize, Debug, Clone)]
pub enum Kind {
    Text(String),
    Input(String),
    Interactive(String, Interactive),
    Grid(Vec<(NonZeroU32, NonZeroU32)>),
}
js_serializable!( Kind );
js_deserializable!( Kind );

// Kinds of interactive grammars
#[derive(Deserialize, Debug, Clone)]
pub enum Interactive {
    Button(),
    Slider(/*value*/ f64, /*min*/ f64, /*max*/ f64),
    Toggle(bool),
}

impl Grammar {
    pub fn default() -> Grammar {
        Grammar {
            name: "".to_string(),
            style: Style::default(),
            kind: Kind::Input("".to_string()),
        }
    }

    pub fn style(&self, coord: &Coordinate) -> String {
        match &self.kind {
            Kind::Grid(sub_coords) => {
                let mut grid_area_str = "\"".to_string();
                let mut prev_row = 1;
                let mut sub_coords = sub_coords.clone();
                sub_coords.sort_by(|(a_row, a_col), (b_row, b_col)| 
                    if a_row < b_row {
                        Ordering::Less
                    } else if a_row == b_row {
                        if a_col < b_col { Ordering::Less } else { Ordering::Greater }
                    } else { Ordering::Greater }
                );
                for (row, col) in sub_coords {
                    if row.get() > prev_row {
                        grid_area_str.pop();
                        grid_area_str += "\"\n\"";
                    }
                    let sub_coord = Coordinate::child_of(coord, (row.clone(), col.clone()));
                    grid_area_str += format!{"cell-{} ", sub_coord.to_string()}.deref();
                    prev_row = row.get();
                }
                grid_area_str.pop();
                grid_area_str += "\"";
                format!{
                    "display: grid;\ngrid-area: cell-{};\nheight: fit-content;\nwidth: fit-content !important;\ngrid-template-areas: \n{};\n",
                    coord.to_string(),
                    grid_area_str,
                }
            },
            _ => format!{"{}grid-area: cell-{};\n", self.style.to_string(), coord.to_string()},
        }
    }

    pub fn suggestion(alias : String, value: String) -> Grammar {
        Grammar {
            name: alias,
            style: Style::default(),
            kind: Kind::Text(value),
        }
    }

    pub fn as_grid(rows: NonZeroU32, cols: NonZeroU32) -> Grammar {
        let mut grid : Vec<(NonZeroU32, NonZeroU32)> = Vec::new();
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




