use electron_sys::ipc_renderer;
use pest::Parser;
use std::collections::{HashMap, HashSet};

use std::num::NonZeroU32;
use std::ops::Deref;
use std::option::Option;
use stdweb::traits::IEvent;
use stdweb::unstable::TryInto;
use stdweb::web::{document, IElement, INode, IParentNode};
use wasm_bindgen::JsValue;
use yew::events::{KeyDownEvent, KeyPressEvent, KeyUpEvent};
use yew::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::services::ConsoleService;

use crate::coordinate::{Col, Coordinate, Row};
use crate::grammar::{Grammar, Kind, Lookup};
use crate::grammar_map::*;
use crate::session::Session;
use crate::style::Style;
use crate::util::{move_grammar, non_zero_u32_tuple, resize, resize_diff};
use crate::view::{view_grammar, view_menu_bar, view_side_nav, view_tab_bar};
use crate::{coord, coord_col, coord_row, g, gg, row_col_vec};

#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;

// Model contains the entire state of the application
#[derive(Debug)]
pub struct Model {
    // Parts of the application state are described below:

    // - `view_root` represents the parent grammar that the view starts rendering from
    view_root: Coordinate,

    // - `active_cell`
    pub active_cell: Option<Coordinate>,

    // - `first_select_cell` is the top-leftmost cell in a selection
    // - `last_select_cell` is the bottom-rightmost cell in a selection
    pub first_select_cell: Option<Coordinate>,
    pub last_select_cell: Option<Coordinate>,

    pub secondary_selections: HashSet<Coordinate>,

    // TODO: are `min_select_cell` and `max_select_cell` still useful
    pub min_select_cell: Option<Coordinate>,
    pub max_select_cell: Option<Coordinate>,

    // - `shift_key_pressed` is a simple indicator for when shift key is toggled
    pub shift_key_pressed: bool,

    // - `zoom` is the value that corresponds to how "zoomed" the sheet is
    pub zoom: f32,

    // - `meta_suggestions` contains a map of the name of suggestions to the
    //   suggested grammars stored in coord_col!("meta", "A")
    pub meta_suggestions: Vec<(String, Coordinate)>,

    // - `lookups` represent an ordered list of coordinates that have lookups corresponding
    // to them. the indexes are used to generate correspoding color coding for each lookup
    pub lookups: Vec<Coordinate>,

    // - `col_widths` & `row_heights` map coordinate to sizes based on column or row
    pub col_widths: HashMap<Col, f64>,
    pub row_heights: HashMap<Row, f64>,

    // - `sessions` represents the currently open sessions that are shown in the tab bar,
    //   where each session
    // - `current_session_index` tells us which of the open sessions is currently active
    pub sessions: Vec<Session>,
    pub current_session_index: usize,

    // - `side_menus` represent the state
    pub side_menus: Vec<SideMenu>,
    pub open_side_menu: Option<i32>,

    // - `focus_node_ref` is a reference to the current cell that should be in focus
    pub focus_node_ref: NodeRef,
    pub next_focus_node_ref: NodeRef,

    // - `resizing` is an optional reference to the current coordinate being resized
    //    (which is None if no resizing is happening)
    pub resizing: Option<Coordinate>,

    // - `link` is a function of the Yew framework for referring back to the current component
    //    so actions can be chained, for instance
    pub link: ComponentLink<Model>,

    // - `default_nested_row_cols` shows the default number of rows and columns
    //   created by Ctrl+G or the "Nest Grid" button
    // - `default_definition_name` shows the default name of the grammar created
    //   by Ctrl+G the "Add Definition" button
    pub default_nested_row_cols: (NonZeroU32, NonZeroU32),
    pub default_definition_name: String,

    // - `mouse_cursor` corresponds to the appearance of the mouse cursor
    pub mouse_cursor: CursorType,

    // - `console` and `reader` are used to access native browser APIs for the
    //    dev console and FileReader respectively
    console: ConsoleService,
    reader: ReaderService,

    // - `tasks` are used to store asynchronous requests to read/load files
    tasks: Vec<ReaderTask>,
}

