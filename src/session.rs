use serde::{
    de::Error,
    ser::{SerializeStruct, SerializeStructVariant, SerializeTupleVariant, Serializer},
    Deserialize, Deserializer, Serialize,
};
use std::collections::HashMap;
use std::option::Option;

use crate::coord;
use crate::coordinate::Coordinate;
use crate::grammar::{Grammar, Interactive, Kind};
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

// Session Custom Serialization
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

// Need coordinateParser and its derive for creating a coordinate during deserialization
#[derive(Parser)]
#[grammar = "coordinate.pest"]
pub struct CoordinateParser;
// Coordinate Custom Deserialization
impl<'de> Deserialize<'de> for Coordinate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Imports for the Macro coord! inn this scope
        use crate::util::non_zero_u32_tuple;
        use pest::Parser;
        use std::num::NonZeroU32;
        use std::panic;

        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(coord!(s))
    }
}

// Style Custom Serialization
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
        state.serialize_field("col_span", &self.col_span)?;
        state.serialize_field("row_span", &self.row_span)?;
        state.serialize_field("display", &self.display)?;
        state.end()
    }
}

// Grammar Custom Serialization
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

// Interactive Custom Serialization
impl Serialize for Interactive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            Interactive::Button() => {
                let mut sv = serializer.serialize_tuple_variant("Interactive", 0, "Button", 0)?;
                sv.end()
            }
            Interactive::Slider(val, min, max) => {
                let mut sv = serializer.serialize_tuple_variant("Interactive", 1, "Slider", 3)?;
                sv.serialize_field(val)?;
                sv.serialize_field(min)?;
                sv.serialize_field(max)?;
                sv.end()
            }
            Interactive::Toggle(b) => {
                let mut sv = serializer.serialize_struct("Interactive", 1)?;
                sv.serialize_field("Toggle", b)?;
                sv.end()
            }
        }
    }
}

// kind Custom Serialization
impl Serialize for Kind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            Kind::Text(s) => {
                let mut sv = serializer.serialize_struct("kind", 1)?;
                sv.serialize_field("Text", s)?;
                sv.end()
            }
            Kind::Input(s) => {
                let mut sv = serializer.serialize_struct("kind", 1)?;
                sv.serialize_field("Input", s)?;
                sv.end()
            }
            Kind::Interactive(s, x) => {
                let mut sv = serializer.serialize_tuple_variant("kind", 0, "Interactive", 2)?;
                sv.serialize_field(s)?;
                sv.serialize_field(x)?;
                sv.end()
            }
            Kind::Grid(v) => {
                let mut sv = serializer.serialize_struct("kind", 1)?;
                sv.serialize_field("Grid", v)?;
                sv.end()
            }
            Kind::Lookup(s, x) => {
                let mut sv = serializer.serialize_struct_variant("kind", 1, "Lookup", 2)?;
                sv.serialize_field("raw_value", s)?;
                sv.serialize_field("lookup", x)?;
                sv.end()
            }
            Kind::Defn(s, c, rules) => {
                let mut sv = serializer.serialize_struct_variant("kind", 2, "Defn", 3)?;
                sv.serialize_field("name", s)?;
                sv.serialize_field("coordinate", c)?;
                sv.serialize_field("rules", rules)?;
                sv.end()
            }
        }
    }
}

// Coordinate Custom Serialization
impl Serialize for Coordinate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
