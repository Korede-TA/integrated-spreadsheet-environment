#![recursion_limit = "512"]

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::char::from_u32;
use std::ops::Deref;
use std::iter::FromIterator;
use std::cmp::Ordering;
use std::option::Option;
use serde::{Serialize, Deserialize};
use yew::{html, ChangeData, Component, ComponentLink, Html, ShouldRender, InputData};
use yew::callback::Callback;
use yew::events::{IKeyboardEvent, ClickEvent, KeyPressEvent};
use yew::services::{ConsoleService};
use yew::services::reader::{File, FileChunk, FileData, ReaderService, ReaderTask};
use yew::virtual_dom::{VList};
use wasm_bindgen::prelude::*;
use stdweb::Value;
use stdweb::web::{document, Element, HtmlElement, IHtmlElement, INonElementParentNode};
use stdweb::unstable::TryFrom;
use log::trace;
use itertools::Itertools;
use pest::Parser;

//use dialog::DialogBox;
//use nfd::Response;
use std::fs::File as stdFile;
use std::fs;

extern crate console_error_panic_hook;
use std::panic;
extern crate web_logger;
extern crate pest;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate stdweb;
#[macro_use] extern crate pest_derive;

/*
 * DATA MODEL:
 * is centered around the "grammars" map: HashMap<Coordinate, Grammar>
 * this is a linear-time accessible directory of every grammar in the system
 * as indexed by the grammar coordinate
 *
 */

/*
 * # Other Notes:
 *
 * Enums vs Structs: 
 * Structs are just a basic collection of fields like in a class.
 * Enums are used to represent a value that can take multiple forms.
 * For instance, 
 *
 * `#[derive()]`:
 * These is a macro provided in the Rust standard library for generating code 
 * to automatically implement certain traits (interfaces) in Rust
 *
 * NonZeroU32:
 * In a number of places in the application, we make use of integers that can be neither
 * negative (unsigned) nor zero, such as the coordinate values. We adapt the standard rust 
 * data type NonZeroU32 (non-zero unsigned 32-bit integer) as a type for such values
 */

// Style contains the relevant CSS properties for styling
// a grammar Cell or Grid
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Style {
    width: f64,  // CSS: width
    height: f64, // CSS: height
    border_color: String,  // CSS: border-color
    border_collapse: bool, // CSS: border-collapse
    font_weight: i32,      // CSS: font-weight
    font_color: String,    // CSS: font-color
}
js_serializable!( Style );
js_deserializable!( Style );

impl Style {
    fn default() -> Style {
        Style {
            width: 90.00,
            height: 30.00,
            border_color: "grey".to_string(),
            border_collapse: false,
            font_weight: 400,
            font_color: "black".to_string(),
        }
    }

