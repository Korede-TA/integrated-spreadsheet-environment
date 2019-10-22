use yew::{html, Component, ComponentLink, Html, ShouldRender};
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
type Coordinate = [(i32, i32)]

fn show_coord(c : Coordinate) -> String {
    fn show_col(col : i32) -> String {
        let mut col_str = String::fromString("");
        for i in (col..0).step_by(26) {
            let alphaOffset = 64;
            let normalizedI = if col == 26 {
                26 
            } else {
                i % 26
            }
            let baseChar = (normalizedI + alphaOffset) as char;
            col_str.push(baseChar);
        }
        col_str
    }

    let mut coord_str = String::fromString("");
    for (row, col) in c {
        coord_str.push_str(row.to_string());
        let col_letter = (96 + col) as char;
        coord_str.push(show_col(col)));
        coord_str.push_str("-");
    }
    let _ = coord_str.pop(); // remove trailing "-"
    coord_str
}

fn childOf(c: Coordinate) -> c {

}

type DimensionIndex =  ([i32, i32], i32) 

fn ppDimensionIndex(d : DimensionIndex) -> String {
    // TODO
    ""
}

// Standalone module, with all code that involves
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
    grammars: GrammarMap,
}

enum Msg {
  ChangeCellData(Coordinate, String),
  SelectBelow(Coordinate),
  AddNestedTable(Coordinate, grammar::Layout),
  SelectCells(selection),
  AutoCompeleteGrammar(Coordinate, grammar::Grammar),
  ToggleContextMenu(Coordinate, bool),
  ActivateCell(Coordinate),
  Noop,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: jComponentLink<Self>) -> Self {
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

        Model {
            root: root,
            what_we_count: "click".into()
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Noop => {
                // Update your model on events
                true
            }
            _ => false
        }
    }

    fn view_row() -> Html<Self> {
        <div></div>
    }

    fn view_grammar(&self, g : Grammar) -> Html<Self> {
        let classes
        <td id=format!("cell-{}", show_coord(g.coordinate)) class=format!("cell dropdown row- col-", )>
            [ classes
                [ "cell"
                ; "dropdown"
                ; "row-" ^ C.showPrefix coord ^ (coord |> C.row |> C.showRow)
                ; "col-" ^ C.showPrefix coord ^ (coord |> C.col |> C.showCol) ]
            ; id ("cell-" ^ C.show coord) ]
            (*; onWithOptions "click" opts (SelectCells coord) ]*)
            [viewGrammar m coord]
        </td>
    }

    fn view(&self) -> Html<Self> {
        html! {
            <button onclick=|_| Msg::Noop>{ "Click me!" }</button>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
