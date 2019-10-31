use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew::virtual_dom::{VList};

type Color = String;

// #[derive(Debug)]
#[derive(Debug)]
struct Borders {
    color : Color,
    collapse: bool
}

impl Borders {
    fn all(color : Color) -> Borders {
        Borders {
            color: color,
            collapse: false,
        }
    }

    fn set_color(&mut self, color : Color) {
        self.color = color;
    }
}

#[derive(Debug)]
struct Font {
    weight: i32, 
    color: Color, 
}

#[derive(Debug)]
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
#[derive(Debug)]
enum Kind {
    Text(String),
    Input(String),
    Table(Vec<Vec<Grammar>>),
}

#[derive(Debug)]
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
            kind: Kind::Input("".to_string()),
        }
    }

}

// Model
struct Model {
    root: Grammar,
    count: i32,
    value: String,
}

type Coordinate = Vec<(i32, i32)>;

enum Msg {
  ChangeCellValue(String),
  AddNestedTable(Coordinate),
  AutoCompeleteGrammar(Coordinate, Grammar),
  ToggleContextMenu(Coordinate, bool),
  Noop,
  Increment,
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
                    vec! [ Grammar::default(), Grammar::default(), Grammar::default(), ],
                    vec! [ Grammar::default(), Grammar::default(), Grammar::default(), ],
                    vec! [ Grammar::default(), Grammar::default(), Grammar::default(), ],
                ]),
            },
            count: 0,
            value: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Noop => {
                // Update your model on events
                true
            }
            Msg::Increment => {
                self.count+=1;
                true
            }
            Msg::ChangeCellValue(value) => {
                self.value = value.clone();
                true
            }
            _ => false
        }
    }

    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <h1>{ "Integrated Spreasheet Environment!" }</h1>
                // Render your model here
                <button onclick=|_| Msg::Increment>{ "Increment!" }</button>
                <p>{ self.count }</p>

                <input oninput=|e| Msg::ChangeCellValue(e.value)>
                </input>
                <p>{ self.value.clone() }</p>

            </div>
        }
    }


}

fn main() {
    yew::start_app::<Model>();
}
