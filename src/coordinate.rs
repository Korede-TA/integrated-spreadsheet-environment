use std::num::NonZeroU32;
use std::char::from_u32;
use std::option::Option;
use serde::Deserialize;
use std::panic;

use crate::grammar::{Grammar, Kind};
use crate::model::Model;
use crate::style::Style;
use crate::utils::coord_show;

// Coordinate specifies the nested coordinate structure
#[derive(Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
pub struct Coordinate {
    pub row_cols: Vec<(NonZeroU32, NonZeroU32)>, // should never be empty list
}
js_serializable!( Coordinate );
js_deserializable!( Coordinate );


impl Coordinate {
    pub fn child_of(parent: &Self, child_coord: (NonZeroU32, NonZeroU32)) -> Coordinate {
        let mut new_row_col = parent.clone().row_cols;
        new_row_col.push(child_coord);
        Coordinate{ row_cols: new_row_col }
    }

    pub fn parent(&self) -> Option<Coordinate> {
        if self.row_cols.len() == 1 {
            return None;
        }

        let parent = {
            let mut temp = self.clone();
            temp.row_cols.pop();
            temp
        };

        Some(parent)
    }

    pub fn to_string(&self) -> String {
        coord_show(self.row_cols.iter()
             .map(|(r, c)| (r.get(), c.get())).collect()).unwrap()
    }

    pub fn row(&self) -> NonZeroU32 {
        if let Some(last) = self.row_cols.last() {
            last.0
        } else {
            panic!{"a coordinate should always have a row, this one doesnt"}
        }
    }

    fn row_mut(&mut self) -> &mut NonZeroU32 {
        if let Some(last) = self.row_cols.last_mut() {
            &mut last.0
        } else {
            panic!{"a coordinate should always have a row, this one doesnt"}
        }
    }

    pub fn full_row(&self) -> Row {
        Row(self.parent().expect("full_row shouldn't be called on root or meta"), self.row())
    }

    pub fn row_to_string(&self) -> String {
        if let Some(parent) = self.parent() {
            format!{"{}-{}", parent.to_string(), self.row().get()}
        } else {
            format!{"{}", self.row().get()}
        }
    }

    pub fn col(&self) -> NonZeroU32 {
        if let Some(last) = self.row_cols.last() {
            last.1
        } else {
            panic!{"a coordinate should always have a column, this one doesnt"}
        }
    }

    fn col_mut(&mut self) -> &mut NonZeroU32 {
        if let Some(last) = self.row_cols.last_mut() {
            &mut last.1
        } else {
            panic!{"a coordinate should always have a column, this one doesnt"}
        }
    }

    pub fn full_col(&self) -> Col {
        Col(self.parent().expect("full_col shouldn't be called on root or meta"), self.col())
    }

    pub fn col_to_string(&self) -> String {
        if let Some(parent) = self.parent() {
            format!{"{}-{}", parent.to_string(), from_u32(self.col().get() + 64).unwrap()}
        } else {
            format!{"{}", from_u32(self.col().get() + 64).unwrap()}
        }
    }

    // if a cell is the parent, grandparent,..., (great xN)-grandparent of another
    // Optinoally returns: Some(N) if true (including N=0 if sibling),
    // or None if false
    fn is_n_parent(&self, other: &Self) -> Option<i32> {
        if self.row_cols.len() > other.row_cols.len() {
            return None
        }

        let mut n = 0;
        for (a, b) in self.row_cols.iter().zip(other.row_cols.iter()) {
           if a != b {
               break;
           }
            n += 1;
        }
        Some(n)
    }

    pub fn neighbor_above(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            if last.0.get() > 1 {
                *last = (/* row */ NonZeroU32::new(last.0.get() - 1).unwrap(), /* column */ last.1);
                return Some(Coordinate { row_cols: new_row_col })
            }
        }

        None
    }

    pub fn neighbor_below(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            *last = (/* row */ NonZeroU32::new(last.0.get() + 1).unwrap(), /* column */ last.1);
            return Some(Coordinate { row_cols: new_row_col })
        }

        None
    }

    pub fn neighbor_left(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            if last.1.get() > 1 {
                *last = (/* row */ last.0, /* column */ NonZeroU32::new(last.1.get() - 1).unwrap());
                return Some(Coordinate { row_cols: new_row_col })
            }
        }

        None
    }

    pub fn neighbor_right(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            *last = (/* row */ last.0, /* column */ NonZeroU32::new(last.1.get() + 1).unwrap());
            return Some(Coordinate { row_cols: new_row_col })
        }

        None
    }

}


#[derive(Debug, Clone, Hash)]
pub struct Row(/* parent */ pub Coordinate, /* row_index */ pub NonZeroU32);

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Row {}

#[derive(Debug, Clone, Hash)]
pub struct Col(/* parent */ pub Coordinate, /* col_index */ pub NonZeroU32);

impl PartialEq for Col {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Col {}
