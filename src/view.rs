
use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Interactive, Kind, Lookup};
use crate::model::{Action, Model, SideMenu};
use crate::style::get_style;
use std::collections::hash_map::Keys;
use std::ops::Deref;
use stdweb::web::{HtmlElement, IHtmlElement};
use yew::prelude::*;
use yew::services::reader::File;
use yew::virtual_dom::VList;

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
                    <input type="text" value=m.tabs[m.current_tab].title onchange=m.link.callback(|v| {
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
    html! {
        <div class="menu-bar horizontal-bar">
            <input
                class="active-cell-indicator"
                disabled=true
                // TODO: clicking on this should highlight
                // the active cell
                value={
                    if let Some(cell) = m.active_cell.clone() {
                        cell.to_string()
                    } else {
                        "".to_string()
                    }
                }>
            </input>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::SaveSession()) >
                { "Save" }
            </button>
            <button class="menu-bar-button">
                { "Git" }
            </button>
            <button class="menu-bar-button">
                { "Zoom In (+)" }
            </button>
            <button class="menu-bar-button">
                { "Zoom Out (-)" }
            </button>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::InsertRow)>
                { "Insert Row" }
            </button>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::InsertCol)>
                { "Insert Column" }
            </button>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::DeleteRow)>
                { "Delete Row" }
            </button>
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::DeleteCol)>
                { "Delete Column" }
            </button>
        </div>
    }
}

