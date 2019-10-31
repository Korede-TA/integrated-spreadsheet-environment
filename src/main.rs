use yew::{html, Component, ComponentLink, Html, ShouldRender};
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
struct Coordinate {
    row_column_pairs: Vec<(i32, i32)>
    plain_coordinate: (i32, i32)
}

impl Coordinate {
    fn show(&self) -> String {
        fn show_col(col : i32) -> String {
            let mut col_str = String::new();
            for i in (col..0).step_by(26) {
                let alphaOffset = 64;
                let normalizedI = if col == 26 {
                    26 
                } else {
                    i % 26
                };
                let baseChar = (normalizedI + alphaOffset) as char;
                col_str.push(baseChar);
            }
            col_str
        }

        let mut coord_str = String::new();
        for (row, col) in self.row_column_pairs {
            coord_str.push_str(row.to_string());
            let col_letter = (96 + col) as char;
            coord_str.push(show_col(col));
            coord_str.push_str("-");
        }
        coord_str.pop(); // remove trailing "-"
        coord_str
    }

    fn parent(&self) -> Option<Self> {
        match self.row_column_pairs.as_slice().split_last() {
            Some(_, first_elements) => Some(first_elements),
            _ => None,
        }
    }

    fn row(&self) -> Option<i32> {
        match self.row_column_pairs.as_slice().split_last() {
            Some((row, _), _) => Some(row),
            _ => None,
        }
    }

    fn col(&self) -> Option<i32> {
        match self.row_column_pairs.as_slice().split_last() {
            Some((_, col), _) => Some(col),
            _ => None,
        }
    }
}

struct Span(i32, i32);

// grammars===tables, encapsulates the rows/columns and spans of a table

type Color = String;

#[derive(Hash, Eq, PartialEq, Debug)]
struct Borders {
    top: (Color, bool),
    right: (Color, bool),
    left: (Color, bool),
    bottom: (Color, bool),
    collapse: bool
}

impl Borders {
    pub fn all(color : Color) -> Borders {
        Borders {
            top: (color, true),
            right: (color, true),
            left: (color, true),
            bottom: (color, true),
            collapse: false,
        }
    }

    fn none() -> Borders {
        Borders {
            top: ("none", true),
            right: ("none", true),
            left: ("none", true),
            bottom: ("none", true),
            collapse: false,
        }
        TS
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Font {
    weight: i32, 
    color: Color, 
    // family : String,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Style {
    borders: Borders, 
    font: Font,
}

impl Style {
    fn default() -> Style {
        Style {
            borders: Borders::all("black"),
            font: Font {
                weight: 400,
                color: "black",
            }
        }
    }
}

// Kinds of grammars in the system
#[derive(Hash, Eq, PartialEq, Debug)]
enum Kind {
    // Text is a static piece of text that is displayed
    Text(String),
    // Input is a html input that can be modified by the user 
    Input(String),
    // Button(String),
    Table(Vec<Vec<Grammar>>),
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Grammar {
    name: String,
    coord: (i32, i32),
    style: Style,
    kind: Kind,
}

// Model
struct Model {
    root: Grammar,
}

enum Msg {
  ChangeCellData(Coordinate, String),
  SelectBelow(Coordinate),
  AddNestedTable(Coordinate, Grid),
  // SelectCells(selection),
  AutoCompeleteGrammar(Coordinate, Grammar),
  ToggleContextMenu(Coordinate, bool),
  ActivateCell(Coordinate),
  Noop,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let root = Grammar {
            name: "root",
            coordinate: [(1,1)],
            Style: Style::default(),
            mode: Mode::Table(Grid {
                rows: 3, cols: 3
            }),
        };

        /* let children : Vec<grammar::Grammar> = 
            (1..(3*3)).map(|i| -> grammar::Grammar {
                name: ""
                coordinate: [(1,1)]
            }); */

        Model {
            root: root,
            grammar: vec![],
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

    /*fn view_row() -> Html<Self> {
        <tr>
            { 
            }
        </tr>
    }

    fn view_grammar(&self, g : Grammar) -> Html<Self> {

        html! {
            <td id={ format!("cell-{}", g.coordinate.show()) } class={ format!("cell dropdown row- col-", ) }>
                /*
                [ classes
                    [ "cell"
                    ; "dropdown"
                    ; "row-" ^ C.showPrefix coord ^ (coord |> C.row |> C.showRow)
                    ; "col-" ^ C.showPrefix coord ^ (coord |> C.col |> C.showCol) ]
                ; id ("cell-" ^ C.show coord) ]
                (*; onWithOptions "click" opts (SelectCells coord) ]*)
                [viewGrammar m coord]
                */
            </td>
        }
    } */

    fn view(&self) -> Html<Self> {
        html! {
            <button onclick=|_| Msg::Noop>{ "Click me!" }</button>

                
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
