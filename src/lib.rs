#![recursion_limit = "512"]

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator. (see Cargo.toml for why we use optimixed allocator)
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use yew::{html, ChangeData, Component, ComponentLink, Html, ShouldRender};
use yew::services::{ConsoleService};
use yew::services::reader::{File, FileChunk, FileData, ReaderService, ReaderTask};
use yew::virtual_dom::{VList};
use wasm_bindgen::prelude::*;
use stdweb::web::{document, Element, INode, IParentNode};
use log::trace;
use itertools::Itertools;

#[macro_use] extern crate maplit;
#[macro_use] extern crate stdweb;
#[macro_use] extern crate log;
extern crate web_logger;

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
            border_color: "black".to_string(),
            border_collapse: true,
            font_weight: 400,
            font_color: "black".to_string(),
        }
    }

    fn to_string(&self) -> String {
        // TODO: fill this out
        format!{
        "border: 1px solid {};
border-collapse: {};
font-weight: {};
color: {};\n",
        self.border_color,
        self.border_collapse,
        self.font_weight,
        self.font_color,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Interactive {
    Button,
    Slider(f64),
    Toggle(bool),
}

// Kinds of grammars in the system.
// Since this is an Enum, a Grammar's kind field
// can only be set to one these variants at a time
#[derive(Serialize, Deserialize, Debug, Clone)]
enum Kind {
    Text(String),
    Input(String),
    Interactive(Interactive),
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
            kind: Kind::Input("default input val".to_string()),
        }
    }

    fn style(&self, coord: &Coordinate) -> String {
        format!{"{}\ngrid-area: cell-{};\n", self.style.to_string(), coord.to_string()}
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
        for i in 1..rows.get() {
            for j in 1..cols.get() {
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
    console: ConsoleService,
    reader: ReaderService,

    // tabs correspond to sessions
    tabs: Vec<String>,
    current_tab: i32,

    // side menus
    side_menus: Vec<SideMenu>,
    open_side_menu: Option<i32>,

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
}

struct SideMenu {
    name: String,
    icon_path: String,
}

// Since Gridlines are based on sequences of coordinate Row or Column
// we define a LineSeq type to track a list of non-zero unsigned integers (32-bit)
// and 
#[derive(Debug, Clone)]
struct LineSeq { 
    lines: Vec<NonZeroU32>,
    line_type: GridLineType,
}

impl LineSeq {
    fn from_coord(coord: &Coordinate, line_type: GridLineType) -> LineSeq {
        let mut seq = LineSeq {
            lines: Vec::new(),
            line_type: line_type.clone(),
        };

        for item in coord.row_cols.iter() {
            match line_type {
                GridLineType::Column => {
                    seq.lines.push(item.1);
                },
                GridLineType::Row => {
                    seq.lines.push(item.0);
                },
            };
        };

        seq
    }

    fn to_string(&self) -> String {
        let mut output = match self.line_type {
            GridLineType::Column => "grid-col".to_string(),
            GridLineType::Row => "grid-col".to_string(),
        };
        for line in self.lines.clone() {
            output += "-";
            output += line.to_string().deref();
        }
        output
    }
}


// based on CSS grid template rows and columns
#[derive(PartialEq, Debug, Clone)]
struct GridLine {
    parent: Coordinate,
    line_type: GridLineType,
    index: NonZeroU32,
}

impl PartialEq for GridLine {
    fn eq(&self, other: &Self) -> bool {
        (self.parent.eq(other.parent))
        && (self.line_type == other.line_type)
        && (self.index == other.index)
    }
}

impl PartialOrd for GridLine {
    // Gridline::partial_cmp is downstream of LineTemplate::partial_cmp,
    // so it's only called partially as a helper for sorting the gridlines.
    // It's only used to determine:
    // 1. which gridlines are exactly equal (i.e. return Some(Ordering::Equal)
    // 2. which gridlines have incompatible line_types (i.e return None)
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // use GridLine::eq to check if lines are equal
            (line1, line2) if line1.eq(line2) => Some(Ordering::Equal)

            // lines with different line_types are totally incompatible
            (
                GridLine { line_type: lt1, .. }, 
                GridLine { line_type: lt2, .. }
            ) if lt1 != lt2 => None,

            // lines aligning sibling cells, are compared according to their index
            (
                GridLine { parent: parent1, index: index1, .. },
                GridLine { parent: parent2, index: index2, .. }
            ) if parent1.is_n_parent(parent) == Some(0) => {
                if index1 > index2 {
                    Some(Ordering::Greater)
                } else if index1 < index2 {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Equal)
                }
            },

            // default to sorting lines as greater, meaning they'll be sorted
            // towards the end of the gridline
            _ => Some(Ordering::Greater)
        }
    }
}


