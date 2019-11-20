use std::collections::HashMap;
use std::num::NonZeroU32;
use serde::{Serialize, Deserialize};
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::{ConsoleService};
use yew::virtual_dom::{VNode, VList, VText};

#[macro_use] extern crate maplit;
#[macro_use] extern crate stdweb;

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

    // list of suggestions that are available to a cell at some point in time
    // TODO: change this to be Vec<Coordinates>
    suggestions: Vec<String>,

    // utility fields: 
    // - Yew Services for accessing browser APIs 
    //   (https://github.com/yewstack/yew#services), 
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
                coord!{ META!{}; (1,1) } => Grammar::default(),
                coord!{ META!{}; (1,2) } => Grammar::default(),
            },
            suggestions: vec![ "JS Module".to_string(), "Java Module".to_string() ],
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
                    grammar_nodes.add_child(match &grammar.map(|g| g.kind.clone()) {
                        Some(Kind::Text(value)) => {
                            html! {
                                <div style=&grammar.map(|g| g.style.to_string()).unwrap_or_default()
                                >
                                    { value }
                                </div>
                            }
                        }
                        Some(Kind::Input(value)) => {
                            let mut suggestion_nodes = VList::<Model>::new();
                            for s in &self.suggestions {
                                suggestion_nodes.add_child(VNode::VText(VText::new(s.to_string())));
                            }
                            html! {
                                <div class="cell suggestion">
                                    <input 
                                        class="cell-data"
                                        style=&grammar.map(|g| g.style.to_string()).unwrap_or_default()
                                        value=value
                                        oninput=|e| {
                                            Action::ChangeInput(full_coord.clone(), e.value)
                                        }>
                                    </input>
                                    <div class="suggestion-content">
                                        { suggestion_nodes }
                                    </div>
                                </div>
                            }
                        }
                        Some(Kind::Grid(_)) => {
                            html! {
                                <div style=&grammar.map(|g| g.style.to_string()).unwrap_or_default()>
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
                <h1>{ "integrated spreasheet environment" }</h1>

                <div id="grammars">
                    { grammar_nodes }
                </div>
            </div>
        }
    }


}

pub fn main() {
    yew::start_app::<Model>();
}
