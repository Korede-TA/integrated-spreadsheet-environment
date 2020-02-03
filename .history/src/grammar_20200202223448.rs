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
    pub grid_list : Vec<(NonZeroU32, NonZeroU32)>,
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
    Grid((NonZeroU32, NonZeroU32)),
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
            grid_list: vec!((std::num::NonZeroU32::new(1).unwrap(), std::num::NonZeroU32::new(1).unwrap())),
        }
    }

    pub fn style(&self, coord: &Coordinate) -> String {

        match &self.kind {
            Kind::Grid(sub_coords) => {
                let mut grid_area_str = "\"".to_string();
                let mut prev_row = 1;
                let mut row = 1;
                let mut col = 2;
                
                
                while row == sub_coords.0.get() && col == sub_coords.1.get() {
                    if row > prev_row {
                        grid_area_str.pop();
                        grid_area_str += "\"\n\"";
                    }
                    grid_area_str += format!{"cell-{}{} ", row.to_string(), col.to_string()}.deref();
                    col += 1;
                    
                    if col == sub_coords.1.get(){
                        prev_row = row;
                        row += 1;
                        col = 1;
                    }
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
            grid_list: vec!((std::num::NonZeroU32::new(1).unwrap(), std::num::NonZeroU32::new(1).unwrap())),
        }
    }

    pub fn as_grid(rows: NonZeroU32, cols: NonZeroU32) -> Grammar {
        let mut grid = vec!((std::num::NonZeroU32::new(1).unwrap(), std::num::NonZeroU32::new(1).unwrap()));
        for i in 2..(rows.get() + 1) {
             for j in 2..(cols.get() + 1) {
                 grid.push((NonZeroU32::new(i).unwrap(), NonZeroU32::new(j).unwrap()));
             }
         }

        Grammar {
            name: "".to_string(),
            style: Style::default(),
            kind: Kind::Grid((rows, cols)),
            grid_list: grid,
        }
    }
}

#[macro_export]
macro_rules! get_grid {
    ( $sub_coords:tt ) => {
        {
            let mut col = 1;
            let mut row = 2;
            let mut c = vec!((std::num::NonZeroU32::new(1).unwrap(), std::num::NonZeroU32::new(1).unwrap()));
            while col < $sub_coords.1.get() + 1{
                c.push((std::num::NonZeroU32::new(row.clone()).unwrap(), std::num::NonZeroU32::new(col.clone()).unwrap())) ;
                if row == $sub_coords.0.get(){
                    row = 1;
                    col +=1;
                }else{
                    row += 1;
                }   
            }
            c
        }
    };
}




