use std::collections::HashMap;
use std::num::NonZeroU32;
use std::char::from_u32;
use std::ops::Deref;
use std::option::Option;
use stdweb::web::{document, HtmlElement, IHtmlElement, INonElementParentNode};
use stdweb::unstable::TryFrom;

use crate::coordinate::{Coordinate};
use crate::grammar::{Grammar, Kind};
use crate::model::Model;
use crate::style::Style;
use crate::row_col_vec;

pub fn move_grammar(map: &mut HashMap<Coordinate, Grammar>, source: Coordinate, dest: Coordinate) {
    if let Some(source_grammar) = map.clone().get(&source) {
        map.insert(dest.clone(), source_grammar.clone());
        if let Kind::Grid(sub_coords) = source_grammar.clone().kind {
            for sub_coord in sub_coords {
                move_grammar(
                    map,
                    Coordinate::child_of(&source, sub_coord),
                    Coordinate::child_of(&dest, sub_coord)
                );
            }
        }
    }
}

pub fn non_zero_u32_tuple(val: (u32, u32)) -> (NonZeroU32, NonZeroU32) {
    let (row, col) = val;
    (NonZeroU32::new(row).unwrap(), NonZeroU32::new(col).unwrap())
}

pub fn row_col_to_string((row, col): (u32, u32)) -> String {
    let row_str = row.to_string();
    let col_str = from_u32(col + 64).unwrap();
    format!{"{}{}", col_str, row_str} 
}

pub fn coord_show(row_cols: Vec<(u32, u32)>) -> Option<String> {
    match row_cols.split_first() {
        Some((&(1,1), rest)) => {
            let mut output = "root".to_string();
            for rc in rest.iter() {
                output.push('-');
                output.push_str(row_col_to_string(*rc).deref());
            }
            Some(output)
        }
        Some((&(1,2), rest)) => {
            let mut output = "meta".to_string();
            for rc in rest.iter() {
                output.push('-');
                output.push_str(row_col_to_string(*rc).deref());
            }
            Some(output)
        }
        _ => None
    } 
}

pub fn apply_definition_grammar(m: &mut Model, root_coord: Coordinate) {
    // definition grammar contains the name of the grammar and then the list of
    // different parts of the grammar
    //
    //  ------------------------------
    //  | Definition |    { name }   |
    //  ------------------------------
    //  |----------------------------|
    //  || {rule name} | { rule     ||
    //  ||             |  grammar } ||
    //  ||-------------|------------||
    //  ||             |            ||
    //  ||             |            ||
    //  |--------(expandable)--------|
    //  ------------------------------
    //
    let defn_label_coord = Coordinate::child_of(&root_coord, non_zero_u32_tuple((1,1)));
    let mut defn_label_style = Style::default();
    defn_label_style.font_weight = 600;
    m.col_widths.insert(defn_label_coord.full_col(), 184.0); // set width of col
    let defn_label = Grammar {
        name: "defn_label".to_string(),
        style: defn_label_style,
        kind: Kind::Text("Define Grammar".to_string()),
    };
        
    let defn_name_coord = Coordinate::child_of(&root_coord, non_zero_u32_tuple((2,1)));
    m.col_widths.insert(defn_name_coord.full_col(), 184.0); // set width of col
    let defn_name = Grammar {
        name: "defn_name".to_string(),
        style: Style::default(),
        kind: Kind::Input(String::new()),
    };

    let defn_body_coord = Coordinate::child_of(&root_coord, non_zero_u32_tuple((3,1)));
    let mut defn_body = Grammar::as_grid(NonZeroU32::new(2).unwrap(), NonZeroU32::new(2).unwrap());
    defn_body.name = "defn_body".to_string();

    let defn_body_A1_coord = Coordinate::child_of(&defn_body_coord, non_zero_u32_tuple((1,1)));
    let defn_body_A1 = Grammar {
        name: "".to_string(),
        style: Style::default(),
        kind: Kind::Input(String::new()),
    };

    let defn_body_A2_coord = Coordinate::child_of(&defn_body_coord, non_zero_u32_tuple((2,1)));
    let defn_body_A2 = Grammar {
        name: "".to_string(),
        style: Style::default(),
        kind: Kind::Input(String::new()),
    };

    let defn_body_B1_coord = Coordinate::child_of(&defn_body_coord, non_zero_u32_tuple((1,2)));
    let defn_body_B1 = Grammar {
        name: "".to_string(),
        style: Style::default(),
        kind: Kind::Input(String::new()),
    };

    let defn_body_B2_coord = Coordinate::child_of(&defn_body_coord, non_zero_u32_tuple((2,2)));
    let defn_body_B2 = Grammar {
        name: "".to_string(),
        style: Style::default(),
        kind: Kind::Input(String::new()),
    };

    let defn = Grammar {
        name: "defn".to_string(),
        style: Style::default(),
        kind: Kind::Grid(row_col_vec![(1,1), (2,1), (3,1)]),
    };

    m.get_session_mut().grammars.insert(root_coord, defn);
    m.get_session_mut().grammars.insert(defn_name_coord, defn_name);
    m.get_session_mut().grammars.insert(defn_label_coord, defn_label);
    m.get_session_mut().grammars.insert(defn_body_coord, defn_body);
    m.get_session_mut().grammars.insert(defn_body_A1_coord, defn_body_A1);
    m.get_session_mut().grammars.insert(defn_body_A2_coord, defn_body_A2);
    m.get_session_mut().grammars.insert(defn_body_B1_coord, defn_body_B1);
    m.get_session_mut().grammars.insert(defn_body_B2_coord, defn_body_B2);
}

