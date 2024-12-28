#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
    unused_crate_dependencies,
    clippy::unwrap_used
)]

use quick_xml::impl_deserialize_for_internally_tagged_enum;
use serde::Deserialize;
#[cfg(test)]
use serde_path_to_error as _;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaustXML {
    pub name: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub copyright: Option<String>,
    pub version: Option<String>,
    pub classname: Option<String>,
    pub inputs: usize,
    pub outputs: usize,
    #[serde(default)]
    pub meta: Vec<Meta>,
    pub ui: UI,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[derive(PartialEq, Eq)]
pub struct Meta {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UI {
    pub activewidgets: ActiveWidgets,
    pub passivewidgets: PassiveWidgets,
    pub soundfilewidgets: SoundfileWidgets,
    pub layout: Layout,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveWidgets {
    pub count: usize,
    #[serde(default)]
    pub widget: Vec<ActiveWidget>,
}

#[derive(Debug, PartialEq)]
pub enum ActiveWidget {
    VSlider(VSlider),
    HSlider(HSlider),
    NEntry(NEntry),
    Button(Button),
    CheckBox(CheckBox),
}

impl_deserialize_for_internally_tagged_enum! {
    ActiveWidget, "@type",
    ("vslider"    => VSlider(VSlider)),
    ("hslider"    => HSlider(HSlider)),
    ("nentry" => NEntry(NEntry)),
    ("button" => Button(Button)),
    ("checkbox"  => CheckBox(CheckBox)),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VSlider {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub init: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    #[serde(default)]
    pub meta: Vec<Meta>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HSlider {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub init: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    #[serde(default)]
    pub meta: Vec<Meta>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NEntry {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub init: Option<f32>,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    #[serde(default)]
    pub meta: Vec<Meta>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Button {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub init: Option<f32>,
    #[serde(default)]
    pub meta: Vec<Meta>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CheckBox {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub init: Option<f32>,
    #[serde(default)]
    pub meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassiveWidgets {
    pub count: usize,
    #[serde(default)]
    pub widget: Vec<PassiveWidget>,
}

#[derive(Debug, PartialEq)]
pub enum PassiveWidget {
    VBarGraph(VBarGraph),
    HBarGraph(HBarGraph),
}

impl_deserialize_for_internally_tagged_enum! {
    PassiveWidget, "@type",
    ("vbargraph"    => VBarGraph(VBarGraph)),
    ("hbargraph"    => HBarGraph(HBarGraph)),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VBarGraph {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub min: f32,
    pub max: f32,
    #[serde(default)]
    pub meta: Vec<Meta>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HBarGraph {
    #[serde(rename = "@id")]
    pub id: usize,
    pub label: String,
    pub varname: String,
    pub min: f32,
    pub max: f32,
    #[serde(default)]
    pub meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoundfileWidgets {
    pub count: usize,
    #[serde(rename = "$soundfiles", default)]
    pub soundfiles: Vec<SoundfileWidgets>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoundfileWidget {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    #[serde(rename = "$value")]
    pub items: Vec<LayoutItem>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LayoutItem {
    #[serde(rename = "group")]
    Group(Group),
    #[serde(rename = "widgetref")]
    WRef(WRef),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[serde(rename = "@type")]
    pub r#type: String,
    pub label: String,
    #[serde(rename = "$value", default)]
    pub items: Vec<LayoutItem>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WRef {
    #[serde(rename = "@id")]
    pub id: usize,
}
