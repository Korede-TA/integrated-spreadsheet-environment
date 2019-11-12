use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew::virtual_dom::{VList};
use std::collections::HashMap;
use std::num::NonZeroU32;
#[macro_use] extern crate maplit;

type Color = String;

// #[derive(Debug)]
#[derive(Debug, Clone)]
struct Borders {
    color : Color,
    collapse: bool
    // Add width for the border
    // width: 
    
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

    // Will get and return actual width
    // fn get_width(arg: Type) -> RetType {
    //     unimplemented!();
    // }
    // Will update the current width of the borders
    // fn set_width(arg: Type) -> RetType {
    //     unimplemented!();
    // }
}

#[derive(Debug, Clone)]
struct Font {
    weight: i32, 
    color: Color, 
}

#[derive(Debug, Clone)]
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
        format!{
        "border: 1px solid {};
        border-collapse: {};
        font-weight: {};
        color: {};",
        self.borders.color,
        self.borders.collapse,
        self.font.weight,
        self.font.color,
        }
    }
}

// Kinds of grammars in the system
#[derive(Debug, Clone)]
enum Kind {
    Text(String),
    Input(String),
    Grid(Vec<(NonZeroU32, NonZeroU32)>),
}

#[derive(Debug, Clone)]
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

// Model
struct Model {
    root: Grammar,
    grammars: GrammarMap,
    count: i32,
    value: String,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
struct Coordinate {
    row_cols: Vec<(NonZeroU32, NonZeroU32)>,
}

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

impl Coordinate {
    fn root() -> Coordinate {
        Coordinate{ row_cols: row_col_vec![(1, 1)] }
    }

    fn child_of(parent: &Self, child_coord: (NonZeroU32, NonZeroU32)) -> Coordinate {
        let mut new_row_col = parent.clone().row_cols;
        new_row_col.push(child_coord);
        Coordinate{ row_cols: new_row_col }
    }
}

fn non_zero_u32_tuple(val: (u32, u32)) -> (NonZeroU32, NonZeroU32) {
    let (row, col) = val;
    (NonZeroU32::new(row).unwrap(), NonZeroU32::new(col).unwrap())
}

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

macro_rules! ROOT {
    () => ( coord!{ (1,1) } );
}

macro_rules! META {
    () => ( coord!{ (1,2) } );
}

type GrammarMap = HashMap<Coordinate, Grammar>;

enum NavEvent {
   ScrollTo(Coordinate),
   
}

enum SelectEvent {
    SelectRange(Coordinate /* top-left */ , Coordinate /* bottom-right */),
}

enum DataEvent {
    ChangeInput(Coordinate, String /* new value */),
}

enum StructureEvent {
    AddNestedTable(Coordinate, Vec<(NonZeroU32, NonZeroU32)>),
    InsertGrammar(Coordinate, Grammar),
}

enum CompleteEvent {
    ShowDropdown(Coordinate),
    RefineSearch(String),
    HideDropdown,
}

enum AdminEvent {
    ShowDropdown(Coordinate),
    HideDropdown,
}

enum EventType {
    Nav(NavEvent),
    Select(SelectEvent),
    Data(DataEvent),
    Structure(StructureEvent),
    Complete(CompleteEvent),
    Admin(AdminEvent),
    Noop,
    Increment 
}

fn update_nav(model: &mut Model, nav_event: NavEvent) -> ShouldRender {
    false
}

fn update_select(model: &mut Model, select_event: SelectEvent) -> ShouldRender {
    false
}

fn update_data(model: &mut Model, data_event: DataEvent) -> ShouldRender {
    match data_event {
        DataEvent::ChangeInput(_coord, value) => {
            model.value = value.clone();
            true
        }
        _ => false
    }
}

fn update_structure(model: &mut Model, structure_event: StructureEvent) -> ShouldRender {
    false
}

fn update_complete(model: &mut Model, complete_event: CompleteEvent) -> ShouldRender {
    false
}

fn update_admin(model: &mut Model, admin_event: AdminEvent) -> ShouldRender {
    false
}

impl Component for Model {
    type Message = EventType;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let root_grammar = Grammar {
            name: "root".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![ (1,1), (1,2), (2,1), (2,2)]),
        };
        Model {
            root: root_grammar.clone(),
            grammars: hashmap! {
                ROOT!{} => root_grammar.clone(),
                coord!{ ROOT!{}; (1,1) } => Grammar::default(),
                coord!{ ROOT!{}; (1,2) } => Grammar::default(),
                coord!{ ROOT!{}; (2,1) } => Grammar::default(),
                coord!{ ROOT!{}; (2,2) } => Grammar::default(),
            },
            count: 0,
            value: String::new(),
        }
    }

    fn update(&mut self, event_type: Self::Message) -> ShouldRender {
        match event_type{
            EventType::Noop => {
                // Update your model on events
                true
            }
            EventType::Increment => {
                self.count+=1;
                true
            }
            EventType::Nav(nav_event) => { update_nav(self, nav_event) }
            EventType::Select(select_event) => { update_select(self, select_event) }
            EventType::Data(data_event) => { update_data(self, data_event) }
            EventType::Structure(structure_event) => { update_structure(self, structure_event) }
            EventType::Complete(complete_event) => { update_complete(self, complete_event) }
            EventType::Admin(admin_event) => { update_admin(self, admin_event) }
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
                            html! {
                                <input 
                                    style=&grammar.map(|g| g.style.to_string()).unwrap_or_default()
                                    value=value
                                    oninput=|e| {
                                        EventType::Data(DataEvent::ChangeInput(
                                                full_coord.clone(),
                                                e.value,
                                        ))
                                    }>
                                </input>
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
                // Render your model here
                <button onclick=|_| EventType::Increment>{ "Increment!" }</button>
                <p>{ self.count }</p>

                // <input oninput=|e| Msg::ChangeCellValue(e.value)>
                // </input>
                <p>{ self.value.clone() }</p>

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
