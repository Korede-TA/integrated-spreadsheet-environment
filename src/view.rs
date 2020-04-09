#![recursion_limit = "1024"]
use std::num::NonZeroU32;
use std::ops::Deref;
use stdweb::traits::IEvent;
use stdweb::unstable::TryFrom;
use stdweb::web::{html_element::InputElement, HtmlElement, IHtmlElement};
// use web_sys::HtmlInputElement as InputElement;
use yew::events::{ClickEvent, IKeyboardEvent, IMouseEvent, KeyPressEvent};
use yew::prelude::*;
use yew::services::reader::File;
use yew::{html, ChangeData, Html, InputData};

use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Interactive, Kind, Lookup};
use crate::model::{Action, CursorType, Model, ResizeMsg, SelectMsg, SideMenu};
use crate::style::get_style;
use crate::util::non_zero_u32_tuple;

pub fn view_side_nav(m: &Model) -> Html {
    let mut side_menu_nodes = VList::new();
    let mut side_menu_section = html! { <></> };
    for (index, side_menu) in m.side_menus.iter().enumerate() {
        if Some(index as i32) == m.open_side_menu {
            side_menu_nodes.add_child(html! {
                <button class="active-menu" onclick=m.link.callback(|e| Action::SetActiveMenu(None))>
                    <img src={side_menu.icon_path.clone()} 
                         width="40px" alt={side_menu.name.clone()}>
                    </img>
                </button>
            });

            side_menu_section = view_side_menu(m, side_menu);
        } else {
            side_menu_nodes.add_child(html! {
                <button onclick=m.link.callback(move |e| Action::SetActiveMenu(Some(index as i32)))>
                    <img
                        src={side_menu.icon_path.clone()}
                        width="40px" alt={side_menu.name.clone()}>
                    </img>
                </button>
            });
        }
    }

    html! {
        <div class="sidenav">
            { side_menu_nodes }

            { side_menu_section }
        </div>
    }
}

pub fn view_side_menu(m: &Model, side_menu: &SideMenu) -> Html {
    match side_menu.name.deref() {
        "Home" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS Home MENU"}
                </div>
            }
        }
        "File Explorer" => {
            html! {
                <div class="side-menu-section">
                    <h1>
                        {"File Explorer"}
                    </h1>

                    <h3>{"load session"}</h3>
                    <br></br>
                    <input type="file" onchange=m.link.callback(|value| {
                        if let ChangeData::Files(files) = value {
                            if files.len() >= 1 {
                                if let Some(file) = files.iter().nth(0) {
                                    return Action::ReadSession(file);
                                }
                            } else {
                                return Action::Alert("Could not load file".to_string());
                            }
                        }
                        Action::Noop
                    })>
                    </input>
                    <h3>{"save session"}</h3>
                    <br></br>
                    <input type="text" value=m.get_session().title onchange=m.link.callback(|v| {
                        if let ChangeData::Value(s) = v {
                            return Action::SetSessionTitle(s);
                        }
                        Action::Noop
                    })>

                    </input>
                    <input type="button" value="Save" onclick=m.link.callback(|_| Action::SaveSession())>
                    </input>
                </div>
            }
        }
        "Settings" => {
            html! {
                <div class="side-menu-section">
                    <h1>
                        {"Settings"}
                    </h1>

                    <h3>{"load driver"}</h3>
                    <br></br>
                    // drivers will be represented as directories, so we use "webkitdirectory"
                    // (which isn't standard, but supported in chrome) to get an array of files in
                    // the dirctory
                    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/webkitdirectory
                    <input
                        type="file"
                        webkitdirectory=""
                        onchange=m.link.callback(|value| {
                        if let ChangeData::Files(files) = value {
                            // `files` will be a flat list with each file's "webkitRelativePath",
                            // being a full path starting with the directory name.
                            // ReadDriverFiles will load the .js file with the same name as the
                            // directory, and upload the rest of the files to be served by electron
                            let files_list : Vec<File> = files.into_iter().collect();
                            if files_list.len() >= 1 {
                                return Action::ReadDriverFiles(files_list);
                            } else {
                                return Action::Alert("Could not load Driver".to_string());
                            }
                        }
                        Action::Noop
                    })>
                    </input>
                </div>
            }
        }
        "Info" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS info MENU"}
                </div>
            }
        }

        _ => html! {<> </>},
    }
}

