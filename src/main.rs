use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew::virtual_dom::{VList};

type Color = String;

struct Borders {
    top: (Color, bool),
    right: (Color, bool),
    left: (Color, bool),
    bottom: (Color, bool),
    collapse: bool
}

impl Borders {
    fn all(color : Color) -> Borders {
        Borders {
            top: (color, true),
            right: (color, true),
            left: (color, true),
            bottom: (color, true),
            collapse: false,
        }
    }
}

struct Font {
    weight: i32, 
    color: Color, 
}

struct Style {
    borders: Borders, 
    font: Font,
}

impl Style {
    fn default() -> Style {
        Style {
            borders: Borders::all("black".to_string()),
            font: Font {
                weight: 400,
                color: "black".to_string(),
            }
        }
    }

    fn to_string(&self) -> String {
        // TODO: fill this out
        "".to_string()
    }
}

// Kinds of grammars in the system
enum Kind {
    Text(String),
    Input(String),
    Table(Vec<Vec<Grammar>>),
}

#[derive(Debug, Copy)]
struct Grammar {
    name: String,
    style: Style,
    kind: Kind,
}

impl Grammar {
    fn default() -> Grammar {
        Grammar {
            name: "".to_string(),
            style: Style::default(),
            kind: Kind::Text("".to_string()),
        }
    }

}

// Model
struct Model {
    root: Grammar,
}

type Coordinate = Vec<(i32, i32)>;

enum Msg {
  ChangeCellValue(Coordinate, String),
  SelectBelow(Coordinate),
  AddNestedTable(Coordinate),
  AutoCompeleteGrammar(Coordinate, Grammar),
  ToggleContextMenu(Coordinate, bool),
  ActivateCell(Coordinate),
  Noop,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            root: Grammar {
                name: "root".to_string(),
                style: Style::default(),
                kind: Kind::Table(vec! [
                    vec! [ Grammar::default(), Grammar::default(), Grammar::default(),],
                    vec! [ Grammar::default(), Grammar::default(), Grammar::default(),],
                    vec! [ Grammar::default(), Grammar::default(), Grammar::default(),],
                ]),
            },
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

    fn view(&self) -> Html<Self> {
        fn view_row(row : Vec<Grammar>) -> Html<Model> {
            let cell_nodes = VList::new();
            for grammar in row {
                cell_nodes.add_child(view_grammar(grammar));
            }

            html!{
                <div style="display: table-row;">
                    { cell_nodes }
                </div>
            }
        }

        fn view_grammar(grammar : Grammar) -> Html<Model> {
            html! {
                <div>
                    <button onclick=|_| Msg::Noop>{ "Click me!" }</button>
                    
                    { 
                        match grammar.kind {
                            Kind::Text(value) => {
                                <div style="display: table-cell;">
                                    
                                </div>
                            }

                            Kind::Table(table) => {
                                let row_nodes = VList::new();
                                for row in table {
                                    row_nodes.add_child(view_row(row));
                                }
                                <div style="display: table;">
                                    { row_nodes } 
                                </div>
                            }

                            _ => html! { <td>{"Empty Cell"}</td> }
                        }
                    }
                </div>
            }
        }
        
        view_grammar(self.root)
    }


}

fn main() {
    yew::start_app::<Model>();
}
