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
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::{ConsoleService};
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

// Style contains both the 
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
color: {};",
        self.border_color,
        self.border_collapse,
        self.font_weight,
        self.font_color,
        }
    }
}

// Kinds of grammars in the system.
// Since this is an Enum, a Grammar's kind field
// can only be set to one these variants at a time
#[derive(Serialize, Deserialize, Debug, Clone)]
enum Kind {
    Text(String),
    Input(String),
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
}

// based on CSS grid template rows and columns
enum GridLine {
    Start(Coordinate, /*width*/ Option<f64>),
    End(Coordinate),
}

enum GridLineType {
    Row, Column,
}

// cmp_gridlines compares two gridlines of a certain line_type (Row or Column)
fn cmp_gridlines(line1: &GridLine, line2: &GridLine, line_type: GridLineType) -> Ordering {
    match (line1.clone(), line2.clone()) {
        // start and end lines of the same cell:
        (GridLine::Start(coord1, _), GridLine::End(coord2)) if coord1 == coord2 => Ordering::Greater,
        (GridLine::End(coord1), GridLine::Start(coord2, _)) if coord1 == coord2 => Ordering::Less,
        // start and end lines of sibling cells (in the same sub-table)
        (GridLine::Start(coord1, _), GridLine::End(coord2)) => {
            match coord1.is_n_parent(&coord2) {
                Some(n) if n > 0 => Ordering::Greater,
                _ =>
                    match line_type {
                        // compare rows
                        GridLineType::Row => coord1.row_cols.last().unwrap().0.cmp(&coord2.row_cols.last().unwrap().0),
                        // compare columns
                        GridLineType::Column => coord1.row_cols.last().unwrap().1.cmp(&coord2.row_cols.last().unwrap().1),
                    },
            }
        },
        (GridLine::End(coord1), GridLine::Start(coord2, _)) => {
            match coord1.is_n_parent(&coord2) {
                Some(n) if n > 0 => Ordering::Less,
                _ =>
                    match line_type {
                        // compare rows
                        GridLineType::Row => coord1.row_cols.last().unwrap().0.cmp(&coord2.row_cols.last().unwrap().0),
                        // compare columns
                        GridLineType::Column => coord1.row_cols.last().unwrap().1.cmp(&coord2.row_cols.last().unwrap().1),
                    },
            }
        },
        (GridLine::Start(coord1, _), GridLine::Start(coord2, _)) => {
            // coord1.row_cols.last().unwrap().0.cmp(&coord2.row_cols.last().unwrap().0)
            if let Some(_) = coord1.is_n_parent(&coord2) {
                Ordering::Less
            } else {
                coord1.row_cols.last().unwrap().0.cmp(&coord2.row_cols.last().unwrap().0)
            }
        }
        (GridLine::End(coord1), GridLine::End(coord2)) => {
            if let Some(_) = coord1.is_n_parent(&coord2) {
                Ordering::Less
            } else {
                coord1.row_cols.last().unwrap().1.cmp(&coord2.row_cols.last().unwrap().1)
            }
        }
    }
}

struct Grid {
    template_rows: Vec<GridLine>,
    template_cols: Vec<GridLine>,
}

const MAX_GRID_DEPTH: i32 = 20;
const DEFAULT_CELL_WIDTH: f64 = 30.0;