// based on CSS grid template rows and columns
#[derive(PartialEq, Eq, Debug, Clone)]
enum GridLineType {
    Row,
    Column,
}


// based on CSS grid template rows and columns
#[derive(Debug, Clone)]
enum LineTemplate {
    Start(GridLine),
    End(GridLine),
    Interval(/* start line */ GridLine, /* end line */ GridLine),
    Span(/* height/width in px */ f64)
}

impl PartialEq for LineTemplate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Start(start_line1),
                Self::Start(start_line2)
            ) => {
                start_line1.eq(start_line2)
            },
            (
                Self::Interval(start_line1, end_line1),
                Self::Interval(start_line2, end_line2)
            ) => {
                start_line1.eq(start_line2) && end_line1.eq(end_line2)
            },
            (
                Self::End(end_line1),
                Self::End(end_line2),
            ) => {
                end_line1.eq(end_line2)
            },
            _ => false
        }
    }
}

impl PartialOrd for LineTemplate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (
                Self::Start(GridLine { parent: parent1, .. }),
                Self::Start(GridLine { parent: parent2, .. })
            ) => if parent1.is_n_parent(parent) == Some(_) => {
                Some(Ordering::Less)
            },
            (
                Self::End(GridLine { parent: parent1, .. }),
                Self::End(GridLine { parent: parent2, .. })
            ) => if parent1.is_n_parent(parent) == None => {
                Some(Ordering::Less)
            },
            (
                Self::Start(start_line),
                Self::Interval(interval_start_line, interval_end_line)
            ) => if parent1.is_n_parent(parent) == None => {
                start_line.cmp(interval_start_line)
            },
            (
                Self::End(end_line),
                Self::Interval(interval_start_line, interval_end_line)
            ) => {
                
            },
            (
                Self::Interval(interval_start_line, interval_end_line),
                Self::Start(start_line)
            ) => {

            },
            (
                Self::Interval(interval_start_line, interval_end_line),
                Self::End(end_line)
            ) => {

            },
            _ => None
        }
    }
}

impl GridLine {
    fn from_coord(coord: &Coordinate, line_type: GridLineType) -> Self {
        let (parent, last) = {
            let mut temp = coord.clone();
            let last = temp.row_cols.pop();
            (temp, last.unwrap())
        };

        GridLine {
            parent: parent,
            line_type: line_type.clone(),
            index: match line_type {
                GridLineType::Column => last.1,
                GridLineType::Row => last.0,
            },
        }
    }

