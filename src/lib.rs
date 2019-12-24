#![recursion_limit = "512"]

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::char::from_u32;
use std::ops::Deref;
use std::iter::FromIterator;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use yew::{html, ChangeData, Component, ComponentLink, Html, ShouldRender, KeyPressEvent};
use yew::events::{IKeyboardEvent};
use yew::services::{ConsoleService};
use yew::services::reader::{File, FileChunk, FileData, ReaderService, ReaderTask};
use yew::virtual_dom::{VList};
use wasm_bindgen::prelude::*;
use stdweb::Value;
use stdweb::web::{document, Element, INode, IParentNode};
use log::trace;
use itertools::Itertools;
use pest::Parser;

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
            width: 120.00,
            height: 50.00,
            border_color: "grey".to_string(),
            border_collapse: true,
            font_weight: 400,
            font_color: "black".to_string(),
        }
    }

    fn to_string(&self) -> String {
        format!{
        "width: {}px;
height: {}px;
border: 1px solid {};
border-collapse: {};
font-weight: {};
color: {};\n",
        self.width,
        self.height,
        self.border_color,
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
                for (row, col) in sub_coords {
                    if col.get() == 1 && row.get() != 1 {
                        grid_area_str.pop();
                        grid_area_str += "\"\n\"";
                    }
                    let sub_coord = Coordinate::child_of(coord, (row.clone(), col.clone()));
                    grid_area_str += format!{"cell-{} ", sub_coord.to_string()}.deref();
                }
                grid_area_str.pop();
                grid_area_str += "\"";
                format!{"display: grid;\n{}grid-template-areas: \n{};\n", self.style.to_string(), grid_area_str}
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

            info!{"COORD: {}", $coord_str};
            let mut fragments: Vec<(NonZeroU32, NonZeroU32)> = Vec::new();

            let pairs = CoordinateParser::parse(Rule::coordinate, $coord_str).unwrap_or_else(|e| panic!("{}", e));

            for pair in pairs {
                info!{"PAIR: {}", pair};
            
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
    () => ( coord!("A1") );
}

macro_rules! META {
    () => ( coord!("A2") );
}

fn row_col_to_string((row, col): (u32, u32)) -> String {
    let row_str = row.to_string();
    let col_str = from_u32(col + 64).unwrap();
    format!{"{}{}", col_str, row_str} 
}

fn coord_show(row_cols: Vec<(u32, u32)>) -> Option<String> {
    info!{"coord_show: {:?}", row_cols};
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
             .map(|(r, c)| (r.get(), c.get())).collect()).unwrap()
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

    // Grid Operations
    AddNestedGrid(Coordinate, (u32 /*rows*/, u32 /*cols*/))
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let root_grammar = Grammar {
            name: "root".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (1,2), (1,3), (2,1), (2,2), (2,3) ]),
        };
        let meta_grammar = Grammar {
            name: "meta".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (1,2)]),
        };
        Model {
            root: root_grammar.clone(),
            meta: meta_grammar.clone(),
            view_root: coord!("root"),
            grammars: hashmap! {
                ROOT!{} => root_grammar.clone(),
                coord!("A1-A1") => Grammar::default(),
                // coord!("A1-A2") => Grammar::default(),
                // coord!("A1-A3") => Grammar::default(),
                coord!("A1-B1") => Grammar::default(),
                // coord!("A1-B2") => Grammar::default(),
                // coord!("A1-B3") => Grammar::default(),
                coord!("A2-B2") => Grammar::suggestion("js grammar".to_string(), "This is js".to_string()),
                coord!("A2-B2") => Grammar::suggestion("java grammar".to_string(), "This is java".to_string()),
            },
            value: String::new(),
            active_cell: Some(coord!("A1-A1")),
            suggestions: vec![ coord!("A2-A1"), coord!("A2-A2") ],
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

            Action::SetActiveMenu(active_menu) => {
                self.open_side_menu = active_menu;
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

            Action::AddNestedGrid(coord, (rows, cols)) => {
                let (r, c) = non_zero_u32_tuple((rows, cols));
                let grammar = Grammar::as_grid(r, c);
                if let Kind::Grid(sub_coords) = grammar.clone().kind {
                    self.active_cell = sub_coords.first().map(|c| Coordinate::child_of(&coord, *c));
                    for sub_coord in sub_coords {
                        self.grammars.insert(Coordinate::child_of(&coord, sub_coord), Grammar::default());
                    }
                }
                self.grammars.insert(coord, grammar);
                true
            }
        }
    }

    fn view(&self) -> Html<Self> {

        let active_cell = self.active_cell.clone();
        html! {
            <div>

                { view_side_nav(&self) }

                { view_menu_bar(&self) }

                { view_tab_bar(&self) }


                <div class="main">
                    <h1>{ "integrated spreasheet environment" }</h1>

                    <div id="grammars" class="grid wrapper" onkeypress=|e| {
                        if e.key() == "g" && e.ctrl_key() {
                            if let Some(coord) = active_cell.clone() {
                                return Action::AddNestedGrid(coord.clone(), (3, 3));
                            }
                        }
                        Action::Noop
                    }>
                        { view_grammar(&self, ROOT!{}) }
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
            <input disabled=true 
                value={
                    if let Some(cell) = m.active_cell.clone() {
                        cell.to_string()
                    } else {
                        "".to_string()
                    }
                }>
            </input>
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
            Kind::Interactive(name, Interactive::Button()) => {
                html! {
                    <button>
                        { name }
                    </button>
                }
            }
            Kind::Interactive(name, Interactive::Slider(value, min, max)) => {
                html! {
                    <input type="range" min={min} max={max} value={value}>
                        { name }
                    </input>
                }
            }
            Kind::Interactive(name, Interactive::Toggle(checked)) => {
                html! {
                    <input type="checkbox" checked={checked}>
                        { name }
                    </input>
                }
            }
            Kind::Grid(sub_coords) => {
                view_grid_grammar(
                    m,
                    grammar.clone(),
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
        <div class="cell suggestion" style={ grammar.style(&coord) }>
            { value }
        </div>
    }
}

fn view_grid_grammar(m: &Model, grammar: Grammar, coord: &Coordinate, sub_coords: Vec<Coordinate>) -> Html<Model> {
    html! {
        <div style={ grammar.style(&coord) }>
            {
                let mut node_list = VList::<Model>::new();

                for c in sub_coords {
                    node_list.add_child(view_grammar(m, c.clone()));
                    node_list.add_child(html!{
                        <div class="handler"></div>
                    });
                }

                node_list
            }
        </div>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
    yew::start_app::<Model>();
    Ok(())
}
