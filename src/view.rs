use std::ops::Deref;
use yew::{html, ChangeData, Html, InputData};
use yew::events::ClickEvent;
use yew::virtual_dom::{VList};

use crate::model::{Action,Model,SideMenu};
use crate::grammar::{Grammar, Kind, Interactive};
use crate::coordinate::Coordinate;
use crate::style::get_style;


pub fn view_side_nav(m: &Model) -> Html {
    let mut side_menu_nodes = VList::new();
    let mut side_menu_section = html! { <></> };
    for (index, side_menu) in m.side_menus.iter().enumerate() {
        if Some(index as i32) == m.open_side_menu {
            side_menu_nodes.add_child(html! {
                <button class="active-menu" onclick=m.link.callback(|e| Action::SetActiveMenu(None))>
                    <img 
                        src={side_menu.icon_path.clone()} 
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
        },
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
                    <input type="file" onchange=m.link.callback(|value| {
                        if let ChangeData::Files(files) = value {
                            if files.len() >= 1 {
                                if let Some(file) = files.iter().nth(0) {
                                    return Action::SaveSession();
                                }
                            }
                        }
                        Action::Noop
                    })>
                        
                    </input>
                </div>
            } 
        },
        "Settings" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS Settings MENU"}
                </div>
            } 
        },
        "Info" => {
            html! {
                <div class="side-menu-section">
                    {"THIS IS info MENU"}
                </div>
            } 
        },

        _ => html! {<> </>}

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
            <button class="menu-bar-button" onclick=m.link.callback(|_| Action::Noop) >
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
            <button class="menu-bar-button">
                { "Delete Row" }
            </button>
            <button class="menu-bar-button">
                { "Delete Column" }
            </button>
        </div>
    }
}

pub fn view_tab_bar(m: &Model) -> Html {
    let mut tabs = VList::new();
    for (index, tab) in m.tabs.clone().iter().enumerate() {
        if (index as i32) == m.current_tab {
            tabs.add_child(html! {
                <button class="tab active-tab">{ tab }</button>
            });
        } else {
            tabs.add_child(html! {
                <button class="tab">{ tab }</button>
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
    if let Some(grammar) = m.grammars.get(&coord) {
        match grammar.kind.clone() {
            Kind::Text(value) => {
                view_text_grammar(m, &coord, value)
            }
            Kind::Input(value) => {
                let is_active = m.active_cell.clone() == Some(coord.clone());
                let suggestions = m.suggestions.iter().filter_map(|suggestion_coord| {
                    if let Some(suggestion_grammar) = m.grammars.get(&suggestion_coord) {
                        Some((suggestion_coord.clone(), suggestion_grammar.clone()))
                    } else {
                        None
                    }
                }).collect();
                view_input_grammar(
                    m,
                    coord.clone(),
                    suggestions,
                    value,
                    is_active,
                )
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
            Kind::Grid(sub_coords) => {
                view_grid_grammar(
                    m,
                    &coord,
                    sub_coords.iter().map(|c| Coordinate::child_of(&coord, *c)).collect(),
                )
            }
        }
    } else {
        // return empty fragment
        html! { <></> }
    }
}

pub fn view_input_grammar(
    m: &Model,
    coord: Coordinate,
    suggestions: Vec<(Coordinate, Grammar)>,
    value: String,
    is_active: bool,
) -> Html {
    let mut suggestion_nodes = VList::new();
    let mut active_cell_class = "cell-inactive";
    if is_active {
        active_cell_class = "cell-active";
        for (s_coord, s_grammar) in suggestions {
            let c = coord.clone();
            suggestion_nodes.add_child(html! {
                <a 
                    tabindex=-1
                    onclick=m.link.callback(move |_ : ClickEvent| Action::DoCompletion(s_coord.clone(), c.clone()))>
                    { &s_grammar.name }
                </a>
            })
            
        }
    }
    let suggestions = html!{
        <div class="suggestion-content">
            { suggestion_nodes }
        </div>
    };

    let new_active_cell = coord.clone();

    html! {
        <div
            class=format!{"cell suggestion row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            <input 
                class={ format!{ "cell-data {}", active_cell_class } }
                value=value
                oninput=m.link.callback(move |e : InputData| Action::ChangeInput(coord.clone(), e.value))
                onclick=m.link.callback(move |_ : ClickEvent| Action::SetActiveCell(new_active_cell.clone()))>
            </input>
            
            { suggestions }
        </div>
    }
}

pub fn view_text_grammar(m: &Model, coord: &Coordinate, value : String) -> Html {
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
        nodes.add_child(view_grammar(m, c.clone()));
    }

    html! {
        <div
            class=format!{"cell grid row-{} col-{}", coord.row_to_string(), coord.col_to_string()}
            id=format!{"cell-{}", coord.to_string()}
            style={ get_style(&m, &coord) }>
            { nodes }
        </div>
    }
}