    fn to_string(&self) -> String {
        let (prefix, index_str) = match self.line_type {
            GridLineType::Column => ("col".to_string(), col_to_string(self.index.get() as i32)),
            GridLineType::Row => ("row".to_string(), (self.index).to_string()),
        };
        if self.parent.row_cols.len() == 0 { // in the case of the root coord
            "root".to_string()
        } else {
            format!{"{}-{}-{}", prefix, self.parent.to_string(), index_str}
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    template_rows: Vec<LineTemplate>,
    template_cols: Vec<LineTemplate>,
}

const MAX_GRID_DEPTH: i32 = 20;
const DEFAULT_CELL_WIDTH: f64 = 80.0;
const DEFAULT_CELL_HEIGHT: f64 = 30.0;

// we will utilize CSS grid's `grid-template-rows` and `grid-template-cols`
// properties to define the explicit gridlines for nested grid.
// (see: https://gridbyexample.com/examples/example10/)
//
// the individual grammar cells will use `grid-row-start`, `grid-row-end` and
// `grid-col-start` and `grid-col-end` to define the row and column gridlines where
// they start and stop.
fn compute_grid(
    template_rows: &mut Vec<LineTemplate>,
    template_cols: &mut Vec<LineTemplate>,
    coord: Coordinate,
    grammars: &HashMap<Coordinate, Grammar>,
    depth: i32,
) {
    // algorithm involves: 
    // - running through the list of template rows or columns to find an index to insert the
    // gridline, if necessary
    // - knowing if a paticular gridline should be a Start, End or Interval line
    // the first thing we're gonna do is create a function to find where in the current template
    // list to put a new gridline, or modify an existing gridline or interval

    if depth == MAX_GRID_DEPTH {
        return;
    }

    let grammar = grammars.get(&coord).unwrap();

    match &grammar.kind {
        Kind::Grid(sub_coords) => {
            let row_grid_index = find_gridline_indices(&coord, template_rows, GridLineType::Row);
            let col_grid_index = find_gridline_indices(&coord, template_cols, GridLineType::Column);
            for sub_coord in sub_coords.iter() {
                compute_grid(template_rows, template_cols, Coordinate::child_of(&coord, *sub_coord), grammars, depth+1);
            }
        },
        // simpler cases with non-grid (nested) grammars
        // use the catch-all for these
        _ => {
            
        }
    }
}

// the function needs to be able to look at any coordinate `c` and figure out if/where in the sequence
// of gridlines `lines` it's corresponding gridline (determined using GridLine::from_coord) should be.
// this function should be generalizable to either Row or Column grid linetypes using the
// GridLineType function
fn set_gridlines(c : &Coordinate, lines: &mut Vec<LineTemplate>, line_type: GridLineType) {
    // my initial attempt was to use the find function to find the first index that matches the
    // conditions i've specified inside, but we'll need something more robust
    //
    // the main conditions we need for determining the index:
    // (let's look at the diagram again to better understand this 😅
    // - if the coordinate already has gridlines for it, you can ignore (maybe this function should
    // be returning an Option<i32>)
    // - if the coordinate's parent already has gridlines, add it's start and end lines between the
    // parent's
    // - if the coordinate's left-side neighbor has gridlines, modify the End line of the neighbor to be an
    // Interval lines and add another End line after it
    // - if the coordinate's right-side neighbor has gridlines, modify the Start line of the
    // neigbhbor to be an interval and add a Start line before it

    // how do we find if a coordinate already has lines? use this function `gridline_indices`
    // for getting the index, as an optional.. None means it doesn't exist
    let gridline_indices = |c_| { 
        let mut start_index = None;
        let mut end_index = None;
        for (i, l) in lines.iter().enumerate() {
            match l {
                LineTemplate::Start(gl) => {
                    if *gl == GridLine::from_coord(c_, line_type) {
                        start_index = Some(i);
                    }
                },
                LineTemplate::End(gl) => {
                    if *gl == GridLine::from_coord(c_, line_type) {
                        end_index = Some(i)
                    }
                },
                LineTemplate::Interval(gl, _) => {
                    if *gl == GridLine::from_coord(c_, line_type) {
                        start_index = Some(i);
                    }
                },
                LineTemplate::Interval(_, gl) => {
                    if *gl == GridLine::from_coord(c_, line_type) {
                        end_index = Some(i)
                    }
                },
                _ => ()
            }
        }
        (start_index.unwrap(), end_index.unwrap())
    };

    if let Some((left_n_start_idx, left_n_end_idx)) = c.neighbor_left().map(|p| gridline_indices(p)) {
        match lines.get_mut(left_n_start_idx) {
            Some(LineTemplate::End(gl) @ left_start) => {
                *left_start = LineTemplate::Interval(GridLine::from_coord(c, line_type), gl);
                lines.insert(left_n_start_idx+1, 
                    LineTemplate::End(GridLine::from_coord(c, line_type)));
            },
            LineTemplate::Interval(gl, _) => {
                lines.insert_at(right_n_start_idx+1, 
                    LineTemplate::Interval(GridLine::from_coord(c, line_type), gl));
                lines.insert_at(right_n_start_idx+2, 
                    LineTemplate::End(GridLine::from_coord(c, line_type)));
            }
        }
    } else let if Some(right_n_start_idx, right_n_end_idx) = c.neighbor_right().map(|p| gridline_indices(p)) {
        lines.update_at(right_n_start_idx, match value {
            LineTemplate::Start(gl){
            },
            LineTemplate::Interval(gl, _){
            }
        })
    } else if Some(parent_start_idx, _) == c.parent().fold(|p| gridline_indices(p)) {
        lines.insert_at(parent_start_idx+1, 
            LineTemplate::Start(GridLine::from_coord(c, line_type)))
        lines.insert_at(parent_start_idx+2, 
            LineTemplate::End(GridLine::from_coord(c, line_type)))
    } 


}


impl Grid {
    fn new(grammars: &HashMap<Coordinate, Grammar>, grid_root: Coordinate) -> Self {
        // since all coordinates are relateive to the root, we might as well
        // instantiate the line templates with the terminal lines for the root coordinate. i suppose
        // we'll also need to represent the root start and end in a special way (see
        // Grid::to_string).
        // so all the computations for finding the index will be in relation to the parent,
        // starting with the root (we'll also depend a bit on neigbors ubt we'll get to that
        // shortly)
        let mut template_rows = vec![
            LineTemplate::Start(GridLine::from_coord(ROOT!{}, GridLineType::Row)),
            LineTemplate::End(GridLine::from_coord(ROOT!{}, GridLineType::Row)),
        ];
        let mut template_cols = vec![
            LineTemplate::Start(GridLine::from_coord(ROOT!{}, GridLineType::Column)),
            LineTemplate::End(GridLine::from_coord(ROOT!{}, GridLineType::Column)),
        ];
        compute_grid(&mut template_rows, &mut template_cols, grid_root, grammars, 0);
        // template_rows.sort_by(|a, b| cmp_gridlines(a, b, GridLineType::Row));
        // template_cols.sort_by(|a, b| cmp_gridlines(a, b, GridLineType::Column));
        Grid {
            template_rows: template_rows,
            template_cols: template_cols,
        }
    } 
     
    fn line_template_to_string(lines: Vec<LineTemplate>) -> String {
        let mut output = String::new();
        for line in lines.iter() {
            match line.clone() {
                LineTemplate::Start(line) => {
                    output += format!{"[{}-start]", line.to_string()}.deref();
                },
                LineTemplate::Interval(start_line, end_line) => {
                    output += format!{
                        "[{}-start {}-end]",
                        start_line.to_string(),
                        end_line.to_string(),
                    }.deref();
                },
                LineTemplate::End(line) => {
                    output += format!{"[{}-end]", line.to_string()}.deref();
                },
                LineTemplate::Span(width) => {
                    output += format!{" {}px ", width}.deref();
                },
            }
        }
        output
    }

    fn to_string(&self) -> String {
        format!{
            "display: grid; grid-template-rows: \"{}\"; grid-template-columns: \"{}\";",
            Self::line_template_to_string(self.template_rows.clone()),
            Self::line_template_to_string(self.template_cols.clone()),
        }
    }
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

// macro for easily defining a coordinate
// either absolutely or relative to it's parent coordinate
macro_rules! coord {
    ( $( $x:expr ), + ) => {
        {
            let mut v: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();
            $(
                v.push(non_zero_u32_tuple($x));
            )+
            Coordinate {
                row_cols: v,
            }
        }
    };

    ( $parent:expr ; $x:expr ) => ( Coordinate::child_of(&$parent.clone(), non_zero_u32_tuple($x)) );
}

// macros defining the ROOT and META coordinates
macro_rules! ROOT {
    () => ( coord!{ (1,1) } );
}

macro_rules! META {
    () => ( coord!{ (1,2) } );
}

// helper methods
fn col_to_string(col : i32) -> String {
    const alpha_offset : i32 = 64;
    let normalized_col = if col == 26 { 26 } else { col % 26 };
    let mut base_str = js! { 
        return String.fromCharCode(@{normalized_col + alpha_offset});
    }.into_string().unwrap();
    if col > 26 {
        base_str.push_str(col_to_string(col - 26).deref());
    }
    base_str
}

fn row_col_to_string((row, col): (i32, i32)) -> String {
    let row_str = row.to_string();
    let col_str = col_to_string(col);
    format!{"{}{}", col_str, row_str} 
}

fn coord_show(row_cols: Vec<(i32, i32)>) -> Option<String> {
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


// Methods for interacting with coordinate struct
impl Coordinate {
    fn root() -> Coordinate {
        ROOT!{}
    }

    fn child_of(parent: &Self, child_coord: (NonZeroU32, NonZeroU32)) -> Coordinate {
        let mut new_row_col = parent.clone().row_cols;
        new_row_col.push(child_coord);
        Coordinate{ row_cols: new_row_col }
    }

    fn to_string(&self) -> String {
        coord_show(self.row_cols.iter()
             .map(|(r, c)| (r.get() as i32, c.get() as i32)).collect()).unwrap()
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
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let root_grammar = Grammar {
            name: "root".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (1,2), (2,1), (2,2)]),
        };
        let meta_grammar = Grammar {
            name: "meta".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (1,2)]),
        };
        Model {
            root: root_grammar.clone(),
            meta: meta_grammar.clone(),
            view_root: ROOT!{},
            grammars: hashmap! {
                ROOT!{} => root_grammar.clone(),
                coord!{ ROOT!{}; (1,1) } => Grammar::default(),
                coord!{ ROOT!{}; (1,2) } => Grammar::default(),
                coord!{ ROOT!{}; (2,1) } => Grammar::default(),
                coord!{ ROOT!{}; (2,2) } => Grammar::default(),
                coord!{ META!{}; (1,1) } => Grammar::suggestion("js grammar".to_string(), "This is js".to_string()),
                coord!{ META!{}; (1,2) } => Grammar::suggestion("java grammar".to_string(), "This is java".to_string()),
            },
            value: String::new(),
            active_cell: Some(coord!{ ROOT!{}; (1,1) }),
            suggestions: vec![ coord!{ META!{}; (1,1) }, coord!{ META!{}; (1,1) } ],
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
        }
    }

