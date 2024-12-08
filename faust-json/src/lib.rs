use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faust {
    pub name: String,
    pub filename: String,
    pub version: String,
    pub compile_options: String,
    #[serde(default)] //allow empty list
    pub library_list: Vec<String>,
    #[serde(default)] //allow empty list
    pub include_pathnames: Vec<String>,
    pub size: u32,
    pub inputs: usize,
    pub outputs: usize,
    pub author: Option<String>,
    pub license: Option<String>,
    pub copyright: Option<String>,
    pub classname: Option<String>,
    #[serde(default)] //allow empty list
    pub meta: Vec<Meta>,
    #[serde(default)] //allow empty list
    pub ui: Vec<LayoutItem>,
}

#[derive(Debug, PartialEq)]
pub struct Meta {
    pub key: String,
    pub value: String,
}

impl<'de> Deserialize<'de> for Meta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: std::collections::HashMap<String, Option<String>> =
            Deserialize::deserialize(deserializer).unwrap();
        let Some((key, Some(value))): Option<(&String, &Option<String>)> = map.iter().next() else {
            panic!("bla")
        };

        Ok(Meta {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum LayoutItem {
    TGroup {
        label: String,
        items: Vec<LayoutItem>,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    VGroup {
        label: String,
        items: Vec<LayoutItem>,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    HGroup {
        label: String,
        items: Vec<LayoutItem>,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    VSlider {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        init: f32,
        min: f32,
        max: f32,
        step: f32,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    HSlider {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        init: f32,
        min: f32,
        max: f32,
        step: f32,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    NEntry {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        init: Option<f32>,
        min: f32,
        max: f32,
        step: f32,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    Button {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        init: Option<f32>,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    CheckBox {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        init: Option<f32>,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    VBarGraph {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        min: f32,
        max: f32,
        #[serde(default)]
        meta: Vec<Meta>,
    },
    HBarGraph {
        label: String,
        shortname: String,
        address: String,
        varname: String,
        min: f32,
        max: f32,
        #[serde(default)]
        meta: Vec<Meta>,
    },

    Soundfile {
        label: String,
        url: String,
        address: String,
        varname: String,
        #[serde(default)]
        meta: Vec<Meta>,
    },
}