#[derive(Debug)]
pub struct SideMenu {
    pub name: String,
    pub icon_path: String,
}

// SUBACTIONS
// Sub-actions for resize-related operations
pub enum ResizeMsg {
    Start(Coordinate),
    X(f64),
    Y(f64),
    End,
}

// Sub-actions for adjusting the current look of the cursor
#[derive(Debug)]
pub enum CursorType {
    NS,
    EW,
    Default,
}

pub enum SelectMsg {
    Start(Coordinate),
    End(Coordinate),
}

// ACTIONS
// Triggered in the view, sent to update function
pub enum Action {
    // Do nothing
    Noop,

    // Change string value of Input grammar
    ChangeInput(Coordinate, /* new_value: */ String),

    SetActiveCell(Coordinate),

    NextSuggestion(Coordinate, /* index */ i32),
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
    SetCursorType(CursorType),
    Select(SelectMsg),

    Lookup(
        /* source: */ Coordinate,
        /* lookup_type: */ Lookup,
    ),
    MergeCells(),

    ChangeDefaultNestedGrid((NonZeroU32, NonZeroU32)),

    SetCurrentDefinitionName(String),

    // SetCurrentParentGrammar(Coordinate),
    ToggleLookup(Coordinate),

    AddDefinition(Coordinate, /* name */ String),

    ToggleShiftKey(bool),