// we will utilize CSS grid's `grid-template-rows` and `grid-template-cols`
// properties to define the explicit gridlines for nested grid.
// (see: https://gridbyexample.com/examples/example10/)
//
// the individual grammar cells will use `grid-row-start`, `grid-row-end` and
// `grid-col-start` and `grid-col-end` to define the row and column gridlines where
// they start and stop.
fn compute_grid(
    coord: Coordinate,
    grammars: &HashMap<Coordinate, Grammar>,
    template_rows: &mut Vec<GridLine>,
    template_cols: &mut Vec<GridLine>,
    // gridlines: &mut HashMap<Coordinate, (i32 /* start */ , f64 /* width */, i32 /* stop */)>,
    depth: i32,
) {

    if depth == MAX_GRID_DEPTH {
        return;
    }

    let grammar = grammars.get(&coord).unwrap();

    match &grammar.kind {
        Kind::Grid(sub_coords) => {
            // to account for merged cells, index is based on rows with most columns
            // and column with most rows
            let mut row_to_col_count : HashMap<i32, i32> = HashMap::new();
            for (row, _col) in sub_coords.iter() {
                row_to_col_count.get_mut(&(row.get() as i32)).map(|val| *val += 1);
            };

            template_rows.push(GridLine::Start(coord.clone(), None));
            for sub_coord_fragment in sub_coords {
                let sub_coord = Coordinate::child_of(&coord, *sub_coord_fragment);
                template_rows.push(GridLine::Start(sub_coord.clone(), None));
                template_cols.push(GridLine::Start(sub_coord.clone(), None));
                compute_grid(sub_coord.clone(), grammars, template_rows, template_cols, depth+1);
                template_rows.push(GridLine::End(sub_coord.clone()));
                template_cols.push(GridLine::End(sub_coord.clone()));
            }
            template_rows.push(GridLine::End(coord.clone()));
        }
        _ => {
            template_rows.push(GridLine::Start(coord.clone(), Some(DEFAULT_CELL_WIDTH)));
            template_rows.push(GridLine::End(coord.clone()));
        }
    }
}

impl Grid {
    fn new(grammars: &HashMap<Coordinate, Grammar>, grid_root: Coordinate) -> Self {
        let mut template_rows = Vec::new();
        let mut template_cols = Vec::new();
        compute_grid(grid_root, grammars, &mut template_rows, &mut template_cols, 0);
        template_rows.sort_by(|a, b| cmp_gridlines(a, b, GridLineType::Row));
        template_cols.sort_by(|a, b| cmp_gridlines(a, b, GridLineType::Column));
        Grid {
            template_rows: template_rows,
            template_cols: template_cols,
        }
    }

    fn to_string() -> String {
        String::new()
    }
}


// Coordinate specifies the nested coordinate structure
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
struct Coordinate {
    row_cols: Vec<(NonZeroU32, NonZeroU32)>, // should never be empty list
}

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
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
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

            _ => false
        }
    }

    fn view(&self) -> Html<Self> {

        html! {
            <div>

                <div class="sidenav">
                    <a href="#">
                        <img src="assets/logo.png" width="40px"></img>
                    </a>
                    <a href="#">
                        <img src="assets/folder_icon.png" width="40px"></img>
                    </a>
                    <a href="#">
                        <img src="assets/settings_icon.png" width="40px"></img>
                    </a>
                    <a href="#">
                        <img src="assets/info_icon.png" width="40px"></img>
                    </a>
                </div>

                <div class="main">
                    <div class="tab">
                        <button class="tablinks">{ "Session 1" }</button>
                        <button class="newtab-btn">{ "+" }</button>
                    </div>

                    <h1>{ "integrated spreasheet environment" }</h1>

                    <div id="grammars" style="display: grid;">
                        { view_grammars(&self) }
                    </div>
                </div>
            </div>
        }
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
    let grammar = m.grammars.get(&coord);
    let style = &grammar.map(|g| g.style.to_string()).unwrap_or_default();
    grammar.map(|g| match g.kind.clone() {
        Kind::Text(value) => {
            view_text_grammar(g.clone(), value)
        }
        Kind::Input(value) => {
            let is_active = m.active_cell.clone().map(|c| c == coord).unwrap_or(false);
            let suggestions = m.suggestions.iter().filter_map(|s_coord| {
                m.grammars.get(&s_coord).map(|g| (s_coord.clone(), g.clone()))
            }).collect();
            view_input_grammar(g.clone(), coord, suggestions, value, is_active)
        }
        Kind::Grid(_) => {
            view_grid_grammar(g.clone())
        }
    }).unwrap_or(html! { <></> })
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
        <div class="cell suggestion" style={ grammar.style.clone() }>
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

fn view_text_grammar(grammar: Grammar, value : String) -> Html<Model> {
    html! {
        <div style={ grammar.style.clone() }>
            { value }
        </div>
    }
}

fn view_grid_grammar(grammar: Grammar) -> Html<Model> {
    html! {
        <div style={ grammar.style.clone() }>
            {"NESTED GRAMMAR"}
        </div>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
    yew::start_app::<Model>();
    Ok(())
}