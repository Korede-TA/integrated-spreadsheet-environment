use electron_sys::ipc_renderer;
use pest::Parser;
use std::collections::HashMap;

use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;
use stdweb::unstable::TryInto;
use stdweb::web::{document, IElement, INode, IParentNode};
use wasm_bindgen::JsValue;
use yew::events::KeyPressEvent;
use yew::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::services::ConsoleService;

use crate::coordinate::{Col, Coordinate, Row};
use crate::grammar::{Grammar, Kind, Lookup};
use crate::session::Session;
use crate::style::Style;
use crate::util::{move_grammar, non_zero_u32_tuple, resize, resize_diff};
use crate::view::{view_grammar, view_menu_bar, view_side_nav, view_tab_bar};
use crate::{coord, coord_col, coord_row, row_col_vec};

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

// Model contains the entire state of the application
#[derive(Debug)]
pub struct Model {
    view_root: Coordinate,
    pub first_select_cell: Option<Coordinate>,
    pub last_select_cell: Option<Coordinate>,
    pub min_select_cell: Option<Coordinate>,
    pub max_select_cell: Option<Coordinate>,

    pub grammars: HashMap</*Key*/ Coordinate, /*Value*/ Grammar>,
    value: String, // what is this?????
    pub active_cell: Option<Coordinate>,
    pub zoom: f32,
    pub default_suggestions: Vec<Coordinate>,
    pub suggestions: HashMap<Coordinate, Vec<Coordinate>>,
    pub col_widths: HashMap<Col, f64>,
    pub row_heights: HashMap<Row, f64>,
    pub select_grammar: Vec<Coordinate>,
    pub sessions: Vec<Session>,
    pub current_session_index: usize,
    pub side_menus: Vec<SideMenu>,
    pub open_side_menu: Option<i32>,
    pub focus_node_ref: NodeRef,
    pub resizing: Option<Coordinate>,
    pub link: ComponentLink<Model>,
    console: ConsoleService,
    reader: ReaderService,
    tasks: Vec<ReaderTask>,
}

#[derive(Debug)]
pub struct SideMenu {
    pub name: String,
    pub icon_path: String,
}

pub enum ResizeMsg {
    Start(Coordinate),
    X(f64),
    Y(f64),
    End,
}

// ACTIONS
// Triggered in the view, sent to update function
pub enum Action {
    // Do nothing
    Noop,

    // Change string value of Input grammar
    ChangeInput(Coordinate, /* new_value: */ String),

    SetActiveCell(Coordinate),

    DoCompletion(
        /* source: */ Coordinate,
        /* destination */ Coordinate,
    ),

    SetActiveMenu(Option<i32>),

    ReadSession(/* filename: */ File),

    LoadSession(FileData),

    SaveSession(),

    SetSessionTitle(String),
    ReadDriverFiles(Vec<File>),
    LoadDriverMainFile(FileData),
    UploadDriverMiscFile(FileData),

    // Grid Operations
    AddNestedGrid(Coordinate, (u32 /*rows*/, u32 /*cols*/)),

    InsertRow,
    InsertCol,
    DeleteRow,
    DeleteCol,
    Recreate,
    ZoomIn,
    ZoomOut,
    ZoomReset,

    Resize(ResizeMsg),

    // Alerts and stuff
    Alert(String),

    SetSelectedCells(Coordinate),
    Lookup(
        /* source: */ Coordinate,
        /* lookup_type: */ Lookup,
    ),

    ToggleLookup(Coordinate),

    DefnUpdateName(Coordinate, /* name */ String),
    DefnUpdateRule(Coordinate, /* rule Row  */ Row),
    DefnAddRule(Coordinate), // adds a new column, points rule coordinate to bottom of ~meta~ sub-table
    // Definition Rules are represented as grammars
    MergeCells(),
}

impl Model {
    pub fn get_session(&self) -> &Session {
        &self.sessions[self.current_session_index]
    }

    pub fn get_session_mut(&mut self) -> &mut Session {
        &mut self.sessions[self.current_session_index]
    }