    // Alerts and stuff
    Alert(String),
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
            kind: Kind::Grid(row_col_vec![(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1)]),
        };
        let mut m = Model {
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
            meta_suggestions: vec![
                ("js_grammar".to_string(), coord!("meta-A1")),
                ("java_grammar".to_string(), coord!("meta-A2")),
                ("defn".to_string(), coord!("meta-A3")),
            ],

            console: ConsoleService::new(),
            reader: ReaderService::new(),

            first_select_cell: None,
            last_select_cell: None,

            secondary_selections: HashSet::new(),

            min_select_cell: None,
            max_select_cell: None,
            zoom: 1.0,

            sessions: vec![Session {
                title: "my session".to_string(),
                root: root_grammar.clone(),
                meta: meta_grammar.clone(),
                grammars: {
                    let mut map = HashMap::new();
                    build_grammar_map(
                        &mut map,
                        coord!("root"),
                        gg![
                            [
                                g!(Grammar::input("", "A1")),
                                g!(Grammar::input("", "B1")),
                                g!(Grammar::input("", "C1"))
                            ],
                            [
                                g!(Grammar::input("", "A2")),
                                g!(Grammar::input("", "B2")),
                                g!(Grammar::input("", "C2"))
                            ],
                            [
                                g!(Grammar::input("", "A3")),
                                g!(Grammar::input("", "B3")),
                                gg![
                                    [
                                        g!(Grammar::input("", "C3-A1")),
                                        g!(Grammar::input("", "C3-B1"))
                                    ],
                                    [
                                        g!(Grammar::input("", "C3-A2")),
                                        g!(Grammar::input("", "C3-B2"))
                                    ]
                                ]
                            ]
                        ],
                    );
                    map
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
            next_focus_node_ref: NodeRef::default(),

            shift_key_pressed: false,

            default_nested_row_cols: non_zero_u32_tuple((3, 3)),

            default_definition_name: "".to_string(),

            mouse_cursor: CursorType::Default,

            lookups: vec![],
        };
        // load suggestions from
        m.meta_suggestions = m
            .query_col(coord_col!("meta", "A"))
            .iter()
            .filter_map(|coord| {
                if let Some(name) = m.get_session().grammars.get(coord).map(|g| g.name.clone()) {
                    Some((name, coord.clone()))
                } else {
                    None
                }
            })
            .collect();
        m
    }

    // The update function is split into sub-update functions that
    // are specifc to each EventType
    fn update(&mut self, event_type: Self::Message) -> ShouldRender {
        let should_render = match event_type {
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
                            g.kind = Kind::Input(new_value);
                        }
                        Grammar {
                            kind: Kind::Lookup(_, lookup_type),
                            ..
                        } => {
                            g.kind = Kind::Lookup(new_value, lookup_type.clone());
                        }
                        _ => (),
                    }
                }
                true
            }

            Action::SetActiveCell(coord) => {
                self.active_cell = Some(coord.clone());
                let active_cell_id = format! {"cell-{}", coord.to_string()};
                js! {
                    try {
                        let element = document.getElementById(@{active_cell_id.clone()});
                        element.firstChild.focus();
                    } catch (e) {
                        console.log("cannot focus cell with coordinate ", @{active_cell_id});
                    }
                };
                true
            }

            Action::NextSuggestion(coord, index) => {
                let next_suggestion_id =
                    format! {"cell-{}-suggestion-{}", coord.to_string(), index};
                js! {
                    try {
                        let element = document.getElementById(@{next_suggestion_id.clone()});
                        element.focus();
                    } catch (e) {
                        console.log("cannot focus on next suggestion");
                    }
                };
                true
            }

            Action::Select(SelectMsg::Start(coord)) => {
                self.first_select_cell = Some(coord.clone());
                self.last_select_cell = None;
                true
            }
            Action::Select(SelectMsg::End(coord)) => {
                if let Some(mut selection_start) = self.first_select_cell.clone() {
                    // ensure that selection_start and selection_end have common parent
                    let mut common_parent = selection_start.parent();
                    let mut selection_end = Some(coord.clone());
                    let depth_start = selection_start.row_cols.len();
                    let depth_end = selection_end.clone().unwrap().row_cols.len();
                    // depend on which select coord has higher depth, find their common parent
                    if depth_start < depth_end {
                        while selection_end.clone().and_then(|c| c.parent()) != common_parent {
                            selection_end = selection_end.and_then(|c| c.parent());
                        }
                    } else {
                        common_parent = selection_end.clone().unwrap().parent();
                        while selection_start.parent() != common_parent {
                            selection_start = selection_start.parent().unwrap();
                        }
                    
                    }
                 // find the min of row,col and max of row,col in selected region 
                 // which may contain a span coord that has smaller or larger row,col
                    let (mut start_row, mut start_col) = selection_start.clone().row_col();
                    let (mut end_row, mut end_col) = selection_end.clone().unwrap().row_col();
                    if start_row > end_row {
                        let tmp = start_row.clone();
                        start_row = end_row;
                        end_row = tmp;
                    } 
                    if start_col > end_col {
                        let tmp = start_col.clone();
                        start_col = end_col;
                        end_col = tmp;
                    }            
                    let depth_check = selection_start.row_cols.len().clone();
                    let ref_grammas = self.get_session().grammars.clone();
                    let mut check = false;
                    while !check {
                        check = true;
                        let row_range = start_row.get()..=end_row.get();
                        let col_range = start_col.get()..=end_col.get(); 
                        for (coord, grammar) in ref_grammas.iter() {
                            let (coord_row, coord_col) = coord.clone().row_col();
                            let coord_depth = coord.clone().row_cols.len();
                            if row_range.contains(&coord_row.get())
                                && col_range.contains(&coord_col.get()) && (coord_depth == depth_check) {
                                let col_span = grammar.clone().style.col_span;
                                let row_span = grammar.clone().style.row_span;
                                if col_span.0 != 0 && col_span.1 != 0 {
                                    if col_span.0 < start_col.get() {
                                        start_col = NonZeroU32::new(col_span.0).unwrap();
                                        check = false;
                                    }
                                    if col_span.1 > end_col.get() {
                                        end_col = NonZeroU32::new(col_span.1).unwrap();
                                        check = false;
                                    }
                                }
                                if row_span.0 != 0 && row_span.1 != 0 {
                                    if row_span.0 < start_row.get() {
                                        start_row = NonZeroU32::new(row_span.0).unwrap();
                                        check = false;
                                    }
                                    if row_span.1 > end_row.get() {
                                        end_row = NonZeroU32::new(row_span.1).unwrap();
                                        check = false;
                                    }
                                }
                            }
                        }
                    }
                    
                    
                    selection_start.row_cols[depth_check - 1] = (start_row, start_col);
                    selection_end.as_mut().unwrap().row_cols[depth_check - 1] = (end_row, end_col);
                    self.first_select_cell = Some(selection_start.clone());
                    self.last_select_cell = selection_end.clone();
                }
                true
            }

            Action::MergeCells() => {
                if self.first_select_cell.is_none() || self.last_select_cell.is_none() {
                    info!("Expect for select for two coord");
                    return false;
                }           
                let (first_row, first_col) = self.first_select_cell.clone().unwrap().row_col();
                let (last_row, last_col) = self.last_select_cell.clone().unwrap().row_col();
                
                let depth_check = self.last_select_cell.clone().unwrap().row_cols.len();
                let parent_check = self.last_select_cell.clone().unwrap().parent();

                let row_range = first_row.get()..=last_row.get();
                let col_range = first_col.get()..=last_col.get(); 
                
                let mut merge_height = 0.00;
                let mut merge_width = 0.00;
                let mut max_coord = Coordinate::default();
                let mut max_grammar = Grammar::default();
                let mut ref_grammas = self.get_session_mut().grammars.clone();
                for (coord, grammar) in ref_grammas.iter_mut() {
                    if coord.parent() == parent_check { 
                    }
                    if  coord.to_string().contains("root-") {
                        if row_range.contains(&coord.row().get()) && col_range.contains(&coord.col().get()) && coord.parent() == parent_check                    
                        {                                                  
                            let coord_style = grammar.style.clone();
                            if coord_style.display != false  { 
                                if coord.row().get() == last_row.get() {  
                                    merge_width = merge_width + coord_style.width;
                                
                                } 
                                if coord.col().get() == last_col.get() {
                                    merge_height = merge_height + coord_style.height;
                                }
                                    if coord.row().get() == last_row.get()
                                    && (coord.col().get() == last_col.get())
                                {          
                                    max_coord = coord.clone();
                                    max_grammar = grammar.clone();
                                } else {
                                    grammar.style.display = false;
                                }                                            
                            }
                            grammar.kind =  Kind::Input("".to_string());                   
                            grammar.style.col_span.0 = first_col.get();
                            grammar.style.col_span.1 = last_col.get();
                            grammar.style.row_span.0 = first_row.get();
                            grammar.style.row_span.1 = last_row.get();                    
                            self.get_session_mut()
                                .grammars
                                .insert(coord.clone(), grammar.clone());
                        }
                    }                            
                }
                max_grammar.kind =  Kind::Input("".to_string());
                max_grammar.style.width = merge_width;
                max_grammar.style.height = merge_height;
                max_grammar.style.col_span.0 = first_col.get();
                max_grammar.style.col_span.1 = last_col.get();
                max_grammar.style.row_span.0 = first_row.get();
                max_grammar.style.row_span.1 = last_row.get();             
                self.get_session_mut()
                    .grammars
                    .insert(max_coord.clone(), max_grammar.clone());
                true
            }

            Action::DoCompletion(source_coord, dest_coord) => {
                move_grammar(self, source_coord, dest_coord.clone());
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

            Action::SetSessionTitle(name) => {
                self.get_session_mut().title = name;
                true
            }

            Action::ReadDriverFiles(files_list) => {
                // Get the main file and miscellaneous/additional files from the drivers list
                let (main_file, misc_files) = {
                    let (main_file_as_vec, misc_files) : (Vec<File>, Vec<File>) = files_list.iter().fold((Vec::new(), Vec::new()), |accum, file| { 
                        // Iter::partition is used to divide a list into two given a certain condition.
                        // Here the condition here is to separate the main file of the driver from the
                        // addditional ones, where the main file's path looks like
                        // '{directory_name}/{file_name}.js' and the directory_name == file_name
                        let mut new_accum = accum.clone();
                        // use `webkitRelativePath` as the `name`, if it's available
                        // we'll call out to regular JS to do this using the `js!` macro.
                        // Note that yew::services::reader::File::name() calls "file.name" under the
                        // hood (https://docs.rs/stdweb/0.4.20/src/stdweb/webapi/file.rs.html#23)
                        let full_file_name : String = js!(
                            if (!!@{&file}.webkitRelativePath) {
                                return @{&file}.webkitRelativePath;
                            } else {
                                console.log("couldn't get relative path of file: ", @{&file}.name);
                                return @{&file}.name; // equivalent to yew::services::reader::File::name()
                            }
                        ).try_into().unwrap();
                        let path_parts : Vec<&str> = full_file_name.split("/").collect();
                        match (path_parts.first(), path_parts.last(), path_parts.len()) {
                            (Some(directory), Some(file_name), 2) if format!{"{}.js", directory} == file_name.to_string() => {
                                if new_accum.0.len() == 0 {
                                    new_accum.0.push(file.clone());
                                } else {
                                    panic!("[Action::ReadDriverFiles]: there shouldn't be more than 1 main file in the driver directory")
                                }
                            },
                            _ => {
                                new_accum.1.push(file.clone());
                            },
                        };
                        new_accum
                    });
                    // the `partition` call above gives us a tuple of two Vecs (Vec, Vec) where the first Vec
                    // should have only one element, so we'll convert it to a (Vec::Item, Vec).
                    // If this has an error, then there's something wrong with how the driver
                    // directory is organized.
                    (
                        main_file_as_vec.first().unwrap().clone(),
                        misc_files.clone(),
                    )
                };

                // upload misc files so they can be served by electron to be used by main driver file
                let upload_callback = self
                    .link
                    .callback(|file_data| Action::UploadDriverMiscFile(file_data));
                for file in misc_files {
                    let task = self.reader.read_file(file, upload_callback.clone());
                    self.tasks.push(task);
                }

                // Load main driver file. After this task has been scheduled and executed, the
                // driver is ready for use.
                self.tasks.push(
                    self.reader
                        .read_file(main_file, self.link.callback(Action::LoadDriverMainFile)),
                );

                false
            }

            Action::UploadDriverMiscFile(file_data) => {
                // Here, we use some electron APIs to call out to the main process in JS.
                // For this, we use the `electron_sys` library which is pretty experimental but
                // feature complete.
                // See here for documentation how to communicate between the main and renderer proess in Electron:
                //   https://www.tutorialspoint.com/electron/electron_inter_process_communication.htm
                // And here, for the documentation for the electon_sys Rust bindings for electron.ipcRenderer:
                //   https://docs.rs/electron-sys/0.4.0/electron_sys/struct.IpcRenderer.html

                let args: [JsValue; 2] = [
                    JsValue::from_str(file_data.name.deref()),
                    JsValue::from_str(std::str::from_utf8(&file_data.content).unwrap()),
                ];
                ipc_renderer.send_sync("upload-driver-misc-file", Box::new(args));
                false
            }

            Action::LoadDriverMainFile(main_file_data) => {
                info! {"Loading Driver: {}", &main_file_data.name};
                let file_contents = std::str::from_utf8(&main_file_data.content).unwrap();
                // dump file contents into script tag and attach to the DOM
                let script = document().create_element("script").unwrap();
                script.set_text_content(file_contents);
                script.set_attribute("type", "text/javascript");
                script.set_attribute("class", "ise-driver");
                script.set_attribute("defer", "true");
                let head = document().query_selector("head").unwrap().unwrap();
                head.append_child(&script);
                true
            }

            Action::AddNestedGrid(coord, (rows, cols)) => {
                if self.active_cell.is_none() {
                    info!("Expect a cell is active");
                    return false;
                }
                // height and width initial value
                let mut tmp_heigth = 30.0;
                let mut tmp_width = 90.0;

                let current_cell = self.active_cell.clone();
                let mut ref_grammas = self.get_session().grammars.clone();
                let current_grammar = ref_grammas.get(&current_cell.clone().unwrap()).unwrap();

                let (r, c) = non_zero_u32_tuple((rows, cols));
                let mut grammar = Grammar::as_grid(r, c);
                if let Kind::Grid(sub_coords) = grammar.clone().kind {
                    self.active_cell = sub_coords.first().map(|c| Coordinate::child_of(&coord, *c));
                               
                    let current_width = current_grammar.style.width;
                    let current_height = current_grammar.style.height;

                    // let current_width = *self.col_widths.get(&coord.full_col()).unwrap_or(&90.0);
                    // let current_height = *self.row_heights.get(&coord.full_row()).unwrap_or(&30.0);

                    // check if active cell row height and width is greater than default value
                    if current_width > tmp_width {
                        // set height argument to active cell height if greater
                        //Get the actual amount of cell being created and use it instead of "3" being HARD CODED.
                        tmp_width = current_width / 3.0;
                    }
                    if current_height > tmp_heigth {
                        // set width argument to active cell width if greater
                        //Get the actual amount of cell being created and use it instead of "3" being HARD CODED.
                        tmp_heigth = current_height / 3.0;
                    }

                    
                        for sub_coord in sub_coords {
                            let new_coord = Coordinate::child_of(&coord, sub_coord);
    
                            self.get_session_mut()
                                .grammars
                                .insert(new_coord.clone(), Grammar::default());
                            if current_grammar.style.col_span.0 == 0 && current_grammar.style.row_span.0 == 0 {
                                // initialize row & col heights as well
                                if !self.row_heights.contains_key(&new_coord.clone().full_row()) {
                                    self.row_heights
                                        .insert(new_coord.clone().full_row(), tmp_heigth);
                                    //30.0);
                                }
                                if !self.col_widths.contains_key(&new_coord.clone().full_col()) {
                                    self.col_widths
                                        .insert(new_coord.clone().full_col(), tmp_width);
                                    //90.0);
                                }
                            }
                        }
                    
                    // info!("tmp_width-{},tmp_heigth-{}", tmp_width.clone(), tmp_heigth.clone());
                    // info!("current_width-{}, current_height-{}", current_width.clone(), current_height.clone());
                }
               
                if let Some(parent) = Coordinate::parent(&coord)
                    .and_then(|p| self.get_session_mut().grammars.get_mut(&p))
                {
                    parent.kind = grammar.clone().kind; // make sure the parent gets set to Kind::Grid
                } 
                 
                if current_grammar.style.row_span.0 != 0 || current_grammar.style.col_span.0 != 0 {
                    grammar.style.row_span = current_grammar.style.row_span.clone();
                    grammar.style.col_span = current_grammar.style.col_span.clone();
                }
                self.get_session_mut()
                    .grammars
                    .insert(coord.clone(), grammar.clone());           
                resize(
                    self,
                    coord.clone(),
                    (rows as f64) * (/* default row height */tmp_heigth),
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
                    coord!("meta-A4") => Grammar::default_button(),
                    coord!("meta-A5") => Grammar::default_slider(),
                    coord!("meta-A6") => Grammar::default_toggle(),
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
                        self.resizing = Some(coord);
                    }
                    ResizeMsg::X(offset_x) => {
                        if let Some(coord) = self.resizing.clone() {
                            resize_diff(self, coord, 0.0, offset_x);
                            self.mouse_cursor = CursorType::EW;
                        }
                    }
                    ResizeMsg::Y(offset_y) => {
                        if let Some(coord) = self.resizing.clone() {
                            resize_diff(self, coord, offset_y, 0.0);
                            self.mouse_cursor = CursorType::NS;
                        }
                    }
                    ResizeMsg::End => {
                        self.resizing = None;
                        self.mouse_cursor = CursorType::Default;
                    }
                }
                true
            }

            Action::SetCursorType(cursor_type) => {
                self.mouse_cursor = cursor_type;
                true
            }

            Action::Lookup(source_coord, lookup_type) => {
                match lookup_type {
                    Lookup::Cell(dest_coord) => {
                        move_grammar(self, source_coord, dest_coord.clone());
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
            Action::AddDefinition(coord, defn_name) => {
                // adds a new grammar or sub-grammar to the meta
                let max_a_row =
                    self.query_col(coord_col!("meta", "A"))
                        .iter()
                        .fold(1, |max_a_row, c| {
                            if c.col().get() == 1 && c.row().get() > max_a_row {
                                c.row().get()
                            } else {
                                max_a_row
                            }
                        });
                // add new sub_coord to coord!("meta") grid
                let defn_meta_sub_coord = non_zero_u32_tuple((max_a_row + 1, 1));
                if let Kind::Grid(sub_coords) = &mut self.get_session_mut().meta.kind {
                    sub_coords.push(defn_meta_sub_coord.clone());
                }
                let defn_coord = Coordinate::child_of(&(coord!("meta")), defn_meta_sub_coord);
                info! {"Adding Definition: {} to {}", coord.to_string(), defn_coord.to_string()};
                move_grammar(self, coord, defn_coord.clone());
                // give moved grammar name {defn_name} as specified in "Add Definition" button
                if let Some(g) = self.get_session_mut().grammars.get_mut(&defn_coord) {
                    g.name = defn_name;
                }
                true
            }

            Action::ToggleShiftKey(toggle) => {
                self.shift_key_pressed = toggle;
                false
            }

            Action::ChangeDefaultNestedGrid(row_col) => {
                self.default_nested_row_cols = row_col;
                false
            }

            Action::SetCurrentDefinitionName(name) => {
                self.default_definition_name = name;
                false
            }
        };

        self.meta_suggestions = self
            .query_col(coord_col!("meta", "A"))
            .iter()
            .filter_map(|coord| {
                if let Some(name) = self
                    .get_session()
                    .grammars
                    .get(coord)
                    .map(|g| g.name.clone())
                {
                    Some((name, coord.clone()))
                } else {
                    None
                }
            })
            .collect();

            
            

        should_render
    }

    fn view(&self) -> Html {
        let is_resizing = self.resizing.is_some();
        // for integration tests
        let serialized_model = serde_json::to_string(&self.get_session()).unwrap();
        let zoom = format! { "zoom: {};", &self.zoom };
        let cursor = format! { "cursor: {};", match self.mouse_cursor {
            CursorType::NS => "ns-resize",
            CursorType::EW => "ew-resize",
            CursorType::Default => "default",
        }};
        let (default_row, default_col) = {
            let (r, c) = self.default_nested_row_cols.clone();
            (r.get(), c.get())
        };
        let active_cell = self.active_cell.clone().expect("active_cell should be set");
        html! {
            <div>

                { view_side_nav(&self) }

                { view_menu_bar(&self) }

                { view_tab_bar(&self) }
                <input id="integration-test-model-dump" hidden=true >{serialized_model}</input>

                <div class="main">
                    <div id="grammars" class="grid-wrapper" style={zoom+&cursor}
                        // Global Key-mappings
                        onkeypress=self.link.callback(move |e : KeyPressEvent| {
                            let keys = key_combination(&e);
                            info! {"global keypress: {}", keys.clone()};
                            match keys.deref() {
                                // Tab (navigation) is handled in onkeydown
                                "Ctrl-g" => {
                                    Action::AddNestedGrid(active_cell.clone(), (default_row, default_col))
                                }
                                _ => Action::Noop
                            }
                        })
                        // Global Key toggles
                        onkeydown=self.link.callback(move |e : KeyDownEvent| {
                            let keys = key_combination(&e);
                            match keys.deref() {
                                // Tab is intercepted in onkeydown instead of onkeypress
                                // to allow us prevent default tabbing behavior
                                // "Tab" => {
                                //     e.prevent_default();
                                //     if let Some(c) = neighbor_right.clone().or(first_col_next_row.clone()) {
                                //         return Action::SetActiveCell(c)
                                //     }
                                //     Action::Noop
                                // }
                                "Shift" => Action::ToggleShiftKey(true),
                                _ => Action::Noop
                            }
                        })
                        onkeyup=self.link.callback(move |e : KeyUpEvent| {
                            if e.key() == "Shift" {
                                Action::ToggleShiftKey(false)
                            } else {
                                Action::Noop
                            }
                        })
                        // Global Mouse event/toggles
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

fn key_combination<K>(e: &K) -> String
where
    K: IKeyboardEvent,
{
    format! {"{}{}{}{}{}",
        if e.meta_key() { "Meta-" } else { "" },
        if e.ctrl_key() { "Ctrl-" } else { "" },
        if e.alt_key() { "Alt-" } else { "" },
        if e.shift_key() { "Shift-" } else { "" },
        if e.key().trim() != "" { e.key() } else { e.code() } ,
    }
}