    // The update function is split into sub-update functions that 
    // are specifc to each EventType
    fn update(&mut self, event_type: Self::Message) -> ShouldRender {
        match event_type {
            Action::Noop => {
                // Update your model on events
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
                let source_grammar = self.grammars.get(&source_coord);
                match source_grammar.clone() {
                    Some(g) => {
                        self.grammars.insert(dest_coord, g.clone());
                    }
                    None => ()
                }
                true
            }

            Action::SetActiveMenu(activeMenu) => {
                self.open_side_menu = activeMenu;
                true
            }

            Action::ReadSession(file) => {
                let callback = self.link.send_back(Action::LoadSession);
                let task = self.reader.read_file(file, callback);
                self.tasks.push(task);
                false
            }

            Action::LoadSession(file_data) => {
                let session : Session = serde_json::from_str(format!{"{:?}", file_data}.deref()).unwrap();
                self.load_session(session);
                true
            }
        }
    }

    fn view(&self) -> Html<Self> {
        let grid = Grid::new(&self.grammars, ROOT!{});

        html! {
            <div>

                { view_side_nav(&self) }

                { view_menu_bar(&self) }

                { view_tab_bar(&self) }


                <div class="main">
                    <h1>{ "integrated spreasheet environment" }</h1>

                    <div id="grammars" class="grid" style={ grid.to_string() }>
                        { view_grammars(&self) }
                    </div>
                </div>
            </div>
        }
    }
}