    fn to_string(&self) -> String {
        format!{
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Interactive {
    Button(),
    Slider(/*value*/ f64, /*min*/ f64, /*max*/ f64),
    Toggle(bool),
}

// Kinds of grammars in the system.
// Since this is an Enum, a Grammar's kind field
// can only be set to one these variants at a time
#[derive(Serialize, Deserialize, Debug, Clone)]
enum Kind {
    Text(String),
    Input(String),
    Interactive(String, Interactive),
    Grid(Vec<(NonZeroU32, NonZeroU32)>),
}
js_serializable!( Kind );
js_deserializable!( Kind );

// Grammar is the main data-type representing
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Grammar {
    name: String,
    style: Style,
    kind: Kind,
}
js_serializable!( Grammar );
js_deserializable!( Grammar );

impl Grammar {
    fn default() -> Grammar {
        Grammar {
            name: "".to_string(),
            style: Style::default(),
            kind: Kind::Input("".to_string()),
        }
    }

    fn style(&self, coord: &Coordinate) -> String {
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

    fn suggestion(alias : String, value: String) -> Grammar {
        Grammar {
            name: alias,
            style: Style::default(),
            kind: Kind::Text(value),
        }
    }

    fn as_grid(rows: NonZeroU32, cols: NonZeroU32) -> Grammar {
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

fn move_grammar(map: &mut HashMap<Coordinate, Grammar>, source: Coordinate, dest: Coordinate) {
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

// Session encapsulates the serializable state of the application that gets stored to disk
// in a .ise file (which is just a JSON file)
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Session {
    root: Grammar,
    meta: Grammar,
    grammars: HashMap<Coordinate, Grammar>,
}
js_serializable!( Session );
js_deserializable!( Session );

// Model contains the entire state of the application
#[derive(Debug)]
struct Model {
    // model holds a direct reference to the topmost root A1 and meta A2 grammars
    // these two grammars are excluded from the grammar Map
    root: Grammar,
    meta: Grammar,

    // the view that the UI treats as the topmost grammar to start rendering from.
    view_root: Coordinate,

    grammars: HashMap</*Key*/ Coordinate, /*Value*/ Grammar>,
    value: String,
    active_cell: Option<Coordinate>,
    suggestions: Vec<Coordinate>,

    col_widths: HashMap<Col, f64>,
    row_heights: HashMap<Row, f64>,

    // tabs correspond to sessions
    tabs: Vec<String>,
    current_tab: i32,

    // side menus
    side_menus: Vec<SideMenu>,
    open_side_menu: Option<i32>,

    console: ConsoleService,
    reader: ReaderService,

    link: ComponentLink<Model>,
    tasks: Vec<ReaderTask>,
}

impl Model {
    fn load_session(&mut self, session: Session) {
        self.root = session.root;
        self.meta = session.meta;
        self.grammars = session.grammars;
    }

    fn to_session(&self) -> Session {
        Session {
            root: self.root.clone(),
            meta: self.meta.clone(),
            grammars: self.grammars.clone(),
        }
    }

    fn get_style(&self, coord: &Coordinate) -> String {
        let grammar = self.grammars.get(coord).expect("no grammar with this coordinate");
        if coord.row_cols.len() == 1 {  // root or meta
            return grammar.style(coord);
        }
        if let Kind::Grid(_) = grammar.kind {
            return format!{
                "{}\nwidth: fit-content;\nheight: fit-content;\n",
                grammar.style(coord),
            };
        }
        let col_width = self.col_widths.get(&coord.full_col()).unwrap_or(&90.0);
        let row_height = self.row_heights.get(&coord.full_row()).unwrap_or(&30.0);
        format!{
            "{}\nwidth: {}px;\nheight: {}px;\n",
            grammar.style(coord), col_width, row_height,
        }
    }

    fn query_parent(&self, coord_parent: Coordinate) -> Vec<Coordinate> {
        self.grammars.keys().clone().filter_map(|k| {
            if k.parent() == Some(coord_parent.clone()) {
                Some(k.clone())
            } else { None }
        }).collect()
    }

    fn query_col(&self, coord_col: Col) -> Vec<Coordinate> {
        self.grammars.keys().clone().filter_map(|k| {
            if k.row_cols.len() == 1 /* ignore root & meta */ {
                None
            } else if k.full_col() == coord_col {
                Some(k.clone())
            } else { None }
        }).collect()
    }

    fn query_row(&self, coord_row: Row) -> Vec<Coordinate> {
        self.grammars.keys().clone().filter_map(|k| {
            if k.row_cols.len() == 1 /* ignore root & meta */ {
                None
            } else if k.full_row() == coord_row {
                Some(k.clone())
            } else { None }
        }).collect()
    }
}

#[derive(Debug)]
struct SideMenu {
    name: String,
    icon_path: String,
}

// Coordinate specifies the nested coordinate structure
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
struct Coordinate {
    row_cols: Vec<(NonZeroU32, NonZeroU32)>, // should never be empty list
}
js_serializable!( Coordinate );
js_deserializable!( Coordinate );

fn non_zero_u32_tuple(val: (u32, u32)) -> (NonZeroU32, NonZeroU32) {
    let (row, col) = val;
    (NonZeroU32::new(row).unwrap(), NonZeroU32::new(col).unwrap())
}

// macro for easily defining a vector of non-zero tuples
// used in Coordinate::root() below
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

#[derive(Parser)]
#[grammar = "coordinate.pest"]
struct CoordinateParser;

// macro for easily defining a coordinate
// either absolutely or relative to it's parent coordinate
// TODO: this code is messy, can be optimized more later 
macro_rules! coord {
    ( $coord_str:tt ) => {
        {

            let mut fragments: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();

            let pairs = CoordinateParser::parse(Rule::coordinate, $coord_str).unwrap_or_else(|e| panic!("{}", e));

            for pair in pairs {
                match pair.as_rule() {
                    Rule::special if pair.as_str() == "root" => {
                        fragments.push(non_zero_u32_tuple((1, 1)));
                    }
                    Rule::special if pair.as_str() == "meta" => {
                        fragments.push(non_zero_u32_tuple((1, 2)));
                    }
                    Rule::fragment => {
                        let mut fragment: (u32, u32) = (0,0);
                        for inner_pair in pair.into_inner() {
                            match inner_pair.as_rule() {
                                // COLUMN
                                Rule::alpha => {
                                    let mut val: u32 = 0;
                                    for ch in inner_pair.as_str().to_string().chars() {
                                        val += (ch as u32) - 64;
                                    }
                                    fragment.1 = val;
                                }
                                // ROW
                                Rule::digit => {
                                    fragment.0 = inner_pair.as_str().parse::<u32>().unwrap();
                                }
                                _ => unreachable!()
                            };
                        }
                        fragments.push(non_zero_u32_tuple(fragment));
                    }
                    _ => unreachable!()
                }
            }

            Coordinate {
                row_cols: fragments,
            }
        }
    };

}

macro_rules! coord_col {
    ( $parent_str:tt, $col_str:tt ) => {
        {
            let mut col: u32 = 0;
            for ch in $col_str.to_string().chars() {
                col += (ch as u32) - 64;
            }

            Col(coord!($parent_str), NonZeroU32::new(col).unwrap())
        }
    };
}

macro_rules! coord_row {
    ( $parent_str:tt, $row_str:tt ) => {
        {
            let row: u32 = $row_str.parse::<u32>().unwrap();

            Row(coord!($parent_str), NonZeroU32::new(row).unwrap())
        }
    };
}

fn row_col_to_string((row, col): (u32, u32)) -> String {
    let row_str = row.to_string();
    let col_str = from_u32(col + 64).unwrap();
    format!{"{}{}", col_str, row_str} 
}

fn coord_show(row_cols: Vec<(u32, u32)>) -> Option<String> {
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

#[derive(Debug, Clone, Hash)]
struct Row(/* parent */ Coordinate, /* row_index */ NonZeroU32);

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Row {}

#[derive(Debug, Clone, Hash)]
struct Col(/* parent */ Coordinate, /* col_index */ NonZeroU32);

impl PartialEq for Col {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Col {}

// Methods for interacting with coordinate struct
impl Coordinate {
    fn child_of(parent: &Self, child_coord: (NonZeroU32, NonZeroU32)) -> Coordinate {
        let mut new_row_col = parent.clone().row_cols;
        new_row_col.push(child_coord);
        Coordinate{ row_cols: new_row_col }
    }

    fn parent(&self) -> Option<Coordinate> {
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

    fn to_string(&self) -> String {
        coord_show(self.row_cols.iter()
             .map(|(r, c)| (r.get(), c.get())).collect()).unwrap()
    }

    fn row(&self) -> NonZeroU32 {
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

    fn full_row(&self) -> Row {
        Row(self.parent().expect("full_row shouldn't be called on root or meta"), self.row())
    }

    fn row_to_string(&self) -> String {
        if let Some(parent) = self.parent() {
            format!{"{}-{}", parent.to_string(), self.row().get()}
        } else {
            format!{"{}", self.row().get()}
        }
    }

    fn col(&self) -> NonZeroU32 {
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

    fn full_col(&self) -> Col {
        Col(self.parent().expect("full_col shouldn't be called on root or meta"), self.col())
    }

    fn col_to_string(&self) -> String {
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

    fn neighbor_above(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            if last.0.get() > 1 {
                *last = (/* row */ NonZeroU32::new(last.0.get() - 1).unwrap(), /* column */ last.1);
                return Some(Coordinate { row_cols: new_row_col })
            }
        }

        None
    }

    fn neighbor_below(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            *last = (/* row */ NonZeroU32::new(last.0.get() + 1).unwrap(), /* column */ last.1);
            return Some(Coordinate { row_cols: new_row_col })
        }

        None
    }

    fn neighbor_left(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            if last.1.get() > 1 {
                *last = (/* row */ last.0, /* column */ NonZeroU32::new(last.1.get() - 1).unwrap());
                return Some(Coordinate { row_cols: new_row_col })
            }
        }

        None
    }

    fn neighbor_right(&self) -> Option<Coordinate> {
        let mut new_row_col = self.clone().row_cols;
        
        if let Some(last) = new_row_col.last_mut() {
            *last = (/* row */ last.0, /* column */ NonZeroU32::new(last.1.get() + 1).unwrap());
            return Some(Coordinate { row_cols: new_row_col })
        }

        None
    }

}

// ACTIONS
// Triggered in the view, sent to update function
enum Action {
    // Do nothing
    Noop,

    // Change string value of Input grammar
    ChangeInput(Coordinate, /* new_value: */ String),

    // Show suggestions dropdown at Coordinate based on query
    ShowSuggestions(Coordinate, /* query: */ String),

    SetActiveCell(Coordinate),

    DoCompletion(/* source: */ Coordinate, /* destination */ Coordinate),

    SetActiveMenu(Option<i32>),

    ReadSession(/* filename: */ File),

    LoadSession(FileData),

    SaveSession,

    // Grid Operations
    AddNestedGrid(Coordinate, (u32 /*rows*/, u32 /*cols*/)),

    InsertRow,
    InsertCol,

    // Alerts and stuff
    Alert(String),
}

fn apply_definition_grammar(m: &mut Model, root_coord: Coordinate) {
    // definition grammar contains the name of the grammar and then the list of
    // different parts of the grammar
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

    m.grammars.insert(root_coord, defn);
    m.grammars.insert(defn_name_coord, defn_name);
    m.grammars.insert(defn_label_coord, defn_label);
    m.grammars.insert(defn_body_coord, defn_body);
    m.grammars.insert(defn_body_A1_coord, defn_body_A1);
    m.grammars.insert(defn_body_A2_coord, defn_body_A2);
    m.grammars.insert(defn_body_B1_coord, defn_body_B1);
    m.grammars.insert(defn_body_B2_coord, defn_body_B2);
}

fn resize(m: &mut Model, coord: Coordinate, row_height: f64, col_width: f64) {
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

fn resize_diff(m: &mut Model, coord: Coordinate, row_height_diff: f64, col_width_diff: f64) {
    if let Some(parent_coord) = coord.parent() {
        if let Some(row_height) = m.row_heights.get_mut(&coord.full_row()) {
            *row_height = row_height_diff + /* horizontal border width */ 2.0; 
        }
        if let Some(col_width) = m.col_widths.get_mut(&coord.full_col()) {
            *col_width = col_width_diff + /* vertical border height */ 2.0;
        }
        resize_diff(m, parent_coord, row_height_diff, col_width_diff);
    }
}


// when a cell is expanded, grow cells in the same row/column as well
fn resize_cells(map: &mut HashMap<Coordinate, Grammar>, on: Coordinate) {
    let (height, width) = {
        let element = HtmlElement::try_from(
            document().get_element_by_id(format!{"cell-{}", on.to_string()}.deref()).unwrap()).unwrap();
        let rect = element.get_bounding_client_rect();
        (rect.get_height(), rect.get_width())
    };
    info!{"expanding...: H {}px, W {}px", height.clone(), width.clone()}
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
}


impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let root_grammar = Grammar {
            name: "root".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (2,1), (3,1), (1,2), (2,2), (3,2) ]),
        };
        let meta_grammar = Grammar {
            name: "meta".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (2,1) ]),
        };
        let mut m = Model {
            root: root_grammar.clone(),
            meta: meta_grammar.clone(),
            view_root: coord!("root"),
            grammars: hashmap! {
                coord!("root")    => root_grammar.clone(),
                coord!("root-A1") => Grammar::default(),
                coord!("root-A2") => Grammar::default(),
                coord!("root-A3") => Grammar::default(),
                coord!("root-B1") => Grammar::default(),
                coord!("root-B2") => Grammar::default(),
                coord!("root-B3") => Grammar::default(),
                coord!("meta")    => meta_grammar.clone(),
                coord!("meta-A1") => Grammar::suggestion("js grammar".to_string(), "This is js".to_string()),
                coord!("meta-A2") => Grammar::suggestion("java grammar".to_string(), "This is java".to_string()),
            },
            col_widths: hashmap! {
               coord_col!("root","A") => 90.0,
               coord_col!("root","B") => 90.0,
            },
            row_heights: hashmap! {
               coord_row!("root","1") => 30.0,
               coord_row!("root","2") => 30.0,
               coord_row!("root","3") => 30.0,
            },
            value: String::new(),
            active_cell: Some(coord!("root-A1")),
            suggestions: vec![ coord!("meta-A1"), coord!("meta-A2"), coord!("meta-A3") ],
            // suggestions: vec![],

            console: ConsoleService::new(),
            reader: ReaderService::new(),

            tabs: vec![
               "Session 1".to_string(),
               "My Session".to_string(),
               "Session 100".to_string(),
            ],

            current_tab: 0,

            side_menus: vec![
                SideMenu {
                    name: "Home".to_string(),
                    icon_path: "assets/logo.png".to_string(),
                },
                SideMenu {
                    name: "File Explorer".to_string(),
                    icon_path: "assets/folder_icon.png".to_string(),
                },
                SideMenu {
                    name: "Settings".to_string(),
                    icon_path: "assets/settings_icon.png".to_string(),
                },
                SideMenu {
                    name: "Info".to_string(),
                    icon_path: "assets/info_icon.png".to_string(),
                },
            ],
            open_side_menu: None,

            link,
            tasks: vec![],
        };
        apply_definition_grammar(&mut m, coord!("meta-A3"));
        m
    }

    // The update function is split into sub-update functions that 
    // are specifc to each EventType
    fn update(&mut self, event_type: Self::Message) -> ShouldRender {
        match event_type {
            Action::Noop => false,

            Action::Alert(message) => {
                self.console.log(&message);
                // TODO: make this into a more visual thing
                false
            }

            Action::ChangeInput(coord, new_value) => {
                let old_grammar = self.grammars.get_mut(&coord);
                match old_grammar {
                    Some(g @ Grammar { kind: Kind::Text(_), .. }) => {
                        self.console.log(&new_value);
                        g.kind = Kind::Text(new_value);
                    },
                    _ => ()
                }
                false
            }

            Action::ShowSuggestions(coord, query) => {
                false
            }

            Action::SetActiveCell(coord) => {
                self.active_cell = Some(coord);
                true
            }

            Action::DoCompletion(source_coord, dest_coord) => {
                move_grammar(&mut self.grammars, source_coord, dest_coord.clone());
                resize_cells(&mut self.grammars, dest_coord);
                true
            }

            Action::SetActiveMenu(active_menu) => {
                self.open_side_menu = active_menu;
                true
            }

            Action::ReadSession(file) => {
                let callback = self.link.callback(Action::LoadSession);
                let task = self.reader.read_file(file, callback);
                self.tasks.push(task);
                false
            }

            Action::LoadSession(file_data) => {
                let session : Session = serde_json::from_str(format!{"{:?}", file_data}.deref()).unwrap();
                self.load_session(session);
                true
            }

            Action::SaveSession => {
                let _session : Session = self.to_session();
                // TODO: Setup saving to session
                false
            }

            Action::AddNestedGrid(coord, (rows, cols)) => {
                let (r, c) = non_zero_u32_tuple((rows, cols));
                let grammar = Grammar::as_grid(r, c);
                if let Kind::Grid(sub_coords) = grammar.clone().kind {
                    self.active_cell = sub_coords.first().map(|c| Coordinate::child_of(&coord, *c));
                    for sub_coord in sub_coords {
                        let new_coord = Coordinate::child_of(&coord, sub_coord);
                        self.grammars.insert(new_coord.clone(), Grammar::default());
                        // initialize row & col heights as well
                        if !self.row_heights.contains_key(&new_coord.clone().full_row()) {
                            self.row_heights.insert(new_coord.clone().full_row(), 30.0);
                        }
                        if !self.col_widths.contains_key(&new_coord.clone().full_col()) {
                            self.col_widths.insert(new_coord.clone().full_col(), 90.0);
                        }
                    }
                }
                if let Some(parent) = Coordinate::parent(&coord).and_then(|p| self.grammars.get_mut(&p)) {
                    parent.kind = grammar.clone().kind; // make sure the parent gets set to Kind::Grid
                }
                self.grammars.insert(coord.clone(), grammar);
                resize(self, coord,
                    (rows as f64) * (/* default row height */ 30.0),
                    (cols as f64) * (/* default col width */ 90.0));
                true
            }
            Action::InsertCol => {
                if let Some(coord) = self.active_cell.clone() {
                    // find the bottom-most coord
                    let mut right_most_coord = coord.clone();
                    while let Some(right_coord) = right_most_coord.neighbor_right() {
                        if self.grammars.contains_key(&right_coord) {
                            right_most_coord = right_coord;
                        } else { break }
                    }

                    let right_most_col_coords = self.query_col(right_most_coord.full_col());
                    let new_col_coords = right_most_col_coords.iter().map(|c| {
                        (c.row(), NonZeroU32::new(c.col().get() + 1).unwrap())
                    });

                    let parent = coord.parent().unwrap();
                    if let Some(Grammar{ kind: Kind::Grid(sub_coords), name, style }) = self.grammars.get(&parent) {
                        let mut new_sub_coords = sub_coords.clone();
                        let mut grammars = self.grammars.clone();
                        for c in new_col_coords {
                            grammars.insert(Coordinate::child_of(&parent.clone(), c), Grammar::default());
                            new_sub_coords.push(c);
                        }
                        grammars.insert(parent, Grammar {
                            kind: Kind::Grid(new_sub_coords.clone()),
                            name: name.clone(),
                            style: style.clone()
                        });
                        self.grammars = grammars;
                    }
                }
                true
            }
            Action::InsertRow => {
                if let Some(coord) = self.active_cell.clone() {
                    // find the bottom-most coord
                    let mut bottom_most_coord = coord.clone();
                    while let Some(below_coord) = bottom_most_coord.neighbor_below() {
                        if self.grammars.contains_key(&below_coord) {
                            bottom_most_coord = below_coord;
                        } else { break }
                    }

                    let bottom_most_row_coords = self.query_row(bottom_most_coord.full_row());
                    let new_row_coords = bottom_most_row_coords.iter().map(|c| {
                        (NonZeroU32::new(c.row().get() + 1).unwrap(), c.col())
                    });

                    let parent = coord.parent().unwrap();
                    if let Some(Grammar{ kind: Kind::Grid(sub_coords), name, style }) = self.grammars.get(&parent) {
                        let mut new_sub_coords = sub_coords.clone();
                        let mut grammars = self.grammars.clone();
                        for c in new_row_coords {
                            grammars.insert(Coordinate::child_of(&parent.clone(), c), Grammar::default());
                            new_sub_coords.push(c);
                        }
                        grammars.insert(parent, Grammar {
                            kind: Kind::Grid(new_sub_coords.clone()),
                            name: name.clone(),
                            style: style.clone()
                        });
                        self.grammars = grammars;
                    }
                }
                true
            }
        }
    }

    fn view(&self) -> Html {

        let active_cell = self.active_cell.clone();
        html! {
            <div>

                { view_side_nav(&self) }

                { view_menu_bar(&self) }

                { view_tab_bar(&self) }

                <div class="main">
                    <div id="grammars" class="grid-wrapper" onkeypress=self.link.callback(move |e : KeyPressEvent| {
                        if e.key() == "g" && e.ctrl_key() {
                            if let Some(coord) = active_cell.clone() {
                                return Action::AddNestedGrid(coord.clone(), (3, 3));
                            }
                        }
                        Action::Noop
                    })>
                        { view_grammar(&self, coord!{"root"}) }
                    </div>
                </div>
            </div>
        }
    }
}

fn view_side_nav(m: &Model) -> Html {
    let mut side_menu_nodes = VList::new();
    let mut side_menu_section = html! { <></> };
    for (index, side_menu) in m.side_menus.iter().enumerate() {
        if Some(index as i32) == m.open_side_menu {
            side_menu_nodes.add_child(html! {
                <button class="active-menu" onclick=m.link.callback(|e| Action::SetActiveMenu(None))>
                    <img 
                        src={side_menu.icon_path.clone()} 
                        width="40px" alt={side_menu.name.clone()}>
                    </img>
                </button>
            });

            side_menu_section = view_side_menu(m, side_menu);
        } else {
            side_menu_nodes.add_child(html! {
                <button onclick=m.link.callback(move |e| Action::SetActiveMenu(Some(index as i32)))>
                    <img 
                        src={side_menu.icon_path.clone()} 
                        width="40px" alt={side_menu.name.clone()}>
                    </img>
                </button>
            });
        }
    }

    html! {
        <div class="sidenav">
            { side_menu_nodes }

            { side_menu_section }
        </div>
    }
}

fn view_side_menu(m: &Model, side_menu: &SideMenu) -> Html {
    match side_menu.name.deref() {
        "Home" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS Home MENU"}
                </div>
            } 
        },
        "File Explorer" => {
            html! {
                <div class="side-menu-section">
                    <h1>
                        {"File Explorer"}
                    </h1>

                    <h3>{"load session"}</h3>
                    <br></br>
                    <input type="file" onchange=m.link.callback(|value| {
                        if let ChangeData::Files(files) = value {
                            if files.len() >= 1 {
                                if let Some(file) = files.iter().nth(0) {
                                    return Action::ReadSession(file);
                                }
                            } else {
                                return Action::Alert("Could not load file".to_string());
                            }
                        }
                        Action::Noop
                    })>
                    </input>

                    <h3>{"save session"}</h3>
                    <br></br>
                    <input type="file" onchange=m.link.callback(|value| {
                        if let ChangeData::Files(files) = value {
                            if files.len() >= 1 {
                                if let Some(file) = files.iter().nth(0) {
                                    return Action::ReadSession(file);
                                }
                            }
                        }
                        Action::Noop
                    })>
                        
                    </input>
                </div>
            } 
        },
        "Settings" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS Settings MENU"}
                </div>
            } 
        },
        "Info" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS info MENU"}
                </div>
            } 
        },