pub fn resize(m: &mut Model, coord: Coordinate, row_height: f64, col_width: f64) {
    if let Some(parent_coord) = coord.parent() {
        let mut row_height_diff = 0.0;
        let mut col_width_diff = 0.0;
        if let Some(old_row_height) = m.row_heights.get_mut(&coord.full_row()) {
            let new_row_height = row_height + /* horizontal border width */ 2.0;
            row_height_diff = new_row_height - *old_row_height;
            *old_row_height = new_row_height;
        }
        if let Some(old_col_width) = m.col_widths.get_mut(&coord.full_col()) {
            let new_col_width = col_width + /* vertiacl border height */ 2.0;
            col_width_diff = new_col_width - *old_col_width;
            *old_col_width = new_col_width;
        }
        info!{"resizing cell: (row: {}, col: {}); height: {}, width: {}", coord.row_to_string(), coord.col_to_string(),  row_height_diff, col_width_diff};
        resize_diff(m, parent_coord, row_height_diff, col_width_diff);
    }
}

pub fn resize_diff(m: &mut Model, coord: Coordinate, row_height_diff: f64, col_width_diff: f64) {
    if let Some(parent_coord) = coord.parent() {
        if let Some(row_height) = m.row_heights.get_mut(&coord.full_row()) {
            *row_height += row_height_diff + /* horizontal border width */ 2.0; 
        }
        if let Some(col_width) = m.col_widths.get_mut(&coord.full_col()) {
            *col_width += col_width_diff + /* vertical border height */ 2.0;
        }
        resize_diff(m, parent_coord, row_height_diff, col_width_diff);
    }
}

// Use width and height values from DOM to resize element
pub fn dom_resize(m: &mut Model, on: Coordinate) {
    let (height, width) = {
        let element = HtmlElement::try_from(
            document().get_element_by_id(format!{"cell-{}", on.to_string()}.deref()).unwrap()).unwrap();
        let rect = element.get_bounding_client_rect();
        (rect.get_height(), rect.get_width())
    };
    info!{"expanding...: H {}px, W {}px", height.clone(), width.clone()}
    resize(m, on, height, width);
    /*
    let on_grammar = map.get_mut(&on).unwrap();
    on_grammar.style.height = height.clone();
    on_grammar.style.width = width.clone();
    let parent_kind = on.parent().and_then(|p| map.get(&p)).map(|g| g.kind.clone());
    if let Some(Kind::Grid(neighbors_coords)) = parent_kind {
        for (row, col) in neighbors_coords {
            if let Some(cell) = map.get_mut(&Coordinate::child_of(&on, (row.clone(), col.clone()))) {
                if row == on.row() {
                    cell.style.height = height.clone();
                } else if col == on.col() {
                    cell.style.width = width.clone();
                }
            }
        }
    }
    */
}

// macro for easily defining a vector of non-zero tuples
// used in Coordinate::root() below
#[macro_export]
macro_rules! row_col_vec {
    ( $( $x:expr ), * ) => {
        {
            let mut v: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();
            $(
                v.push(non_zero_u32_tuple($x));
            )*
            v
        }
    };
}