pub fn view_menu_bar(m: &Model) -> Html {
    let active_cell = m.active_cell.clone();
    let (default_row, default_col) = {
        let (r, c) = m.default_nested_row_cols.clone();
        (r.get(), c.get())
    };
    // SPECIAL MENU BAR ITEMS
    let nest_grid_button = html! {
        /* the "Nest Grid" button is special because
         * it contains fields for the variable size of the button
         */
        <button class="menu-bar-button" onclick=m.link.callback(move |_| {
            if let Some(current) = &active_cell {
                Action::AddNestedGrid(current.clone(), (default_row, default_col))
            } else { Action::Noop }
        })>
            { "Nest Grid  " }
            <input
                class="active-cell-indicator"
                placeholder="Row"
                size="3"
                oninput=m.link.callback(move |e: InputData| {
                    if let Ok (row) = e.value.parse::<i32>() {
                        Action::ChangeDefaultNestedGrid(non_zero_u32_tuple(((row as u32), default_col)))
                    } else {
                        Action::Noop
                    }
                })
                onclick=m.link.callback(|e: ClickEvent| { e.prevent_default(); Action::Noop })
                value={default_row}>
            </input>
            <input
                class="active-cell-indicator"
                placeholder="Col"
                size="3"
                onchange=m.link.callback(move |e: ChangeData| {
                    if let ChangeData::Value(value) = e {
                        if let Ok (col) = value.parse::<i32>() {
                            return Action::ChangeDefaultNestedGrid(
                                non_zero_u32_tuple((default_row, (col as u32)))
                            );
                        }
                    }
                    Action::Noop
                })
                onclick=m.link.callback(|e: ClickEvent| { e.prevent_default(); Action::Noop })
                value={default_col}>
            </input>
        </button>
    };

    let add_definition_button = {
        let (can_add_definition, default_name, callback) = match (
            m.first_select_cell.clone(),
            m.last_select_cell.clone(),
        ) {
            // definitions can occur when a range of coordinates are selected where:
            // - the first (top-leftmost) and last (bottom-rightmost) selected cells have the same parent
            // - the first selected cell is the first (top-leftmost) child of the parent
            // - the last selected cell is the last (bottom-rightmost) child of the parent
            // cell, which should be a Kind::Grid grammar
            (Some(first), Some(last)) if first.parent() == last.parent() => {
                if let Some((Kind::Grid(sub_coords))) = 
                        /* get the coordinate of the parent, lookup the grammar, then get the grammar.kind */
                        first
                        .parent()
                        .and_then(|c| m.get_session().grammars.get(&c))
                        .map(|g| (g.kind.clone()))
                    {
                        use std::cmp::Ordering;
                        let mut sc = sub_coords.clone();
                        sc.sort_by(|(a_row, a_col), (b_row, b_col)| {
                            if a_row > b_row {
                                Ordering::Greater
                            } else if a_row < b_row {
                                Ordering::Less
                            } else {
                                if a_col > b_col {
                                    Ordering::Greater
                                } else if a_col < b_col {
                                    Ordering::Less
                                } else {
                                    Ordering::Equal
                                }
                            }
                        });
                        let first_sc = sc.first().expect(
                            "add_definition_button: expect selection parent sub_coords.len > 1",
                        );
                        let last_sc = sc.last().expect(
                            "add_definition_button: expect selection parent sub_coords.len > 1",
                        );
                        let defn_name = if m.default_definition_name == "" {
                            first.parent().unwrap().to_string().replace("-", "_")
                        } else {
                            m.default_definition_name.clone()
                        };
                        (
                            // can add definition?
                            *first_sc == first.row_col() &&
                            *last_sc == last.row_col(),
                            // definition name
                            defn_name.clone(),
                            // callback
                            m.link.callback(move |_| {
                                Action::AddDefinition(first.parent().unwrap(), defn_name.clone())
                            }),
                        )
                    } else {
                        (false, "".to_string(), m.link.callback(|_| Action::Noop))
                    }
            }
            _ => (false, "".to_string(), m.link.callback(|_| Action::Noop)),
        };
        /*
        let suggestions: Vec<_> = m
            .meta_suggestions
            .iter()
            .map(|(name, suggestion_coord)| {
                html! {
                    <option value={ name } onclick=m.link.callback(|e: ClickEvent| {
                        Action::SetCurrentParentGrammar(suggestion_coord.clone())
                    })>
                    </option>
                }
            })
            .collect();
        */

        html! {
            <button class="menu-bar-button" disabled={ !can_add_definition } onclick=callback>
                { "Add Definition  " }
                <input
                    class="active-cell-indicator"
                    placeholder="Name"
                    size="10"
                    disabled={ !can_add_definition }
                    onchange=m.link.callback(move |e: ChangeData| {
                        if let ChangeData::Value(value) = e {
                            return Action::SetCurrentDefinitionName(value);
                        }
                        Action::Noop
                    })
                    onclick=m.link.callback(|e: ClickEvent| { e.prevent_default(); Action::Noop })
                    value={"".to_string()}>
                </input>
                /*
                <div>
                    <input
                        class="active-cell-indicator"
                        placeholder="Parent Name"
                        size="10"
                        disabled={ !can_add_definition }
                        onchange=m.link.callback(move |e: ChangeData| {
                            if let ChangeData::Value(value) = e {
                                return Action::SetCurrentDefinitionName(value);
                            }
                            Action::Noop
                        })
                        onclick=m.link.callback(|e: ClickEvent| { e.prevent_default(); Action::Noop })
                        value={parent_defn}>
                    </input>
                    { VList::new_with_children(suggestions) }
                </div>
                */
            </button>
        }
    };
    // ALL MENU BAR ITEMS
    html! {
        <div class="menu-bar horizontal-bar">
            <input
                class="active-cell-indicator"
                disabled=true
                // TODO: clicking on this should highlight
                // the active cell
                value={
                    match (m.active_cell.clone(), m.first_select_cell.clone(), m.last_select_cell.clone()) {
                        (_, Some(first_cell), Some(last_cell)) =>
                            format!{"{}:{}", first_cell.to_string(), last_cell.to_string()},
                        (Some(cell), _, _) => cell.to_string(),
                        _ => "".to_string(),
                    }
                }>
            </input>
            <button id="SaveSession" class="menu-bar-button" onclick=m.link.callback(|_| Action::SaveSession()) >
                { "Save" }
            </button>
            <button class="menu-bar-button">
                { "Git" }
            </button>
            <button id="ZoomIn" class="menu-bar-button" onclick=m.link.callback(|_| Action::ZoomIn)>
                { "Zoom In (+)" }
            </button>
            <button id="ZoomReset" class="menu-bar-button" onclick=m.link.callback(|_| Action::ZoomReset)>
                { "Zoom Reset" }
            </button>
            <button id="ZoomOut" class="menu-bar-button" onclick=m.link.callback(|_| Action::ZoomOut)>
                { "Zoom Out (-)" }
            </button>
            <button id="Reset" class="menu-bar-button" onclick=m.link.callback(|_| Action::Recreate)>
                { "Reset" }
            </button>
            //<>
                { nest_grid_button }
            //</>
            <button id="InsertRow" class="menu-bar-button" onclick=m.link.callback(|_| Action::InsertRow)>
                { "Insert Row" }
            </button>
            <button id="InsertCol" class="menu-bar-button" onclick=m.link.callback(|_| Action::InsertCol)>
                { "Insert Column" }
            </button>
            <button id="Merge" class="menu-bar-button" onclick=m.link.callback(move |_ : ClickEvent| Action::MergeCells())>
                { "Merge" }
            </button>
            <button id="DeleteRow" class="menu-bar-button" onclick=m.link.callback(|_| Action::DeleteRow)>
                { "Delete Row" }
            </button>
            <button id="DeleteCol" class="menu-bar-button" onclick=m.link.callback(|_| Action::DeleteCol)>
                { "Delete Column" }
            </button>
            //<>
                { add_definition_button }
            //</>
        </div>
    }
}