    // only use this if you need a COPY of the current session
    // i.e. not changing its values
    pub fn to_session(&self) -> Session {
        self.get_session().clone()
    }

    fn load_session(&mut self, session: Session) {
        self.get_session_mut().root = session.root;
        self.get_session_mut().meta = session.meta;
        self.get_session_mut().grammars = session.grammars;
    }

    fn query_parent(&self, coord_parent: Coordinate) -> Vec<Coordinate> {
        self.get_session()
            .grammars
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
        self.get_session()
            .grammars
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
        self.get_session()
            .grammars
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
            kind: Kind::Grid(row_col_vec![(1, 1), (2, 1), (3, 1)]),
        };
        let m = Model {
            view_root: coord!("root"),
            col_widths: hashmap! {
               coord_col!("root","A") => 90.0,
               coord_col!("root","B") => 90.0,
               coord_col!("meta","A") => 180.0,
               coord_col!("meta-A3","A") => 90.0,
               coord_col!("meta-A3","B") => 180.0,
            },
            row_heights: hashmap! {
               coord_row!("root","1") => 30.0,
               coord_row!("root","2") => 30.0,
               coord_row!("root","3") => 30.0,
               coord_row!("meta","1") => 180.0,
            },
            active_cell: Some(coord!("root-A1")),
            default_suggestions: vec![coord!("meta-A1"), coord!("meta-A2"), coord!("meta-A3")],
            suggestions: HashMap::new(),

            console: ConsoleService::new(),
            reader: ReaderService::new(),

            select_grammar: vec![],
            first_select_cell: None,
            last_select_cell: None,
            min_select_cell: None,
            max_select_cell: None,
            zoom: 1.0,

            sessions: vec![Session {
                title: "my session".to_string(),
                root: root_grammar.clone(),
                meta: meta_grammar.clone(),
                grammars: hashmap! {
                    coord!("root")    => root_grammar.clone(),
                    coord!("root-A1") => Grammar::default(),
                    coord!("root-A2") => Grammar::default(),
                    coord!("root-A3") => Grammar::default(),
                    coord!("root-B1") => Grammar::default(),
                    coord!("root-B2") => Grammar::default(),
                    coord!("root-B3") => Grammar::default(),
                    coord!("meta")    => meta_grammar.clone(),
                    coord!("meta-A1") => Grammar::text("js grammar".to_string(), "This is js".to_string()),
                    coord!("meta-A2") => Grammar::text("java grammar".to_string(), "This is java".to_string()),
                    coord!("meta-A3") => Grammar {
                        name: "defn".to_string(),
                        style: Style::default(),
                        kind: Kind::Defn(
                            "".to_string(),
                            coord!("meta-A3"),
                            vec![
                                ("".to_string(), coord!("meta-A3-B1")),
                            ],
                        ),
                    },
                    coord!("meta-A3-A1")    => Grammar::default(),
                    coord!("meta-A3-B1")    => Grammar {
                        name: "root".to_string(),
                        style: Style::default(),
                        kind: Kind::Grid(row_col_vec![ (1,1), (2,1), (1,2), (2,2) ]),
                    },
                    coord!("meta-A3-B1-A1") => Grammar::input("".to_string(), "sub-grammar name".to_string()),
                    coord!("meta-A3-B1-B1") => Grammar::text("".to_string(), "+".to_string()),
                    coord!("meta-A3-B1-C1") => Grammar::default(),
                },
            }],

            current_session_index: 0,

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

            resizing: None,

            link,
            tasks: vec![],

            focus_node_ref: NodeRef::default(),
        };
        // apply_definition_grammar(&mut m, coord!("meta-A3"));
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
                if let Some(g) = self.get_session_mut().grammars.get_mut(&coord) {
                    match g {
                        Grammar {
                            kind: Kind::Input(_),
                            ..
                        } => {
                            info!("{}", &new_value);
                            g.kind = Kind::Input(new_value);
                        }
                        Grammar {
                            kind: Kind::Lookup(_, lookup_type),
                            ..
                        } => {
                            info!("{}", &new_value);
                            g.kind = Kind::Lookup(new_value, lookup_type.clone());
                        }
                        _ => (),
                    }
                }
                false
            }

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
                move_grammar(
                    &mut self.get_session_mut().grammars,
                    source_coord,
                    dest_coord.clone(),
                );
                let row_height = self.row_heights.get(&dest_coord.full_row()).unwrap();
                let col_width = self.col_widths.get(&dest_coord.full_col()).unwrap();
                resize(self, dest_coord, *row_height, *col_width);
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
                /* TODO: uncomment when this is working
                use node_sys::fs as node_fs;
                use node_sys::Buffer;
                use js_sys::{
                    JsString,
                    Function
                };
                let session = self.to_session();
                let j = serde_json::to_string(&session.clone());
                let filename = session.title.to_string();
                let jsfilename = JsString::from(filename);
                let jsbuffer = Buffer::from_string(&JsString::from(j.unwrap()), None);
                let jscallback = Function::new_no_args("{}");
                node_fs::append_file(&jsfilename, &jsbuffer, None, &jscallback);
                */
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
                // height and width initial value
                let mut tmp_heigt = 30.0;
                let mut tmp_width = 90.0;
                let (r, c) = non_zero_u32_tuple((rows, cols));
                let grammar = Grammar::as_grid(r, c);
                if let Kind::Grid(sub_coords) = grammar.clone().kind {
                    self.active_cell = sub_coords.first().map(|c| Coordinate::child_of(&coord, *c));
                    // let row_val = coord

                    let current_width = self.col_widths[&coord.full_col()];
                    let current_height = self.row_heights[&coord.full_row()];

                    // check if active cell row height and width is greater than default value
                    if current_width > tmp_width {
                        // set height argument to active cell height if greater
                        //Get the actual amount of cell being created and use it instead of "3" being HARD CODED.
                        tmp_width = current_width / 3.0;
                    }
                    if current_height > tmp_heigt {
                        // set width argument to active cell width if greater
                        //Get the actual amount of cell being created and use it instead of "3" being HARD CODED.
                        tmp_heigt = current_height / 3.0;
                    }

                    for sub_coord in sub_coords {
                        let new_coord = Coordinate::child_of(&coord, sub_coord);
                        self.get_session_mut()
                            .grammars
                            .insert(new_coord.clone(), Grammar::default());
                        // initialize row & col heights as well
                        if !self.row_heights.contains_key(&new_coord.clone().full_row()) {
                            self.row_heights
                                .insert(new_coord.clone().full_row(), tmp_heigt);
                            //30.0);
                        }
                        if !self.col_widths.contains_key(&new_coord.clone().full_col()) {
                            self.col_widths
                                .insert(new_coord.clone().full_col(), tmp_width);
                            //90.0);
                        }
                    }
                }
                if let Some(parent) = Coordinate::parent(&coord)
                    .and_then(|p| self.get_session_mut().grammars.get_mut(&p))
                {
                    parent.kind = grammar.clone().kind; // make sure the parent gets set to Kind::Grid
                }
                self.get_session_mut()
                    .grammars
                    .insert(coord.clone(), grammar);
                resize(
                    self,
                    coord,
                    (rows as f64) * (/* default row height */tmp_heigt),
                    (cols as f64) * (/* default col width */tmp_width),
                );
                true
            }

            Action::ZoomIn => {
                self.zoom += 0.1;
                true
            }
            Action::ZoomReset => {
                self.zoom = 1.0;
                true
            }

            Action::ZoomOut => {
                self.zoom -= 0.1;
                true
            }

            Action::InsertCol => {
                if let Some(coord) = self.active_cell.clone() {
                    // find the bottom-most coord
                    let mut right_most_coord = coord.clone();
                    while let Some(right_coord) = right_most_coord.neighbor_right() {
                        if self.get_session_mut().grammars.contains_key(&right_coord) {
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
                    }) = self.to_session().grammars.get(&parent)
                    {
                        let mut new_sub_coords = sub_coords.clone();
                        let mut grammars = self.get_session_mut().grammars.clone();
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
                        self.get_session_mut().grammars = grammars;
                    }
                }
                true
            }
            Action::InsertRow => {
                if let Some(coord) = self.active_cell.clone() {
                    // find the bottom-most coord
                    let mut bottom_most_coord = coord.clone();
                    while let Some(below_coord) = bottom_most_coord.neighbor_below() {
                        if self.get_session().grammars.contains_key(&below_coord) {
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
                    }) = self.to_session().grammars.get(&parent)
                    {
                        let mut new_sub_coords = sub_coords.clone();

                        let mut grammars = self.get_session_mut().grammars.clone();
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
                        self.get_session_mut().grammars = grammars;
                    }
                }
                true
            }
            Action::DeleteRow => {
                //Taking Active cell
                if let Some(coord) = self.active_cell.clone() {
                    //Have to initialize many things for them to work in loop
                    let mut next_row = coord.clone();
                    let mut grammars = self.get_session_mut().grammars.clone();
                    let mut row_coords1 = self.query_row(next_row.full_row());
                    let _parent = coord.parent().unwrap();

                    let mut temp: Vec<Grammar> = vec![];
                    let mut u = 0;
                    let mut row_coords2 = self.query_row(next_row.full_row());
                    let mut new_row_coords: std::vec::Vec<(
                        std::num::NonZeroU32,
                        std::num::NonZeroU32,
                    )> = vec![];

                    //Changing each rowfrom the one being deleted
                    while let Some(below_coord) = next_row.neighbor_below() {
                        temp.clear();
                        row_coords2 = self.query_row(below_coord.full_row());
                        if row_coords2.len() != 0 {
                            temp = std::vec::from_elem(
                                grammars[&row_coords2[0]].clone(),
                                row_coords2.len(),
                            );

                            //each grammar copied
                            for i in row_coords2.clone() {
                                u = i.col().get() as usize;
                                temp.insert(u, grammars[&i].clone());
                            }
                            u = 0;
                        }

                        if temp.len() == 0 {
                            let parent = next_row.parent().unwrap();
                            if let Some(Grammar {
                                kind: Kind::Grid(sub_coords),
                                name,
                                style,
                            }) = self.to_session().grammars.get(&parent)
                            {
                                new_row_coords = sub_coords.clone();

                                for c in row_coords1.clone() {
                                    for i in (0..new_row_coords.len()).rev() {
                                        if new_row_coords[i] == (c.row(), c.col()) {
                                            new_row_coords.remove(i);
                                            grammars.remove(&Coordinate::child_of(
                                                &parent.clone(),
                                                (c.row(), c.col()),
                                            ));
                                        }
                                    }
                                }
                                grammars.remove(&parent);
                                grammars.remove(&next_row);
                                grammars.insert(
                                    parent,
                                    Grammar {
                                        kind: Kind::Grid(new_row_coords.clone()),
                                        name: name.clone(),
                                        style: style.clone(),
                                    },
                                );
                                break;
                            }
                        } else {
                            // info!("XD {:?}", row_coords1);
                            // info!("Temp {:?}", temp);
                            for c in (0..row_coords1.len()).rev() {
                                grammars.insert(row_coords1[c].clone(), temp[u].clone());
                                u += 1;
                            }
                            u = 0;
                        }

                        row_coords1 = row_coords2.clone();
                        next_row = below_coord;
                    }
                    self.get_session_mut().grammars = grammars;
                }
                true
            }
            Action::DeleteCol => {
                //Taking Active cell
                if let Some(coord) = self.active_cell.clone() {
                    //Have to initialize many things for them to work in loop
                    let mut next_col = coord.clone();
                    let mut grammars = self.get_session_mut().grammars.clone();
                    let mut col_coords1 = self.query_col(next_col.full_col());
                    let parent = coord.parent().unwrap();

                    let mut temp: Vec<Grammar> = vec![];
                    let mut u = 0;
                    let mut col_coords2 = self.query_col(next_col.full_col());
                    let mut new_col_coords: std::vec::Vec<(
                        std::num::NonZeroU32,
                        std::num::NonZeroU32,
                    )> = vec![];
                    if let Some(Grammar {
                        kind: Kind::Grid(sub_coords),
                        name: _,
                        style: _,
                    }) = self.get_session_mut().grammars.get(&parent)
                    {
                        let _new_col_coords = sub_coords.clone();
                    }

                    //Changing each colfrom the one being deleted
                    while let Some(right_coord) = next_col.neighbor_right() {
                        temp.clear();
                        col_coords2 = self.query_col(right_coord.full_col());
                        if col_coords2.len() != 0 {
                            temp = std::vec::from_elem(
                                grammars[&col_coords2[0]].clone(),
                                col_coords2.len(),
                            );

                            //each grammar copied
                            for i in col_coords2.clone() {
                                u = i.col().get() as usize;
                                temp.insert(u, grammars[&i].clone());
                            }
                            info!("{:?}", temp);
                            u = 0;
                        }
                        if temp.len() == 0 {
                            let parent = next_col.parent().unwrap();
                            if let Some(Grammar {
                                kind: Kind::Grid(sub_coords),
                                name,
                                style,
                            }) = self.to_session().grammars.get(&parent)
                            {
                                new_col_coords = sub_coords.clone();

                                for c in col_coords1.clone() {
                                    for i in (0..new_col_coords.len()).rev() {
                                        if new_col_coords[i] == (c.row(), c.col()) {
                                            new_col_coords.remove(i);
                                            grammars.remove(&Coordinate::child_of(
                                                &parent.clone(),
                                                (c.row(), c.col()),
                                            ));
                                        }
                                    }
                                }
                                grammars.remove(&parent);
                                grammars.remove(&next_col);
                                grammars.insert(
                                    parent,
                                    Grammar {
                                        kind: Kind::Grid(new_col_coords.clone()),
                                        name: name.clone(),
                                        style: style.clone(),
                                    },
                                );
                                break;
                            }
                        } else {
                            for c in (0..col_coords1.len()).rev() {
                                grammars.insert(col_coords1[c].clone(), temp[u].clone());
                                u += 1;
                            }
                            u = 0;
                        }

                        col_coords1 = col_coords2.clone();
                        next_col = right_coord;
                    }
                    self.get_session_mut().grammars = grammars;
                }
                true
            }

            Action::Recreate => {
                self.get_session_mut().grammars = hashmap! {
                    coord!("root")    => self.get_session_mut().root.clone(),
                    coord!("root-A1") => Grammar::default(),
                    coord!("root-A2") => Grammar::default(),
                    coord!("root-A3") => Grammar::default(),
                    coord!("root-B1") => Grammar::default(),
                    coord!("root-B2") => Grammar::default(),
                    coord!("root-B3") => Grammar::default(),
                    coord!("meta")    => self.get_session_mut().meta.clone(),
                    coord!("meta-A1") => Grammar::text("js grammar".to_string(), "This is js".to_string()),
                    coord!("meta-A2") => Grammar::text("java grammar".to_string(), "This is java".to_string()),
                    coord!("meta-A3") => Grammar {
                        name: "defn".to_string(),
                        style: Style::default(),
                        kind: Kind::Defn(
                            "".to_string(),
                            coord!("meta-A3"),
                            vec![
                                ("".to_string(), coord!("meta-A3-B1")),
                            ],
                        ),
                    },
                    coord!("meta-A3-A1")    => Grammar::default(),
                    coord!("meta-A3-B1")    => Grammar {
                        name: "root".to_string(),
                        style: Style::default(),
                        kind: Kind::Grid(row_col_vec![ (1,1), (2,1), (1,2), (2,2) ]),
                    },
                    coord!("meta-A3-B1-A1") => Grammar::input("".to_string(), "sub-grammar name".to_string()),
                    coord!("meta-A3-B1-B1") => Grammar::text("".to_string(), "+".to_string()),
                    coord!("meta-A3-B1-C1") => Grammar::default(),
                };
                true
            }

            Action::Resize(msg) => {
                match msg {
                    ResizeMsg::Start(coord) => {
                        info! {"drag start"};
                        self.resizing = Some(coord);
                    }
                    ResizeMsg::X(offset_x) => {
                        if let Some(coord) = self.resizing.clone() {
                            info! {"drag x: {}", offset_x};
                            resize_diff(self, coord, 0.0, offset_x);
                        }
                    }
                    ResizeMsg::Y(offset_y) => {
                        if let Some(coord) = self.resizing.clone() {
                            info! {"drag y: {}", offset_y};
                            resize_diff(self, coord, offset_y, 0.0);
                        }
                    }
                    ResizeMsg::End => {
                        info! {"drag end"};
                        self.resizing = None;
                    }
                }
                true
            }

            Action::Lookup(source_coord, lookup_type) => {
                match lookup_type {
                    Lookup::Cell(dest_coord) => {
                        move_grammar(
                            &mut self.get_session_mut().grammars,
                            source_coord,
                            dest_coord.clone(),
                        );
                    }
                    _ => (),
                }
                false
            }
            Action::ToggleLookup(coord) => {
                match self.get_session_mut().grammars.get_mut(&coord) {
                    Some(
                        g @ Grammar {
                            kind: Kind::Input(_),
                            ..
                        },
                    ) => {
                        g.kind = Kind::Lookup("".to_string(), None);
                    }
                    Some(
                        g @ Grammar {
                            kind: Kind::Lookup(_, _),
                            ..
                        },
                    ) => {
                        g.kind = Kind::Input("".to_string());
                    }
                    _ => {
                        info! { "[Action::ToggleLookup] cannot toggle non-Input/Lookup kind of grammar" }
                    }
                };
                true
            }
            /*
             * The following actions determine how the "defn" grammar behaves. It serves three main
             * roles:
             * 1) Defining grammars to be suggested in the interface
             * 2) Specifying valid sub-grammars to be completed into various slots in the
             *    interface.
             * 3) Defining how grammars connect with respective drivers and have values evaluated
             *    and passed back to the interface.
             */
            Action::DefnUpdateName(coord, name) => {
                // updates the name of a new or existing grammar.
                let _defn_name_coord = Coordinate::child_of(&coord, non_zero_u32_tuple((1, 1)));
                if let Some(g) = self.get_session_mut().grammars.get_mut(&coord) {
                    match g {
                        Grammar {
                            kind: Kind::Input(_),
                            ..
                        } => {
                            info! {"updating defn name: {}", &name};
                            g.kind = Kind::Input(name);
                        }
                        _ => (),
                    }
                }
                true
            }
            Action::DefnUpdateRule(_coord, _rule_row) => {
                let _rule_row_coord = {};
                true
            }
            Action::DefnAddRule(_coord) => {
                // TODO adds a new column, points rule coordinate to bottom of ~meta~ sub-table
                false
            }
        }
    }

    fn view(&self) -> Html {
        let _active_cell = self.active_cell.clone();
        let is_resizing = self.resizing.is_some();
        let zoom = "zoom:".to_string() + &self.zoom.to_string();
        html! {
            <div>

                { view_side_nav(&self) }

                { view_menu_bar(&self) }

                { view_tab_bar(&self) }

                <div class="main">
                    <div id="grammars" class="grid-wrapper" style={zoom}
                        onkeypress=self.link.callback(move |e : KeyPressEvent| {
                            // Global Key-Shortcuts
                            Action::Noop
                        })
                        onmouseup=self.link.callback(move |e : MouseUpEvent| {
                            if is_resizing.clone() {
                                Action::Resize(ResizeMsg::End)
                            } else {
                                Action::Noop
                            }
                        })
                        onmousemove=self.link.callback(move |e : MouseMoveEvent| {
                            if is_resizing.clone() {
                                if e.movement_x().abs() > e.movement_y().abs() {
                                    Action::Resize(ResizeMsg::X(e.movement_x() as f64))
                                } else {
                                    Action::Resize(ResizeMsg::Y(e.movement_y() as f64))
                                }
                            } else {
                                Action::Noop
                            }
                        })>
                        { view_grammar(&self, coord!{"root"}) }
                    </div>
                </div>
            </div>
        }
    }
}