        _ => html! {<> </>}

    }
}


fn view_menu_bar(m: &Model) -> Html {
    html! {
        <div class="menu-bar horizontal-bar">
            <input 
                class="active-cell-indicator"
                disabled=true 
                // TODO: clicking on this should highlight
                // the active cell
                value={
                    if let Some(cell) = m.active_cell.clone() {
                        cell.to_string()
                    } else {
                        "".to_string()
                    }
                }>
            </input>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::SaveSession) >
                { "Save" }
            </button>
            <button class="menu-bar-button">
                { "Git" }
            </button>
            <button class="menu-bar-button">
                { "Zoom In (+)" }
            </button>
            <button class="menu-bar-button">
                { "Zoom Out (-)" }
            </button>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::InsertRow)>
                { "Insert Row" }
            </button>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::InsertCol)>
                { "Insert Column" }
            </button>
            <button class="menu-bar-button">
                { "Delete Row" }
            </button>
            <button class="menu-bar-button">
                { "Delete Column" }
            </button>
        </div>
    }
}

fn view_tab_bar(m: &Model) -> Html {
    let mut tabs = VList::new();
    for (index, tab) in m.tabs.clone().iter().enumerate() {
        if (index as i32) == m.current_tab {
            tabs.add_child(html! {
                <button class="tab active-tab">{ tab }</button>
            });
        } else {
            tabs.add_child(html! {
                <button class="tab">{ tab }</button>
            });
        }
    }
    html! {
        <div class="tab-bar horizontal-bar">
            { tabs }
            <button class="newtab-btn">
                <span>{ "+" }</span>
            </button>
        </div>
    }
}

