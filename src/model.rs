use pest::Parser;
use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;
use std::panic;
use yew::events::{ClickEvent, IKeyboardEvent, KeyPressEvent};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::services::ConsoleService;
use yew::virtual_dom::VList;
use yew::{html, ChangeData, Component, ComponentLink, Html, InputData, ShouldRender};

use crate::coordinate::{Col, Coordinate, Row};
use crate::grammar::{Grammar, Interactive, Kind};
use crate::session::Session;
use crate::style::Style;
use crate::util::{
    apply_definition_grammar, move_grammar, non_zero_u32_tuple, resize, resize_cells,
};
use crate::view::{view_grammar, view_menu_bar, view_side_nav, view_tab_bar};
use crate::{coord, coord_col, coord_row, row_col_vec};

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

// Model contains the entire state of the application
#[derive(Debug)]
pub struct Model {
    // model holds a direct reference to the topmost root A1 and meta A2 grammars
    // these two grammars are excluded from the grammar Map
    root: Grammar,
    meta: Grammar,

    // the view that the UI treats as the topmost grammar to start rendering from.
    view_root: Coordinate,
    pub first_select_cell: Option<Coordinate>,
    pub last_select_cell: Option<Coordinate>,
    pub min_select_cell: Option<Coordinate>,
    pub max_select_cell: Option<Coordinate>,

    pub grammars: HashMap</*Key*/ Coordinate, /*Value*/ Grammar>,
    value: String, // what is this?????
    pub active_cell: Option<Coordinate>,
    pub suggestions: Vec<Coordinate>,
    pub col_widths: HashMap<Col, f64>,
    pub row_heights: HashMap<Row, f64>,

    pub select_grammar: Vec<Coordinate>,

    // tabs correspond to sessions
    pub tabs: Vec<String>,
    pub current_tab: i32,

    // side menus
    pub side_menus: Vec<SideMenu>,
    pub open_side_menu: Option<i32>,

    console: ConsoleService,
    reader: ReaderService,

    pub link: ComponentLink<Model>,
    tasks: Vec<ReaderTask>,
}

#[derive(Debug)]
pub struct SideMenu {
    pub name: String,
    pub icon_path: String,
}

// ACTIONS
// Triggered in the view, sent to update function
pub enum Action {
    // Do nothing
    Noop,

    // Change string value of Input grammar
    ChangeInput(Coordinate, /* new_value: */ String),

    // Show suggestions dropdown at Coordinate based on query
    ShowSuggestions(Coordinate, /* query: */ String),

    SetActiveCell(Coordinate),

    DoCompletion(
        /* source: */ Coordinate,
        /* destination */ Coordinate,
    ),

    SetActiveMenu(Option<i32>),

    ReadSession(/* filename: */ File),

    LoadSession(FileData),

    SaveSession(),

    // Grid Operations
    AddNestedGrid(Coordinate, (u32 /*rows*/, u32 /*cols*/)),

    InsertRow,
    InsertCol,

    // Alerts and stuff
    Alert(String),

    SetSelectedCells(Coordinate),

    MergeCells(),
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

