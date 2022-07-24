use faust_types::*;
use rtrb::{Consumer, Producer, RingBuffer};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

const DEFAULT_NAME: &str = "rust_faust";

#[derive(Debug)]
pub struct Dsp<T> {
    dsp: T,
    name: String,
}

impl<T> Dsp<T>
where
    T: FaustDsp<T = f32> + 'static,
{
    pub fn new() -> (Self, Metadata, Parameters) {
        let mut dsp = T::new();
        let metadata = Metadata::from_dsp(&mut dsp);
        let parameters = Parameters::from_dsp(&mut dsp);
        let name = metadata
            .get("name")
            .map_or(DEFAULT_NAME, String::as_str)
            .to_string();

        let this = {
            Self {
                name: name.clone(),
                dsp,
            }
        };

        return (this, metadata, parameters);
    }
}

pub struct Metadata {
    inner: HashMap<String, String>,
}

impl Metadata {
    fn get(&self, key: &str) -> Option<&String> {
        return self.inner.get(key);
    }

    fn from_dsp<T>(dsp: &dyn FaustDsp<T = T>) -> Self {
        MetaBuilder::from_dsp(dsp).to_metadata()
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }

}

pub struct MetaBuilder {
    inner: HashMap<String, String>,
}

impl MetaBuilder {
    fn from_dsp<T>(dsp: &dyn FaustDsp<T = T>) -> Self {
        let mut metabuilder = Self {
            inner: HashMap::new(),
        };
        dsp.metadata(&mut metabuilder);
        metabuilder

    }

    fn to_metadata(self) -> Metadata {
        Metadata {
            inner: self.inner
        }

    }

}

impl faust_types::Meta for MetaBuilder {
    fn declare(&mut self, key: &str, value: &str) {
        self.inner.insert(key.into(), value.into());
    }
}

pub struct Parameters {
    inner: HashMap<i32, Node>
}

impl fmt::Debug for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }

}

impl Parameters {
    fn from_dsp<D>(dsp: &mut D) -> Self
    where
        D: FaustDsp<T = f32>,
    {
        ParamsBuilder::from_dsp(dsp).to_parameters()
    }

    fn get(&self, key: &i32) -> Option<&Node> {
        return self.inner.get(key);
    }

}

#[derive(Debug)]
pub struct ParamsBuilder {
    inner: HashMap<i32, Node>,
    prefix: Vec<String>,
    first_group: bool,
}

#[derive(Debug)]
struct Params {
    map: HashMap<i32, Node>,
    prefix: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    idx: i32,
    label: String,
    prefix: String,
    typ: NodeType,
    props: Option<Props>,
    metadata: Vec<[String; 2]>,
}

impl Node {
    pub fn path(&self) -> String {
        let mut path = self.prefix.clone();
        if !path.is_empty() {
            path += "/";
        }
        path += &self.label;
        path
    }

    pub fn init_value(&self) -> f32 {
        if let Some(props) = &self.props {
            props.init
        } else {
            0.
        }
    }
}

#[derive(Debug, Clone)]
pub struct Props {
    min: f32,
    max: f32,
    init: f32,
    step: f32,
}

#[derive(Debug, Clone)]
enum NodeType {
    Value,
    Button,
    Toggle,
    Input,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Value
    }
}

