use serde::{
    ser::{SerializeSeq, SerializeStruct, SerializeStructVariant, Serializer},
    Deserialize, Serialize,
};
use std::collections::HashMap;
use std::option::Option;

use crate::coordinate::Coordinate;

use crate::grammar::{Grammar, Kind, Interactive};

use crate::style::Style;

// Session encapsulates the serializable state of the application that gets stored to disk
// in a .ise file (which is just a JSON file)
#[derive(Deserialize, Debug, Clone)]
pub struct Session {
    pub title: String,
    pub root: Grammar,
    pub meta: Grammar,
    pub grammars: HashMap<Coordinate, Grammar>,
}
js_serializable!(Session);
js_deserializable!(Session);

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Session", 3)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("root", &self.root)?;
        state.serialize_field("meta", &self.meta)?;
        state.serialize_field("grammars", &self.grammars)?;
        state.end()
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Style", 6)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("border_color", &self.border_color)?;
        state.serialize_field("border_collapse", &self.border_collapse)?;
        state.serialize_field("font_weight", &self.font_weight)?;
        state.serialize_field("font_color", &self.font_color)?;
        state.end()
    }
}

impl Serialize for Grammar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Grammar", 3)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("style", &self.style)?;
        state.serialize_field("kind", &self.kind)?;
        state.end()
    }
}

impl Serialize for Interactive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            Interactive::Button() => {
                let mut sv = serializer.serialize_struct_variant("Interactive", 0, "Button", 1)?;
                sv.serialize_field("button", &())?;
                sv.end()
            }
            Interactive::Slider(val, min, max) => {
                let mut sv = serializer.serialize_struct_variant("Interactive", 1, "Slider", 3)?;
                sv.serialize_field("slider_value", val)?;
                sv.serialize_field("slider_min", min)?;
                sv.serialize_field("slider_max", max)?;
                sv.end()
            }
            Interactive::Toggle(b) => {
                let mut sv = serializer.serialize_struct_variant("Interactive", 2, "Toggle", 1)?;
                sv.serialize_field("toggle_state", b)?;
                sv.end()
            }
        }
    }
}

impl Serialize for Kind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            Kind::Text(s) => {
                let mut sv = serializer.serialize_struct_variant("Kind", 0, "Text", 1)?;
                sv.serialize_field("text", s)?;
                sv.end()
            }
            Kind::Input(s) => {
                let mut sv = serializer.serialize_struct_variant("Kind", 1, "Input", 1)?;
                sv.serialize_field("input", s)?;
                sv.end()
            }
            Kind::Interactive(s, x) => {
                let mut sv = serializer.serialize_struct_variant("Kind", 2, "Interactive", 2)?;
                sv.serialize_field("name", s)?;
                sv.serialize_field("interactive", x)?;
                sv.end()
            }
            Kind::Grid(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for e in v {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Kind::Lookup(s, x) => {
                let mut sv = serializer.serialize_struct_variant("Kind", 2, "Interactive", 2)?;
                sv.serialize_field("raw_value", s)?;
                sv.serialize_field("lookup", x)?;
                sv.end()
            }
        }
    }
}

impl Serialize for Coordinate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        /*
        let mut seq = serializer.serialize_seq(Some(self.row_cols.len()))?;
        for e in self.row_cols.clone() {
            let (a, b) = e;
            let s = format!("{}-{}",&a,&b);
            seq.serialize_element(&s)?;
        }
        seq.end()
        */
        let s = "";
        for e in self.row_cols.clone() {
            let (a, b) = e;
            let s = format!("{}-{}-{}", s, &a, &b);
        }
        serializer.serialize_str(s)
    }
}