pub fn view_tab_bar(m: &Model) -> Html {
    let mut tabs = VList::new();
    for (index, tab) in m.sessions.clone().iter().enumerate() {
        if (index as usize) == m.current_session_index {
            tabs.add_child(html! {
                <button class="tab active-tab">{ tab.title.clone() }</button>
            });
        } else {
            tabs.add_child(html! {
                <button class="tab">{ tab.title.clone() }</button>
            });
        }
    }
    html! {
        <div class="tab-bar horizontal-bar">
            { tabs }
            <button class="newtab-btn">
                <span>{ "+" }</span>
            </button>
        </div>
    }
}

pub fn view_grammar(m: &Model, coord: Coordinate) -> Html {
    let is_active = m.active_cell.clone() == Some(coord.clone());
    if let Some(grammar) = m.get_session().grammars.get(&coord) {
        // account for merged cells with have been hidden via their Style.display property.
        if grammar.clone().style.display == false {
            return html! {<> </>};
        }
        match grammar.kind.clone() {
            Kind::Text(value) => view_text_grammar(m, &coord, value, is_active),
            Kind::Input(value) => {
                let suggestions = m
                    .meta_suggestions
                    .iter()
                    .filter_map(|(name, suggestion_coord)| {
                        // suggestion_coord
                        // info! {"filtering suggestions by {}", value}
                        if let Some(suggestion_grammar) =
                            m.get_session().grammars.get(&suggestion_coord)
                        {
                            if name.contains(value.deref()) {
                                Some((suggestion_coord.clone(), suggestion_grammar.clone()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                view_input_grammar(m, coord.clone(), suggestions, value, is_active)
            }
            Kind::Interactive(name, Interactive::Button()) => {
                html! {
                    <div
                        class=format!{"cell interactive row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ get_style(&m, &coord) }>
                        <button>
                            { name }
                        </button>
                    </div>
                }
            }
            Kind::Interactive(name, Interactive::Slider(value, min, max)) => {
                html! {
                    <div
                        class=format!{"cell interactive row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ get_style(&m, &coord) }>
                        <input type="range" min={min} max={max} value={value}>
                            { name }
                        </input>
                    </div>
                }
            }
            Kind::Interactive(name, Interactive::Toggle(checked)) => {
                html! {
                    <div
                        class=format!{"cell interactive row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ get_style(&m, &coord) }>
                        <input type="checkbox" checked={checked}>
                            { name }
                        </input>
                    </div>
                }
            }
            Kind::Grid(sub_coords) => view_grid_grammar(
                m,
                &coord,
                sub_coords
                    .iter()
                    .map(|c| Coordinate::child_of(&coord, *c))
                    .collect(),
            ),
            Kind::Lookup(value, lookup_type) => {
                let suggestions: Vec<Coordinate> = m
                    .get_session()
                    .grammars
                    .keys()
                    .filter_map(|lookup_c| {
                        if lookup_c.to_string().contains(value.deref()) {
                            Some(lookup_c.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                view_lookup_grammar(m, &coord, suggestions, value, lookup_type, is_active)
            }
            Kind::Defn(name, defn_coord, sub_grammars) => {
                view_defn_grammar(m, &coord, &defn_coord, name, sub_grammars)
            }
        }
    } else {
        html! { <></> }
    }
}

pub fn view_defn_grammar(
    m: &Model,
    coord: &Coordinate,
    defn_coord: &Coordinate,
    name: String,
    sub_coordinates: Vec<(String, Coordinate)>,
) -> Html {
    let mut nodes = VList::new();
    let _suggestions: Vec<(Coordinate, Grammar)> = vec![];
    let mut index = 1;
    for (name, _coord) in sub_coordinates {
        let name_coord = Coordinate::child_of(defn_coord, non_zero_u32_tuple((index.clone(), 1)));
        let grammar_coord =
            Coordinate::child_of(defn_coord, non_zero_u32_tuple((index.clone(), 2)));
        nodes.add_child(html! {
            <div>
                // { view_text_grammar(m, &name_coord, name) } // changes to the sub-rule name requires re-bindings
                { view_grammar(m, grammar_coord) }  // any change to the grammar, reflects in the grammar map
            </div>
        });
        index += 1;
    }
    let c = coord.clone();
    html! {
        <div
            class=format!{"cell grid row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <input
                class="cell"
                value={name}>
                // oninput=m.link.callback(move |e : InputData| Action::DefnUpdateName(c.clone(), e.value))>
            </input>
            { nodes }
        </div>
    }
}

pub fn view_defn_variant_grammar(
    m: &Model,
    coord: &Coordinate,
    _defn_coord: &Coordinate,
    _name: String,
    sub_coords: Vec<Coordinate>,
) -> Html {
    let mut nodes = VList::new();
    for c in sub_coords {
        nodes.add_child(view_grammar(m, c.clone()));
    }
    html! {
        <div
            class=format!{"cell variant row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            { nodes }
            <button onclick=m.link.callback(|_| Action::InsertCol)>
                {"+"}
            </button>
        </div>
    }
}

pub fn view_lookup_grammar(
    m: &Model,
    coord: &Coordinate,
    suggestions: Vec<Coordinate>,
    value: String,
    _lookup_type: Option<Lookup>,
    is_active: bool,
) -> Html {
    let suggestions_div = if is_active {
        let mut suggestions_nodes = VList::new();
        for lookup_coord in suggestions {
            let dest = coord.clone();
            let source = lookup_coord.clone();
            suggestions_nodes.add_child(html!{
                <a tabindex=2
                    onclick=m.link.callback(move |_ : ClickEvent| Action::DoCompletion(source.clone(), dest.clone()))>
                    { lookup_coord.to_string() }
                </a>
            })
        }
        html! {
            <div class="suggestion-content">
                { suggestions_nodes }
            </div>
        }
    } else {
        html! { <></> }
    };
    let c = coord.clone();
    let to_toggle = coord.clone();
    let can_toggle: bool = value.clone().deref() == "";
    html! {
        <div
            class=format!{"cell suggestion lookup row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <b style="font-size: 20px;">{ "$" }</b>
            <div contenteditable=true
                class=format!{
                        "cell-data {}",
                        if is_active { "cell-active" } else { "cell-inactive" },
                      }
                placeholder="coordinate"
                ref={
                    if is_active {
                        m.focus_node_ref.clone()
                    } else { NodeRef::default() }
                }
                onkeydown=m.link.callback(move |e : KeyDownEvent| {
                    if e.code() == "Backspace" && can_toggle {
                        Action::ToggleLookup(to_toggle.clone())
                    } else { Action::Noop }
                })
                oninput=m.link.callback(move |e : InputData| Action::ChangeInput(c.clone(), e.value))>
            </div>
            { value }
            { suggestions_div }
        </div>
    }
}

pub fn view_input_grammar(
    m: &Model,
    coord: Coordinate,
    suggestions: Vec<(Coordinate, Grammar)>,
    value: String,
    is_active: bool,
) -> Html {
    if let Some(grammar) = m.get_session().grammars.get(&coord) {
        if grammar.clone().style.display == true {
            return html! { <> </> };
        }
    }
    // load the suggestion values, including the completion callbacks
    // and parse them into DOM nodes
    let suggestions_len = if value.clone() != "" && is_active {
        suggestions.len()
    } else {
        0
    };
    let suggestions = if value.clone() != "" && is_active {
        let mut suggestion_nodes = VList::new();
        let mut suggestion_index = 1;
        for (s_coord, s_grammar) in suggestions {
            let s_coord_2 = s_coord.clone();
            let c = coord.clone();
            let dest_coord = coord.clone();
            suggestion_nodes.add_child(html! {
                    <a 
                        id=format!{"cell-{}-suggestion-{}", c.to_string(), suggestion_index}
                        tabindex=2
                        onkeydown=m.link.callback(move |e : KeyDownEvent| {
                            if e.code() == "Tab" {
                                e.prevent_default();
                                return Action::NextSuggestion(c.clone(), if e.shift_key() { suggestion_index-1 } else { suggestion_index+1 });
                            } else if e.code() == "Enter" || e.code() == "Space" {
                                return Action::DoCompletion(s_coord_2.clone(), c.clone());
                            }
                            Action::Noop
                        })
                        onclick=m.link.callback(move |_ : ClickEvent| Action::DoCompletion(s_coord.clone(), dest_coord.clone()))>
                        { &s_grammar.name }
                    </a>
                });
            suggestion_index += 1;
        }
        html! {
            <div class="suggestion-content">
                { suggestion_nodes }
            </div>
        }
    } else {
        html! { <></> }
    };
    /*
     * Calculate if a specific cell should be selected based on the top-rightmost
     * and bottom-leftmost cells
     */
    let is_selected = cell_is_selected(&coord, &m.first_select_cell, &m.last_select_cell);
    let has_lookup_prefix: bool = value.clone() == "$";
    let current_coord = coord.clone();
    let tab_coord = coord.clone();
    let focus_coord = coord.clone();
    let drag_coord = coord.clone();
    let shift_key_pressed = m.shift_key_pressed;
    let new_selected_cell = coord.clone();
    let cell_classes =
        format! {"cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string()};
    let cell_data_classes = format! {
        "cell-data {} {}",
        if is_active { "cell-active" } else { "cell-inactive" },
        if is_selected { "selection" } else { "" }
    };
    // relevant coordinates for navigation purposes
    let neighbor_left = current_coord
        .neighbor_left()
        .filter(|c| m.get_session().grammars.contains_key(&c))
        .clone();
    let neighbor_right = current_coord
        .neighbor_right()
        .filter(|c| m.get_session().grammars.contains_key(&c))
        .clone();
    let first_col_next_row = {
        let temp = &mut current_coord.neighbor_below();
        if let Some(t) = temp {
            let col = t.col_mut();
            *col = NonZeroU32::new(1).unwrap();
            if m.get_session().grammars.contains_key(&t) {
                Some(t.clone())
            } else {
                None
            }
        } else {
            None
        }
    };
    let last_col_prev_row = /* TODO: get the correct value of this */ current_coord.neighbor_above();
    let keydownhandler = m.link.callback(move |e: KeyDownEvent| {
        info! {"suggestion len {}", suggestions_len}
        if e.code() == "Tab" {
            e.prevent_default();
            if suggestions_len > 0 {
                return Action::NextSuggestion(tab_coord.clone(), 1);
            }
            let next_active_cell = if e.shift_key() {
                neighbor_left.clone().or(last_col_prev_row.clone())
            } else {
                neighbor_right.clone().or(first_col_next_row.clone())
            };
            info! {"next_active_cell {}", next_active_cell.clone().unwrap().to_string()};
            return next_active_cell.map_or(Action::Noop, |c| Action::SetActiveCell(c));
        }
        Action::Noop
    });
    html! {
        <div
            class=cell_classes
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <div contenteditable=true
                class=cell_data_classes
                onkeydown=keydownhandler
                onkeypress=m.link.callback(move |e : KeyPressEvent| {
                    if e.code() == "Space" && has_lookup_prefix {
                        Action::ToggleLookup(current_coord.clone())
                    } else { Action::Noop }
                })
                oninput=m.link.callback(move |e : InputData| {
                    Action::ChangeInput(coord.clone(), e.value)
                })
                onclick=m.link.callback(move |e : ClickEvent| {
                    if e.shift_key() {
                        Action::Select(SelectMsg::End(new_selected_cell.clone()))
                    } else {
                        Action::Select(SelectMsg::Start(new_selected_cell.clone()))
                    }
                })
                onfocus=m.link.callback(move |e : FocusEvent| {
                    if !shift_key_pressed {
                        Action::SetActiveCell(focus_coord.clone())
                    } else {
                        Action::Noop
                    }
                })
                /*
                 * RESIZING
                 * - onmouseover: handle cursor change
                 * - onmousedown/up: handle resize events
                 */
                onmouseover=m.link.callback(move |e: MouseOverEvent| {
                    let (offset_x, offset_y) = {
                        // compute the distance from the right & bottom borders that resizing is allowed
                        let target = HtmlElement::try_from(e.target().unwrap()).unwrap();
                        let rect = target.get_bounding_client_rect();
                        (rect.get_width() - e.offset_x(), rect.get_height() - e.offset_y())
                    };
                    let draggable_area = 4.0;
                    if offset_x < draggable_area {
                        Action::SetCursorType(CursorType::EW)
                    } else if offset_y < draggable_area {
                        Action::SetCursorType(CursorType::NS)
                    } else {
                        Action::SetCursorType(CursorType::Default)
                    }
                })
                onmousedown=m.link.callback(move |e: MouseDownEvent| {
                    let (offset_x, offset_y) = {
                        // compute the distance from the right & bottom borders that resizing is allowed
                        let target = HtmlElement::try_from(e.target().unwrap()).unwrap();
                        let rect = target.get_bounding_client_rect();
                        (rect.get_width() - e.offset_x(), rect.get_height() - e.offset_y())
                    };
                    info!{"offset: {} {}", offset_x, offset_y};
                    let draggable_area = 4.0;
                    if offset_x < draggable_area  || offset_y < draggable_area {
                        Action::Resize(ResizeMsg::Start(drag_coord.clone()))
                    } else {
                        Action::Noop
                    }
                })>
                { value }
            </div>
            { suggestions }
        </div>
    }
}

pub fn view_text_grammar(m: &Model, coord: &Coordinate, value: String, is_active: bool) -> Html {
    let is_selected = cell_is_selected(coord, &m.first_select_cell, &m.last_select_cell);
    html! {
        <div
            class=format!{"cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string(),}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <div
                class={
                    format!{
                        "cell-data {} {}",
                        if is_active { "cell-active" } else { "cell-inactive" },
                        if is_selected { "selection" } else { "" }
                    }
                },
                ref={
                    if is_active {
                        m.focus_node_ref.clone()
                    } else { NodeRef::default() }
                }>
                { value }
            </div>
        </div>
    }
}

pub fn view_grid_grammar(m: &Model, coord: &Coordinate, sub_coords: Vec<Coordinate>) -> Html {
    let mut nodes = VList::new();
    for c in sub_coords {
        nodes.add_child(view_grammar(m, c.clone()));
    }
    html! {
        <div
            class=format!{"\ncell grid row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            { nodes }
        </div>
    }
}

// util function for determining if one cell's coordinate is within the range of selected cells.
fn cell_is_selected(
    coord: &Coordinate,
    first_select_cell: &Option<Coordinate>,
    last_select_cell: &Option<Coordinate>,
) -> bool {
    let depth = first_select_cell
        .clone()
        .map(|c| c.row_cols.len())
        .unwrap_or(std::usize::MAX);
    match (
        first_select_cell
            .clone()
            .and_then(|c| c.row_cols.get(depth - 1).cloned()),
        last_select_cell
            .clone()
            .and_then(|c| c.row_cols.get(depth - 1).cloned()),
    ) {
        (_, _) if coord.row_cols.len() < depth => false,
        (Some((first_row, first_col)), Some((last_row, last_col))) => {
            let current_cell = if coord.row_cols.len() > depth {
                coord.truncate(depth).unwrap_or(coord.clone())
            } else {
                coord.clone()
            };
            let row_range = if first_row.get() > last_row.get() {
                (last_row.get()..=first_row.get())
            // (a..=b) is shorthand for an integer Range that's inclusive of lower and upper bounds
            } else {
                (first_row.get()..=last_row.get())
            };
            let col_range = if first_col.get() > last_col.get() {
                (last_col.get()..=first_col.get())
            } else {
                (first_col.get()..=last_col.get())
            };
            row_range.contains(&current_cell.row().get())
                && col_range.contains(&current_cell.col().get())
        }
        _ => false,
    }
}
