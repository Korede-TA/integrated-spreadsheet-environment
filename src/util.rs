#![feature(core_intrinsics)]
use std::char::from_u32;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;
use stdweb::unstable::TryFrom;
use stdweb::web::{document, HtmlElement, IHtmlElement, INonElementParentNode};
use stdweb::Value;

use crate::coordinate::{Col, Coordinate, Row};
use crate::grammar::{Grammar, Kind};
use crate::grammar_map::*;
use crate::model::Model;
use crate::style::Style;
use crate::{g, grid, row_col_vec};

// `move_grammar` function does all the necessary operations when copying nested grammars from one
// coordinate in the grid to another including:
// - copying each nested grammar all the way to the innermost cell
// - adjusting the sizes of the grammars in row_heights and col_widths
//
// TODO:
// - add error return value that can be checked to see if grammar move was successful
// - add UNIT TEST to ensure that destination coord is only manipulated if source coord exists
// - (maybe) incorporate dom_resize to get correct values
pub fn move_grammar(m: &mut Model, source: Coordinate, dest: Coordinate) {
    if let Some(source_grammar) = m.get_session_mut().grammars.clone().get(&source) {
        // copy source grammar from map and insert into destination coordinate
        m.get_session_mut()
            .grammars
            .insert(dest.clone(), source_grammar.clone());
        // resizes new grammar
        let row_height = m.row_heights.get(&source.full_row()).unwrap_or(&30.0);
        let col_width = m.col_widths.get(&source.full_col()).unwrap_or(&90.0);
        resize(m, dest.clone(), *row_height, *col_width);
        // copying over child grammar values
        if let Kind::Grid(sub_coords) = source_grammar.clone().kind {
            for sub_coord in sub_coords {
                move_grammar(
                    m,
                    Coordinate::child_of(&source, sub_coord),
                    Coordinate::child_of(&dest, sub_coord),
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
    format! {"{}{}", col_str, row_str}
}

pub fn coord_show(row_cols: Vec<(u32, u32)>) -> Option<String> {
    match row_cols.split_first() {
        Some((&(1, 1), rest)) => {
            let mut output = "root".to_string();
            for rc in rest.iter() {
                output.push('-');
                output.push_str(row_col_to_string(*rc).deref());
            }
            Some(output)
        }
        Some((&(1, 2), rest)) => {
            let mut output = "meta".to_string();
            for rc in rest.iter() {
                output.push('-');
                output.push_str(row_col_to_string(*rc).deref());
            }
            Some(output)
        }
        _ => None,
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
    let mut defn_label_style = Style::default();
    defn_label_style.font_weight = 600;
    m.col_widths.insert(root_coord.full_col(), 184.0); // set width of col
    m.row_heights.insert(root_coord.full_row(), 184.0); // set width of col
    build_grammar_map(
        &mut m.get_session_mut().grammars,
        root_coord,
        grid![
            [
                g!(Grammar {
                    name: "defn_label".to_string(),
                    style: defn_label_style,
                    kind: Kind::Text("Define Grammar".to_string()),
                }),
                g!(Grammar {
                    name: "defn_name".to_string(),
                    style: Style::default(),
                    kind: Kind::Input(String::new()),
                })
            ],
            [grid![
                [
                    g!(Grammar::input("rule_name", "")),
                    g!(Grammar::input("rule_grammar", ""))
                ],
                [
                    g!(Grammar::input("rule_name", "")),
                    g!(Grammar::input("rule_grammar", ""))
                ]
            ]]
        ],
    );
}

pub fn resize(m: &mut Model, coord: Coordinate, row_height: f64, col_width: f64) {
    if let Some(parent_coord) = coord.parent() {
        let mut row_height_diff = 0.0;
        let mut col_width_diff = 0.0;
        let mut new_row_height = 0.0;
        let mut new_col_width = 0.0;
        let mut new_grammar = Grammar::default();
        if let Some(old_row_height) = m.row_heights.get_mut(&coord.full_row()) {
            if row_height != *old_row_height {
                // In case for the addnested row is different with the old one
                new_row_height = row_height + /* horizontal border width */ 2.0;
            } else {
                new_row_height = row_height;
            }
            row_height_diff = new_row_height - *old_row_height;
            *old_row_height = new_row_height;
        }
        if let Some(old_col_width) = m.col_widths.get_mut(&coord.full_col()) {
            if col_width != *old_col_width {
                // In case for the addnested col is different with the old one
                new_col_width = col_width + /* vertiacl border height */ 2.0;
            } else {
                new_col_width = col_width;
            }
            col_width_diff = new_col_width - *old_col_width;
            *old_col_width = new_col_width;
        }

        /* Update style width and height for the resize coord and neighbor with same column or row
            Also update new size for its parent coord and associate neighbor.
        */
        let mut current_coord = coord.clone();
        let mut get_grammar = m.get_session_mut().grammars.clone();
        while !current_coord.parent().is_none() {
            let p_coord = current_coord.parent().clone();
            for (c, g) in m.get_session_mut().grammars.iter_mut() {
                if c.parent() == p_coord {
                    if c.row().get() == current_coord.row().get() {
                        g.style.height = new_row_height;
                    }
                    if c.col().get() == current_coord.col().get() {
                        g.style.width = new_col_width;
                    }
                }
            }
            if let Some(parent_grammar) = get_grammar.get_mut(&p_coord.clone().unwrap()) {
                new_row_height = parent_grammar.style.height + (2 * 32) as f64;
                new_col_width = parent_grammar.style.width + (2 * 92) as f64;
            }
            current_coord = p_coord.unwrap();
        }
        info! {"resizing cell: (row: {}, col: {}); height: {}, width: {}", coord.row_to_string(), coord.col_to_string(),  row_height_diff, col_width_diff};
        resize_diff(m, parent_coord, row_height_diff, col_width_diff);
    }
}

pub fn resize_diff(m: &mut Model, coord: Coordinate, row_height_diff: f64, col_width_diff: f64) {
    let additional_offset = if m.resizing.is_none() {
        2.0 /* if not resizing, account for internal borders width */
    } else {
        0.0
    };
    if let Some(parent_coord) = coord.parent() {
        if let Some(row_height) = m.row_heights.get_mut(&coord.full_row()) {
            *row_height += row_height_diff + additional_offset;
        }
        if let Some(col_width) = m.col_widths.get_mut(&coord.full_col()) {
            *col_width += col_width_diff + additional_offset;
        }
        resize_diff(m, parent_coord, row_height_diff, col_width_diff);
    }
}

// Use width and height values from DOM to resize element
pub fn dom_resize(m: &mut Model, on: Coordinate) {
    let (height, width) = {
        let element = HtmlElement::try_from(
            document()
                .get_element_by_id(format! {"cell-{}", on.to_string()}.deref())
                .unwrap(),
        )
        .unwrap();
        let rect = element.get_bounding_client_rect();
        (rect.get_height(), rect.get_width())
    };
    info! {"expanding...: H {}px, W {}px", height.clone(), width.clone()}
    resize(m, on, height, width);
    /*
    let on_grammar = map.get_mut(&on).unwrap();
    on_grammar.style.height = height.clone();
    on_grammar.style.width = width.clone();
    let parent_kind = on
        .parent()
        .and_then(|p| map.get(&p))
        .map(|g| g.kind.clone());
    if let Some(Kind::Grid(neighbors_coords)) = parent_kind {
        for (row, col) in neighbors_coords {
            if let Some(cell) = map.get_mut(&Coordinate::child_of(&on, (row.clone(), col.clone())))
            {
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

/* TODO: get this working so w can color code lookups */
mod tests {
    use super::*;

    #[test]
    fn test_non_zero_u32_tuple() {
        assert_eq!(
            non_zero_u32_tuple((1, 2)),
            (NonZeroU32::new(1).unwrap(), NonZeroU32::new(2).unwrap())
        );
        assert_ne!(
            non_zero_u32_tuple((1, 2)),
            (NonZeroU32::new(2).unwrap(), NonZeroU32::new(2).unwrap())
        );
    }

    #[test]
    fn test_row_col_to_string() {
        assert_eq!(row_col_to_string((2, 2)), "B2");
        assert_ne!(row_col_to_string((2, 2)), "A2");
    }

    #[test]
    fn test_coord_show() {
        assert_eq!(coord_show(vec![(1, 1), (1, 1)]).unwrap(), "root-A1");
        assert_ne!(coord_show(vec![(1, 1), (1, 1)]).unwrap(), "root")
    }
}
