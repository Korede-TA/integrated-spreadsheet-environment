#[macro_use]
extern crate seed;
use seed::prelude::*;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
type Coordinate = [(i32, i32)]

fn childOf(c: Coordinate) -> c {

}

type DimensionIndex =  ([i32, i32], i32) 

fn ppDimensionIndex(d : DimensionIndex) -> String {
    // TODO
    ""
}

mod grammar {
    struct Span(i32, i32);

    // grammars===tables, encapsulates the rows/columns and spans of a table
    struct Layout {
        rows: i32,
        cols: i32,
        spans: [(Coordinate, Span)] 
    };

    type Color = String;

    struct Borders
    { top: (Color, bool)
    , right: (Color, bool)
    , left: (Color, bool)
    , bottom: (Color, bool)
    , collapse: bool }

    fn borderAll(color : Color) -> Borders {
        top: (color, true),
        right: (color, true),
        left: (color, true),
        bottom: (color, true),
        collapse: false,
    }

    let borderNone = Borders {
        top: (none, true),
        right: (none, true),
        left: (none, true),
        bottom: (none, true),
        collapse: false,
    }

    struct Font {
        weight: i32, 
        color: Color, 
        // family : String,
    }

    struct Style {
        borders: Borders, 
        font: Font,
    }

    let defaultStyle = Style {
        borders: borderAll("black"),
        font: Font {
            weight: 400,
            color: "black",
        }
    }

    enum Status {
        Active,
        Selected,
        Inactive,
    }

    (* Main modes (read: types) of grammars in the system
    *)
    enum Mode {
        // Text is a static piece of text that is displayed
        Text(String),
        // Input is a html input that can be modified by the user 
        Input(String),
        // Button(String),
        Table(Layout),
    }

    #[derive(Hash, Eq, PartialEq, Debug)]
    struct Grammar {
        name: String, 
        coord: Coordinate,
        style: Style, 
        mode: Mode, 
        status: Status
    }

    type GrammarMap = HashMap<Coordinate, Grammar>;

    fn addToGrammarMap(gm: GrammarMap, grammars: Vec<Grammar>) {
    }

    fn childrenFromLayout(layout: Layout, coord: Coordinate) -> {
       (1...(layout.rows*layout.cols)).map(|i| -> Grammar{
        name: "",
        coordinate: i % layout.cols
       })
    }
}

// Model

struct Model {
    root: grammar::Grammar,
    // grammarMap: GrammarMap,
}

// Setup a default here, for initialization later.
impl Default for Model {
    fn default() -> Self {
        let root = Grammar {
            name: "root",
            coordinate: [(1,1)],
            Style: grammar::defaultStyle,
            mode: grammar::Mode::Table(Layout {
                rows: 3, cols: 3, spans: []
            }),
        }

        let children : Vec<Grammar> = 
            (1..3*3).map(|i| -> Grammar {
                name: ""
                coordinate: [(1,1)]
            })

        let map = ().fold
        Self {
            root: root,
            what_we_count: "click".into()
        }
    }
}


// Update

#[derive(Clone)]
enum Msg {
  ChangeCellData(Coordinate, String),
  SelectBelow(Coordinate),
  AddNestedTable(Coordinate, Grammar.layout),
  SelectCells(selection),
  AutoCompeleteGrammar(Coordinate, Grammar.t),
  ToggleContextMenu(Coordinate, bool),
  ActivateCell(Coordinate),
  Noop,
}

/// How we update the model
fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.count += 1,
        Msg::Decrement => model.count -= 1,
        Msg::ChangeWWC(what_we_count) => model.what_we_count = what_we_count,
    }
}


// View

/// A simple component.
fn success_level(clicks: i32) -> Node<Msg> {
    let descrip = match clicks {
        0 ..= 5 => "Not very many 🙁",
        6 ..= 9 => "I got my first real six-string 😐",
        10 ..= 11 => "Spinal Tap 🙂",
        _ => "Double pendulum 🙃"
    };
    p![ descrip ]
}

/// The top-level component we pass to the virtual dom.
fn view(model: &Model) -> impl View<Msg> {
    let plural = if model.count == 1 {""} else {"s"};

    // Attrs, Style, Events, and children may be defined separately.
    let outer_style = style!{
            St::Display => "flex";
            St::FlexDirection => "column";
            St::TextAlign => "center"
    };

    div![ outer_style,
        h1![ "The Grand Total" ],
        div![
            style!{
                // Example of conditional logic in a style.
                St::Color => if model.count > 4 {"purple"} else {"gray"};
                St::Border => "2px solid #004422";
                St::Padding => unit!(20, px);
            },
            // We can use normal Rust code and comments in the view.
            h3![ format!("{} {}{} so far", model.count, model.what_we_count, plural) ],
            button![ simple_ev(Ev::Click, Msg::Increment), "+" ],
            button![ simple_ev(Ev::Click, Msg::Decrement), "-" ],

            // Optionally-displaying an element
            if model.count >= 10 { h2![ style!{St::Padding => px(50)}, "Nice!" ] } else { empty![] }
        ],
        success_level(model.count),  // Incorporating a separate component

        h3![ "What are we counting?" ],
        input![ attrs!{At::Value => model.what_we_count}, input_ev(Ev::Input, Msg::ChangeWWC) ]
    ]
}


#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Init::new(Model::default()), update, view)
        .finish()
        .run();
}
