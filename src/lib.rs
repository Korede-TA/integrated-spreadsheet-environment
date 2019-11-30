#![recursion_limit = "512"]

use std::collections::HashMap;
use std::num::NonZeroU32;
use serde::{Serialize, Deserialize};
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::{ConsoleService};
use yew::virtual_dom::{VList};
use wasm_bindgen::prelude::*;
use stdweb::web::{document, Element, INode, IParentNode};
use log::trace;

#[macro_use] extern crate maplit;
#[macro_use] extern crate stdweb;
#[macro_use] extern crate log;
extern crate web_logger;

/*
 * DATA MODEL:
 * is centered around the "grammars" map: HashMap<Coordinate, Grammar>
 *
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
        let mut grammar_nodes = VList::<Model>::new();
        grammar_nodes.add_child(html! {
            <div style=self.root.style.to_string()>{"ROOT"}</div>
        });
        match &self.root.kind {
            Kind::Grid(child_coords) => {
                for coord in child_coords {
                    let full_coord = Coordinate::child_of(&(ROOT!{}), *coord);
                    let grammar = &self.grammars.get(&full_coord);
                    let style = &grammar.map(|g| g.style.to_string()).unwrap_or_default();
                    grammar_nodes.add_child(match &grammar.map(|g| g.kind.clone()) {
                        Some(Kind::Text(value)) => {
                            html! {
                                <div style={ style.clone() }
                                >
                                    { value }
                                </div>
                            }
                        }
                        Some(Kind::Input(value)) => {
                            let mut suggestion_nodes = VList::<Model>::new();
                            let mut active_cell_class = "cell-inactive";
                            if self.active_cell.clone().map(|coord| coord == full_coord).unwrap_or(false) {
                                active_cell_class = "cell-active";
                                for s in &self.suggestions {
                                    // suggestion_nodes.add_child(VNode::VText(VText::new(s.to_string())));
                                    let suggested_grammar = &self.grammars.get(&s);
                                    let source_coord = s.clone();
                                    let dest_coord = full_coord.clone();
                                    suggestion_nodes.add_child(html! {
                                        <a 
                                            tabindex=-1
                                            onclick=|e| {
                                                //if e.key() == "Enter"  {
                                                    Action::DoCompletion(source_coord.clone(), dest_coord.clone())
                                                //} else {
                                                //    Action::Noop
                                                //}
                                            }>
                                            {&suggested_grammar.map(|g| g.name.clone()).unwrap_or_default()}
                                        </a>
                                    })
                                    
                                }
                            }
                            let suggestions = html!{
                                <div class="suggestion-content">
                                    { suggestion_nodes }
                                </div>
                            };

                            let new_active_cell = full_coord.clone();

                            html! {
                                <div class="cell suggestion" style={ style.clone() }>
                                    <input 
                                        class={ format!{ "cell-data {}", active_cell_class } }
                                        value=value
                                        oninput=|e| {
                                            Action::ChangeInput(full_coord.clone(), e.value)
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
                        Some(Kind::Grid(_)) => {
                            html! {
                                <div style={ style.clone() }>
                                    {"NESTED GRAMMAR"}
                                </div>
                            }
                        }
                        None => html! { <></> }
                    })
                }
            }
            _ => () 
        }

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

                <main>
                    <div class="tab">
                        <button class="tablinks">{ "London" }</button>
                        <button class="newtab-btn">{ "+" }</button>
                    </div>

                    <h1>{ "integrated spreasheet environment" }</h1>

                    <div id="grammars" style="display: grid;">
                        { grammar_nodes }
                    </div>
                </main>
            </div>
        }
    }

}


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator. (see Cargo.toml for why we use optimixed allocator)
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/* JS Export Macros
 *
 * In order to export Rust functions to JS via Wasm, we have two options
 *
 * 1. Using stdweb, there's a `#[js_export]` macro that let's us call
 *    rust functions like so:
 *    - In Rust:
 *    ```
 *    #[js_export]
 *    fn hash( string: String ) -> String {
 *    ```
 *    - In the browser:
 *    ```
 *    <script src="hasher.js"></script>
 *    <script>
 *      Rust.hasher.then( function( hasher ) {
 *          console.log( hasher.hash( "Hello world!" ) );
 *      });
 *      </script>
 *    ```
 *    See more info here: https://github.com/koute/stdweb/tree/master/examples/hasher
 *
 * 2. Using wasm_bindgen and web_sys:
 *
 */

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    web_logger::init();
    yew::start_app::<Model>();
    Ok(())
}