fn view_side_nav(m: &Model) -> Html<Model> {
    let mut side_menu_nodes = VList::<Model>::new();
    let mut side_menu_section = html! { <></> };
    for (index, side_menu) in m.side_menus.iter().enumerate() {
        if Some(index as i32) == m.open_side_menu {
            side_menu_nodes.add_child(html! {
                <button class="active-menu" onclick=|e| {
                            Action::SetActiveMenu(None)
                    }>
                    <img 
                        src={side_menu.icon_path.clone()} 
                        width="40px" alt={side_menu.name.clone()}>
                    </img>
                </button>
            });

            side_menu_section = view_side_menu(m, side_menu);
        } else {
            side_menu_nodes.add_child(html! {
                <button onclick=|e| {
                            Action::SetActiveMenu(Some(index as i32))
                    }>
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

fn view_side_menu(m: &Model, side_menu: &SideMenu) -> Html<Model> {
    match side_menu.name.deref(){
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
                    <input type="file" onchange=|value| {
                        if let ChangeData::Files(files) = value {
                            if files.len() >= 1 {
                                if let Some(file) = files.iter().nth(0) {
                                    return Action::ReadSession(file);
                                }
                            }
                        }
                        Action::Noop
                    }>
                    </input>

                    <h3>{"save session"}</h3>
                    <br></br>
                    <input type="file" onchange=|value| {
                        if let ChangeData::Files(files) = value {
                            if files.len() >= 1 {
                                if let Some(file) = files.iter().nth(0) {
                                    return Action::ReadSession(file);
                                }
                            }
                        }
                        Action::Noop
                    }>
                        
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


fn view_menu_bar(m: &Model) -> Html<Model> {
    html! {
        <div class="menu-bar horizontal-bar">
            <button class="menu-bar-button">
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
            <button class="menu-bar-button">
                { "Insert Row" }
            </button>
            <button class="menu-bar-button">
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

fn view_tab_bar(m: &Model) -> Html<Model> {
    let mut tabs = VList::<Model>::new();
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
            <button class="newtab-btn">{ "+" }</button>
        </div>
    }
}

fn view_grammars(m: &Model) -> VList<Model> {
    let mut grammar_nodes = VList::<Model>::new();
    grammar_nodes.add_child(html! {
        <div style=m.root.style.to_string()>{"ROOT"}</div>
    });
    match m.root.kind.clone() {
        Kind::Grid(child_coords) => {
            for coord in child_coords {
                let full_coord = Coordinate::child_of(&ROOT!{}, coord.clone());
                grammar_nodes.add_child(view_grammar(m, full_coord));
            }
        }
        _ => () 
    }

    grammar_nodes
}

fn view_grammar(m: &Model, coord: Coordinate) -> Html<Model> {
    if let Some(grammar) = m.grammars.get(&coord) {
        match grammar.kind.clone() {
            Kind::Text(value) => {
                view_text_grammar(grammar.clone(), &coord, value)
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
                view_input_grammar(grammar.clone(), coord, suggestions, value, is_active)
            }
            Kind::Interactive(interactive) => {
                // TODO: 
                html! { <></> }
            }
            Kind::Grid(_) => {
                view_grid_grammar(grammar.clone(), &coord)
            }
        }
    } else {
        // return empty fragment
        html! { <></> }
    }
}

fn view_input_grammar(grammar: Grammar, coord: Coordinate, suggestions: Vec<(Coordinate, Grammar)>, value: String, is_active: bool) -> Html<Model> {
    let mut suggestion_nodes = VList::<Model>::new();
    let mut active_cell_class = "cell-inactive";
    if is_active {
        active_cell_class = "cell-active";
        for (s_coord, s_grammar) in suggestions {
            let c = coord.clone();
            suggestion_nodes.add_child(html! {
                <a 
                    tabindex=-1
                    onclick=|e| {
                        //if e.key() == "Enter"  {
                            Action::DoCompletion(s_coord.clone(), c.clone())
                        //} else {
                        //    Action::Noop
                        //}
                    }>
                    {&s_grammar.name}
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
        <div class="cell suggestion" style={ grammar.style(&coord) }>
            <input 
                class={ format!{ "cell-data {}", active_cell_class } }
                value=value
                oninput=|e| {
                    Action::ChangeInput(coord.clone(), e.value)
                }
                onclick=|e| {
                    Action::SetActiveCell(new_active_cell.clone())
                }
                >
            </input>
            
            { suggestions }
        </div>
    }
}

fn view_text_grammar(grammar: Grammar, coord: &Coordinate, value : String) -> Html<Model> {
    html! {
        <div style={ grammar.style(&coord) }>
            { value }
        </div>
    }
}

fn view_grid_grammar(grammar: Grammar, coord: &Coordinate) -> Html<Model> {
    html! {
        <div style={ grammar.style(&coord) }>
            // empty, with only borders and grid placement
        </div>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
    yew::start_app::<Model>();
    Ok(())
}