fn view_grammar(m: &Model, coord: Coordinate) -> Html {
    if let Some(grammar) = m.grammars.get(&coord) {
        match grammar.kind.clone() {
            Kind::Text(value) => {
                view_text_grammar(m, &coord, value)
            }
            Kind::Input(value) => {
                let is_active = m.active_cell.clone() == Some(coord.clone());
                let suggestions = m.suggestions.iter().filter_map(|suggestion_coord| {
                    if let Some(suggestion_grammar) = m.grammars.get(&suggestion_coord) {
                        Some((suggestion_coord.clone(), suggestion_grammar.clone()))
                    } else {
                        None
                    }
                }).collect();
                view_input_grammar(
                    m,
                    coord.clone(),
                    suggestions,
                    value,
                    is_active,
                )
            }
            Kind::Interactive(name, Interactive::Button()) => {
                html! {
                    <div
                        class=format!{"cell row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ m.get_style(&coord) }>
                        <button>
                            { name }
                        </button>
                    </div>
                }
            }
            Kind::Interactive(name, Interactive::Slider(value, min, max)) => {
                html! {
                    <div 
                        class=format!{"cell row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ m.get_style(&coord) }>
                        <input type="range" min={min} max={max} value={value}>
                            { name }
                        </input>
                    </div>
                }
            }
            Kind::Interactive(name, Interactive::Toggle(checked)) => {
                html! {
                    <div
                        class=format!{"cell row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ m.get_style(&coord) }>
                        <input type="checkbox" checked={checked}>
                            { name }
                        </input>
                    </div>
                }
            }
            Kind::Grid(sub_coords) => {
                view_grid_grammar(
                    m,
                    &coord,
                    sub_coords.iter().map(|c| Coordinate::child_of(&coord, *c)).collect(),
                )
            }
        }
    } else {
        // return empty fragment
        html! { <></> }
    }
}

fn view_input_grammar(
    m: &Model,
    coord: Coordinate,
    suggestions: Vec<(Coordinate, Grammar)>,
    value: String,
    is_active: bool,
) -> Html {
    let mut suggestion_nodes = VList::new();
    let mut active_cell_class = "cell-inactive";
    if is_active {
        active_cell_class = "cell-active";
        for (s_coord, s_grammar) in suggestions {
            let c = coord.clone();
            suggestion_nodes.add_child(html! {
                <a 
                    tabindex=-1
                    onclick=m.link.callback(move |_ : ClickEvent| Action::DoCompletion(s_coord.clone(), c.clone()))>
                    { &s_grammar.name }
                </a>
            })
            
        }
    }
    let suggestions = html!{
        <div class="suggestion-content">
            { suggestion_nodes }
        </div>
    };

    let new_active_cell = coord.clone();

    html! {
        <div
            class=format!{"cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ m.get_style(&coord) }>
            <input 
                class={ format!{ "cell-data {}", active_cell_class } }
                value=value
                oninput=m.link.callback(move |e : InputData| Action::ChangeInput(coord.clone(), e.value))
                onclick=m.link.callback(move |_ : ClickEvent| Action::SetActiveCell(new_active_cell.clone()))>
            </input>
            
            { suggestions }
        </div>
    }
}

fn view_text_grammar(m: &Model, coord: &Coordinate, value : String) -> Html {
    html! {
        <div
            class=format!{"cell text row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ m.get_style(&coord) }>
            { value }
        </div>
    }
}

fn view_grid_grammar(m: &Model, coord: &Coordinate, sub_coords: Vec<Coordinate>) -> Html {
    let mut nodes = VList::new();

    for c in sub_coords {
        nodes.add_child(view_grammar(m, c.clone()));
    }

    html! {
        <div
            class=format!{"cell grid row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ m.get_style(&coord) }>
            { nodes }
        </div>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
//    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    yew::start_app::<Model>();
    Ok(())
}