pub fn view_tab_bar(m: &Model) -> Html {
    let mut tabs = VList::new();
    for (index, tab) in m.tabs.clone().iter().enumerate() {
        if (index as usize) == m.current_tab {
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
    if let Some(grammar) = m.tabs[m.current_tab].grammars.get(&coord) {
        match grammar.kind.clone() {
            Kind::Text(value) => view_text_grammar(m, &coord, value),
            Kind::Input(value) => {
                let suggestions = m
                    .suggestions
                    .iter()
                    .filter_map(|suggestion_coord| {
                        if let Some(suggestion_grammar) =
                            m.tabs[m.current_tab].grammars.get(&suggestion_coord)
                        {
                            Some((suggestion_coord.clone(), suggestion_grammar.clone()))
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
                        class=format!{"cell row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
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
                        class=format!{"cell row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
                        id=format!{"cell-{}", coord.to_string()}
                        style={ get_style(&m, &coord) }>
                        <input type="range" min={min} max={max} value={value}>
                            { name }
                        </input>
                    </div>
                }
            }
            Kind::Interactive(name, Interactive::Toggle(checked)) => {
                ////info!("Interactive {}-{}",coord.row_to_string(), coord.col_to_string() );
                html! {
                    <div
                        class=format!{"cell row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
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
                let suggestions: Vec<Coordinate> = m.tabs[m.current_tab]
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
            Kind::Lookup(value, lookup_type) => {
                let suggestions : Vec<Coordinate> = m.grammars.keys()
                                                            .filter_map(|lookup_c| if lookup_c.to_string().contains(value.deref()) {
                                                                Some(lookup_c.clone())
                                                            } else { None })
                                                            .collect();
                view_lookup_grammar(m, &coord, suggestions, value, lookup_type, is_active)
            }
        }
    } else {
        // return empty fragment
        html! { <></> }
    }
}
pub fn view_lookup_grammar(
    m: &Model,
    coord: &Coordinate,
    suggestions: Vec<Coordinate>,
    value: String,
    lookup_type: Option<Lookup>,
    is_active: bool,
) -> Html {
    let suggestions_div = if is_active {
        let mut suggestions_nodes = VList::new();
        for lookup_coord in suggestions {
            let dest = coord.clone();
            let source = lookup_coord.clone();
            suggestions_nodes.add_child(html!{
                <a tabindex=-1
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
    let active_cell_class = if is_active {
        "cell-active"
    } else {
        "cell-inactive"
    };
    let c = coord.clone();
    let to_toggle = coord.clone();
    let can_toggle: bool = value.clone().deref() == "";
    html! {
        <div
            class=format!{"cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <b>{ "$" }</b>
            <input
                class={ format!{ "cell-data {}", active_cell_class } }
                placeholder="coordinate"
                value=value
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
            </input>
            { suggestions_div }
        </div>
    }
}

pub fn view_lookup_grammar(
    m: &Model,
    coord: &Coordinate,
    suggestions: Vec<Coordinate>,
    value: String,
    lookup_type: Option<Lookup>,
    is_active: bool
) -> Html {
    let suggestions_div = if is_active {
        let mut suggestions_nodes = VList::new();
        for lookup_coord in suggestions { 
            let dest = coord.clone();
            let source = lookup_coord.clone();
            suggestions_nodes.add_child(html!{
                <a tabindex=-1
                    onclick=m.link.callback(move |_ : ClickEvent| Action::DoCompletion(source.clone(), dest.clone()))>
                    { lookup_coord.to_string() }
                </a>
            })
        }
        html!{
            <div class="suggestion-content">
                { suggestions_nodes }
            </div>
        }
    } else {
        html! { <></> }
    };
    let active_cell_class = if is_active { "cell-active" } else { "cell-inactive" };
    let c = coord.clone();
    let to_toggle = coord.clone();
    let can_toggle : bool = value.clone().deref() == "";
    html! {
        <div
            class=format!{"cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <b>{ "$" }</b>
            <input 
                class={ format!{ "cell-data {}", active_cell_class } }
                placeholder="coordinate"
                value=value
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
            </input>
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
    let active_cell_class = if is_active { "cell-active" } else { "cell-inactive" };
    let suggestions_len = suggestions.len();
    let first_suggestion_ref = NodeRef::default();
    let suggestions = if value.clone() != "" && is_active {
        let mut suggestion_nodes = VList::new();
        let mut is_first_suggestion = true;
        for (s_coord, s_grammar) in suggestions {
            if !s_grammar.name.contains(value.clone().deref()) { continue }
            let c = coord.clone();
            suggestion_nodes.add_child(html! {
                <a 
                    ref={ 
                        if is_first_suggestion {
                            first_suggestion_ref.clone()
                        } else { NodeRef::default() }
                    }
                    onclick=m.link.callback(move |_ : ClickEvent| Action::DoCompletion(s_coord.clone(), c.clone()))>
                    { &s_grammar.name }
                </a>
            });
            is_first_suggestion = false;
        }
        html!{
            <div class="suggestion-content">
                { suggestion_nodes }
            </div>
        }
    } else {
        html!{ <></> }

    };

    let new_active_cell = coord.clone();
    let has_lookup_prefix : bool = value.clone() == "$";
    let current_coord = coord.clone();
    
    html! {
        <div
            class=format!{ "cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string() }
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <input
                class={ format!{ "cell-data {}", active_cell_class } }
                value=value
                ref={ 
                    if is_active { 
                        m.focus_node_ref.clone() 
                    } else { NodeRef::default() } 
                }
                onkeypress=m.link.callback(move |e : KeyPressEvent| {
                    if e.code() == "Tab" && suggestions_len > 0 {
                        if let Some(input) = first_suggestion_ref.try_into::<HtmlElement>() {
                            input.focus();
                        }
                        Action::Noop
                    } else if e.code() == "Space" && has_lookup_prefix {
                        info!{"toggling lookup"}
                        Action::ToggleLookup(current_coord.clone())
                    } else if e.key() == "g" && e.ctrl_key() && is_active {
                        Action::AddNestedGrid(current_coord.clone(), (3, 3))
                    } else { Action::Noop }
                })
                oninput=m.link.callback(move |e : InputData| Action::ChangeInput(coord.clone(), e.value))
                onclick=m.link.callback(move |_ : ClickEvent| Action::SetActiveCell(new_active_cell.clone()))>
            </input>
            { suggestions }
        </div>
    }
}

pub fn view_text_grammar(m: &Model, coord: &Coordinate, value: String) -> Html {
    info!(
        "Text Grammar {}-{}",
        coord.row_to_string(),
        coord.col_to_string()
    );
    html! {
        <div
            class=format!{"cell text row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            { value }
        </div>
    }
}

pub fn view_grid_grammar(m: &Model, coord: &Coordinate, sub_coords: Vec<Coordinate>) -> Html {
    let mut nodes = VList::new();
    for c in sub_coords {
        //info!("View {}", c.to_string());
        nodes.add_child(view_grammar(m, c.clone()));
    }
    // //info!("{}", dbg!(nodes.clone()).to_string);

    html! {
        <div
            class=format!{"cell grid row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            { nodes }
        </div>
    }
}