impl ParamsBuilder {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
            first_group: true,
            prefix: Vec::new(),
            // state: Vec::new(),
        }
    }
    fn from_dsp<D>(dsp: &mut D) -> ParamsBuilder
    where
        D: FaustDsp<T = f32>,
    {
        let mut builder = Self::new();
        dsp.build_user_interface(&mut builder);
        builder
    }

    fn to_parameters(self) -> Parameters {
        Parameters {
            inner: self.inner
        }
    }

    fn open_group(&mut self, label: &str) {
        if self.first_group {
            self.first_group = false;
        } else {
            self.prefix.push(label.into());
        }
    }
    fn close_group(&mut self) {
        self.prefix.pop();
    }

    fn add_or_update_widget(
        &mut self,
        label: &str,
        idx: ParamIndex,
        typ: NodeType,
        props: Option<Props>,
        metadata: Option<Vec<[String; 2]>>,
    ) {
        let prefix = self.prefix[..].join("/").to_string();
        let idx = idx.0;
        if self.inner.contains_key(&idx) {
            let node = self.inner.get_mut(&idx).unwrap();
            node.label = label.to_string();
            node.typ = typ;
            if props.is_some() {
                node.props = props;
            }
            if let Some(mut metadata) = metadata {
                node.metadata.append(metadata.as_mut());
            }
        } else {
            let node = Node {
                idx,
                label: label.to_string(),
                prefix,
                typ,
                props,
                metadata: metadata.unwrap_or_default(),
            };
            self.inner.insert(idx, node);
        }
    }
}

impl UI<f32> for ParamsBuilder {
    fn open_tab_box(&mut self, label: &str) {
        self.open_group(label);
    }
    fn open_horizontal_box(&mut self, label: &str) {
        self.open_group(label);
    }
    fn open_vertical_box(&mut self, label: &str) {
        self.open_group(label);
    }
    fn close_box(&mut self) {
        self.close_group()
    }

    // -- active widgets
    fn add_button(&mut self, label: &str, param: ParamIndex) {
        self.add_or_update_widget(label, param, NodeType::Button, None, None);
    }
    fn add_check_button(&mut self, label: &str, param: ParamIndex) {
        self.add_or_update_widget(label, param, NodeType::Toggle, None, None);
    }
    fn add_vertical_slider(
        &mut self,
        label: &str,
        param: ParamIndex,
        init: f32,
        min: f32,
        max: f32,
        step: f32,
    ) {
        let typ = NodeType::Input;
        let props = Props {
            init,
            min,
            max,
            step,
        };
        self.add_or_update_widget(label, param, typ, Some(props), None);
    }
    fn add_horizontal_slider(
        &mut self,
        label: &str,
        param: ParamIndex,
        init: f32,
        min: f32,
        max: f32,
        step: f32,
    ) {
        let typ = NodeType::Input;
        let props = Props {
            init,
            min,
            max,
            step,
        };
        self.add_or_update_widget(label, param, typ, Some(props), None);
    }
    fn add_num_entry(
        &mut self,
        label: &str,
        param: ParamIndex,
        init: f32,
        min: f32,
        max: f32,
        step: f32,
    ) {
        let typ = NodeType::Input;
        let props = Props {
            init,
            min,
            max,
            step,
        };
        self.add_or_update_widget(label, param, typ, Some(props), None);
    }

    // -- passive widgets
    fn add_horizontal_bargraph(&mut self, label: &str, param: ParamIndex, min: f32, max: f32) {
        let typ = NodeType::Value;
        let props = Props {
            init: 0.,
            min,
            max,
            step: 0.,
        };
        self.add_or_update_widget(label, param, typ, Some(props), None);
    }
    fn add_vertical_bargraph(&mut self, label: &str, param: ParamIndex, min: f32, max: f32) {
        let typ = NodeType::Value;
        let props = Props {
            init: 0.,
            min,
            max,
            step: 0.,
        };
        self.add_or_update_widget(label, param, typ, Some(props), None);
    }

    // -- metadata declarations
    fn declare(&mut self, param: Option<ParamIndex>, key: &str, value: &str) {
        if let Some(param_index) = param {
            if !self.inner.contains_key(&param_index.0) {
                self.add_or_update_widget(
                    "Unknown",
                    param_index,
                    NodeType::default(),
                    None,
                    Some(vec![[key.to_string(), value.to_string()]]),
                )
            } else {
                if let Some(node) = self.inner.get_mut(&param_index.0) {
                    node.metadata.push([key.to_string(), value.to_string()]);
                }
            }
        }
    }
}