    fn query_parent(&self, coord_parent: Coordinate) -> Vec<Coordinate> {
        self.grammars
            .keys()
            .clone()
            .filter_map(|k| {
                if k.parent() == Some(coord_parent.clone()) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn query_col(&self, coord_col: Col) -> Vec<Coordinate> {
        self.grammars
            .keys()
            .clone()
            .filter_map(|k| {
                if k.row_cols.len() == 1
                /* ignore root & meta */
                {
                    None
                } else if k.full_col() == coord_col {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn query_row(&self, coord_row: Row) -> Vec<Coordinate> {
        self.grammars
            .keys()
            .clone()
            .filter_map(|k| {
                if k.row_cols.len() == 1
                /* ignore root & meta */
                {
                    None
                } else if k.full_row() == coord_row {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Component for Model {
    type Message = Action;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let root_grammar = Grammar {
            name: "root".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)]),
        };
        let meta_grammar = Grammar {
            name: "meta".to_string(),
            style: Style::default(),
            kind: Kind::Grid(row_col_vec![(1, 1), (2, 1)]),
        };
        let mut m = Model {
            root: root_grammar.clone(),
            meta: meta_grammar.clone(),
            view_root: coord!("root"),
            grammars: hashmap! {
                coord!("root")    => root_grammar.clone(),
                coord!("root-A1") => Grammar::default(),
                coord!("root-A2") => Grammar::default(),
                coord!("root-A3") => Grammar::default(),
                coord!("root-B1") => Grammar::default(),
                coord!("root-B2") => Grammar::default(),
                coord!("root-B3") => Grammar::default(),
                coord!("meta")    => meta_grammar.clone(),
                coord!("meta-A1") => Grammar::suggestion("js grammar".to_string(), "This is js".to_string()),
                coord!("meta-A2") => Grammar::suggestion("java grammar".to_string(), "This is java".to_string()),
            },
            col_widths: hashmap! {
               coord_col!("root","A") => 90.0,
               coord_col!("root","B") => 90.0,
            },
            row_heights: hashmap! {
               coord_row!("root","1") => 30.0,
               coord_row!("root","2") => 30.0,
               coord_row!("root","3") => 30.0,
            },
            value: String::new(),
            active_cell: Some(coord!("root-A1")),
            suggestions: vec![coord!("meta-A1"), coord!("meta-A2"), coord!("meta-A3")],
            // suggestions: vec![],
            console: ConsoleService::new(),
            reader: ReaderService::new(),

            select_grammar: vec![],
            first_select_cell: None,
            last_select_cell: None,
            min_select_cell: None,
            max_select_cell: None,

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
        };
        apply_definition_grammar(&mut m, coord!("meta-A3"));
        m
    }

    // The update function is split into sub-update functions that
    // are specifc to each EventType
    fn update(&mut self, event_type: Self::Message) -> ShouldRender {
        match event_type {
            Action::Noop => false,

            Action::Alert(message) => {
                self.console.log(&message);
                // TODO: make this into a more visual thing
                false
            }

            Action::ChangeInput(coord, new_value) => {
                let old_grammar = self.grammars.get_mut(&coord);
                match old_grammar {
                    Some(
                        g @ Grammar {
                            kind: Kind::Text(_),
                            ..
                        },
                    ) => {
                        self.console.log(&new_value);
                        g.kind = Kind::Text(new_value);
                    }
                    _ => (),
                }
                false
            }

            Action::ShowSuggestions(coord, query) => false,

            Action::SetActiveCell(coord) => {
                self.first_select_cell = Some(coord.clone());
                self.last_select_cell = None;
                self.min_select_cell = None;
                self.max_select_cell = None;
                self.active_cell = Some(coord.clone());
                true
            }

            Action::SetSelectedCells(coord) => {
                self.last_select_cell = Some(coord.clone());
                if self.first_select_cell.is_none() || self.last_select_cell.is_none() {
                    return false;
                }
                let mut first_select_row = NonZeroU32::new(1).unwrap();
                let mut first_select_col = NonZeroU32::new(1).unwrap();
                let mut last_select_row = NonZeroU32::new(1).unwrap();
                let mut last_select_col = NonZeroU32::new(1).unwrap();

                let mut min_select_row = NonZeroU32::new(1).unwrap();
                let mut max_select_row = NonZeroU32::new(1).unwrap();
                let mut min_select_col = NonZeroU32::new(1).unwrap();
                let mut max_select_col = NonZeroU32::new(1).unwrap();
                first_select_row = self.first_select_cell.as_ref().unwrap().row();
                first_select_col = self.first_select_cell.as_ref().unwrap().col();
                last_select_row = self.last_select_cell.as_ref().unwrap().row();
                last_select_col = self.last_select_cell.as_ref().unwrap().col();
                if first_select_row < last_select_row {
                    min_select_row = first_select_row;
                    max_select_row = last_select_row;
                } else {
                    min_select_row = last_select_row;
                    max_select_row = first_select_row;
                }
                if first_select_col < last_select_col {
                    min_select_col = first_select_col;
                    max_select_col = last_select_col;
                } else {
                    min_select_col = last_select_col;
                    max_select_col = first_select_col;
                }

                let ref_grammas = self.grammars.clone();
                for (coord, grammar) in &ref_grammas {
                    if min_select_row <= coord.row()
                        && coord.row() <= max_select_row
                        && min_select_col <= coord.col()
                        && coord.col() <= max_select_col
                        && coord.to_string().contains("root-")
                    {
                        let col_span = grammar.style.col_span;
                        let row_span = grammar.style.row_span;
                        if col_span[0] > 0 && col_span[1] > 0 {
                            if col_span[0] < min_select_col.get() {
                                min_select_col = NonZeroU32::new(col_span[0]).unwrap();
                            }
                            if col_span[1] > max_select_col.get() {
                                max_select_col = NonZeroU32::new(col_span[1]).unwrap();
                            }
                        }
                        if row_span[0] > 0 && row_span[1] > 0 {
                            if row_span[0] < min_select_row.get() {
                                min_select_row = NonZeroU32::new(row_span[0]).unwrap();
                            }
                            if row_span[1] > max_select_row.get() {
                                max_select_row = NonZeroU32::new(row_span[1]).unwrap();
                            }
                        }
                    }
                }

                self.min_select_cell = Some(Coordinate {
                    row_cols: vec![(min_select_row, min_select_col)],
                });
                self.max_select_cell = Some(Coordinate {
                    row_cols: vec![(max_select_row, max_select_col)],
                });
                true
            }

            Action::DoCompletion(source_coord, dest_coord) => {
                move_grammar(&mut self.grammars, source_coord, dest_coord.clone());
                resize_cells(&mut self.grammars, dest_coord);
                true
            }

            Action::SetActiveMenu(active_menu) => {
                self.open_side_menu = active_menu;
                true
            }

            Action::ReadSession(file) => {
                let callback = self.link.callback(Action::LoadSession);
                let task = self.reader.read_file(file, callback);
                self.tasks.push(task);
                false
            }

            Action::LoadSession(file_data) => {
                let session: Session =
                    serde_json::from_str(format! {"{:?}", file_data}.deref()).unwrap();
                self.load_session(session);
                true
            }

            Action::SaveSession() => {
                let session = self.to_session();
                let j = serde_json::to_string(&session);
                let filename = "testfile";
                fs::write(filename, j.unwrap()).expect("Unable to write to file!");
                false
            }

            Action::MergeCells() => {
                if self.min_select_cell.is_none() || self.max_select_cell.is_none() {
                    return false;
                }
                let mut min_select_row = self.min_select_cell.as_ref().unwrap().row();
                let mut max_select_row = self.max_select_cell.as_ref().unwrap().row();
                let mut min_select_col = self.min_select_cell.as_ref().unwrap().col();
                let mut max_select_col = self.max_select_cell.as_ref().unwrap().col();
                let mut merge_height = 0.00;
                let mut merge_width = 0.00;
                let mut max_coord = Coordinate::default();
                let mut max_grammar = Grammar::default();
                let mut ref_grammas = self.grammars.clone();
                for (coord, grammar) in ref_grammas.iter_mut() {
                    if min_select_row <= coord.row()
                        && coord.row() <= max_select_row
                        && min_select_col <= coord.col()
                        && coord.col() <= max_select_col
                        && coord.to_string().contains("root-")
                        && grammar.style.display == true
                    {
                        let coord_style = grammar.style.clone();
                        if (coord.row() == max_select_row) && (coord.col() == max_select_col) {
                            merge_width = merge_width + coord_style.width;
                            merge_height = merge_height + coord_style.height;
                            max_coord = coord.clone();
                            max_grammar = grammar.clone();
                            continue;
                        } else if (coord.row() == max_select_row) {
                            merge_width = merge_width + coord_style.width;
                        } else if coord.col() == max_select_col {
                            merge_height = merge_height + coord_style.height;
                        }
                        if (coord.row() != max_select_row) || (coord.col() != max_select_col) {
                            grammar.style.display = false;
                            merge_height = merge_height;
                            merge_width = merge_width;
                        }
                        grammar.style.col_span[0] = min_select_col.get();
                        grammar.style.col_span[1] = max_select_col.get();
                        grammar.style.row_span[0] = min_select_row.get();
                        grammar.style.row_span[1] = max_select_row.get();
                        self.grammars.insert(coord.clone(), grammar.clone());
                    }
                }
                max_grammar.style.width = merge_width;
                max_grammar.style.height = merge_height;
                max_grammar.style.col_span[0] = min_select_col.get();
                max_grammar.style.col_span[1] = max_select_col.get();
                max_grammar.style.row_span[0] = min_select_row.get();
                max_grammar.style.row_span[1] = max_select_row.get();
                self.grammars.insert(max_coord, max_grammar);
                self.min_select_cell = None;
                self.max_select_cell = None;
                true
            }

            Action::AddNestedGrid(coord, (rows, cols)) => {
                let (r, c) = non_zero_u32_tuple((rows, cols));
                let grammar = Grammar::as_grid(r, c);
                if let Kind::Grid(sub_coords) = grammar.clone().kind {
                    self.active_cell = sub_coords.first().map(|c| Coordinate::child_of(&coord, *c));
                    for sub_coord in sub_coords {
                        let new_coord = Coordinate::child_of(&coord, sub_coord);
                        self.grammars.insert(new_coord.clone(), Grammar::default());
                        // initialize row & col heights as well
                        if !self.row_heights.contains_key(&new_coord.clone().full_row()) {
                            self.row_heights.insert(new_coord.clone().full_row(), 30.0);
                        }
                        if !self.col_widths.contains_key(&new_coord.clone().full_col()) {
                            self.col_widths.insert(new_coord.clone().full_col(), 90.0);
                        }
                    }
                }
                if let Some(parent) =
                    Coordinate::parent(&coord).and_then(|p| self.grammars.get_mut(&p))
                {
                    parent.kind = grammar.clone().kind; // make sure the parent gets set to Kind::Grid
                }
                self.grammars.insert(coord.clone(), grammar);
                resize(
                    self,
                    coord,
                    (rows as f64) * (/* default row height */30.0),
                    (cols as f64) * (/* default col width */90.0),
                );
                true
            }
            Action::InsertCol => {
                if let Some(coord) = self.active_cell.clone() {
                    // find the bottom-most coord
                    let mut right_most_coord = coord.clone();
                    while let Some(right_coord) = right_most_coord.neighbor_right() {
                        if self.grammars.contains_key(&right_coord) {
                            right_most_coord = right_coord;
                        } else {
                            break;
                        }
                    }

                    let right_most_col_coords = self.query_col(right_most_coord.full_col());
                    let new_col_coords = right_most_col_coords
                        .iter()
                        .map(|c| (c.row(), NonZeroU32::new(c.col().get() + 1).unwrap()));

                    let parent = coord.parent().unwrap();
                    if let Some(Grammar {
                        kind: Kind::Grid(sub_coords),
                        name,
                        style,
                    }) = self.grammars.get(&parent)
                    {
                        let mut new_sub_coords = sub_coords.clone();
                        let mut grammars = self.grammars.clone();
                        for c in new_col_coords {
                            grammars.insert(
                                Coordinate::child_of(&parent.clone(), c),
                                Grammar::default(),
                            );
                            new_sub_coords.push(c);
                        }
                        grammars.insert(
                            parent,
                            Grammar {
                                kind: Kind::Grid(new_sub_coords.clone()),
                                name: name.clone(),
                                style: style.clone(),
                            },
                        );
                        self.grammars = grammars;
                    }
                }
                true
            }
            Action::InsertRow => {
                if let Some(coord) = self.active_cell.clone() {
                    // find the bottom-most coord
                    let mut bottom_most_coord = coord.clone();
                    while let Some(below_coord) = bottom_most_coord.neighbor_below() {
                        if self.grammars.contains_key(&below_coord) {
                            bottom_most_coord = below_coord;
                        } else {
                            break;
                        }
                    }

                    let bottom_most_row_coords = self.query_row(bottom_most_coord.full_row());
                    let new_row_coords = bottom_most_row_coords
                        .iter()
                        .map(|c| (NonZeroU32::new(c.row().get() + 1).unwrap(), c.col()));

                    let parent = coord.parent().unwrap();
                    if let Some(Grammar {
                        kind: Kind::Grid(sub_coords),
                        name,
                        style,
                    }) = self.grammars.get(&parent)
                    {
                        let mut new_sub_coords = sub_coords.clone();
                        let mut grammars = self.grammars.clone();
                        for c in new_row_coords {
                            grammars.insert(
                                Coordinate::child_of(&parent.clone(), c),
                                Grammar::default(),
                            );
                            new_sub_coords.push(c);
                        }
                        grammars.insert(
                            parent,
                            Grammar {
                                kind: Kind::Grid(new_sub_coords.clone()),
                                name: name.clone(),
                                style: style.clone(),
                            },
                        );
                        self.grammars = grammars;
                    }
                }
                true
            }
        }
    }

    fn view(&self) -> Html {
        let active_cell = self.active_cell.clone();
        html! {
            <div>

                { view_side_nav(&self) }

                { view_menu_bar(&self) }

                { view_tab_bar(&self) }

                <div class="main">
                    <div id="grammars" class="grid-wrapper" onkeypress=self.link.callback(move |e : KeyPressEvent| {
                        if e.key() == "g" && e.ctrl_key() {
                            if let Some(coord) = active_cell.clone() {
                                return Action::AddNestedGrid(coord.clone(), (3, 3));
                            }
                        }
                        Action::Noop
                    })>
                        { view_grammar(&self, coord!{"root"}) }
                    </div>
                </div>
            </div>
        }
    }
}
