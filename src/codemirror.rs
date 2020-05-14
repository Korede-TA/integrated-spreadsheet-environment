use crate::coordinate::*;
use stdweb::web::{html_element::TextAreaElement, IHtmlElement};
use yew::prelude::*;

pub struct CodeMirror {
    code: String,
    mode: String,
    coordinate: Coordinate,
    link: ComponentLink<Self>,
    node_ref: NodeRef,
}

pub enum Msg {
    UpdateCode(String),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub content: String,
    pub mode: String,
    pub coordinate: Coordinate,
}

impl Component for CodeMirror {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        CodeMirror {
            code: props.content,
            mode: props.mode,
            coordinate: props.coordinate,
            link,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateCode(code) => {
                self.code = code;
                true
            }
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        /// TODO: this code runs the codemirror instantiation, but it doesn't work too well.
        /// leave it disabled for now, so we'll just have a regular textarea in it's place
        // if let Some(input) = self.node_ref.cast::<TextAreaElement>() {
        //     input.focus();
        // }
        // let element_id = format! {"codemirror-{}", self.coordinate.to_string()};
        // js! {
        //     let codeMirrorInstance = CodeMirror.fromTextArea(document.getElementById(@{element_id}), {
        //         lineNumbers: true,
        //         mode: "python"
        //     });
        //     codeMirrorInstance.setValue(@{&self.code});
        //     setTimeout(function() {
        //         codeMirrorInstance.refresh();
        //         console.log("codemirror instance setup finished!");
        //     },1);
        // }
        false
    }

    fn view(&self) -> Html {
        html! {
            <textarea
                value={self.code.clone()}
                ref={self.node_ref.clone()}
                id=format!{"codemirror-{}", self.coordinate.to_string()}
                oninput=self.link.callback(move |e: InputData| Msg::UpdateCode(e.value))>
            </textarea>
        }
    }
}
