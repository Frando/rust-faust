use faust_types::*;
use rtrb::{Consumer, Producer, RingBuffer};
use std::{
    collections::{BTreeMap, HashMap},
    ops::RangeInclusive,
};

const DEFAULT_NAME: &str = "rust_faust";

#[derive(Debug)]
pub struct DspHandle<T> {
    dsp: Box<T>,
    dsp_tx: Producer<State>,
    dsp_rx: Consumer<State>,
    name: String,
}

impl<T> DspHandle<T>
where
    T: FaustDsp<T = f32> + 'static,
{
    pub fn new() -> (Self, StateHandle) {
        let dsp = Box::new(T::new());
        Self::from_dsp(dsp)
    }

    pub fn from_dsp(mut dsp: Box<T>) -> (Self, StateHandle) {
        let meta = MetaBuilder::from_dsp(dsp.as_mut());
        let params = ParamsBuilder::from_dsp(dsp.as_mut());
        let name = meta
            .get("name")
            .map_or(DEFAULT_NAME, String::as_str)
            .to_string();

        let (dsp_tx, main_rx) = RingBuffer::new(1).split();
        let (main_tx, dsp_rx) = RingBuffer::new(1).split();

        let this = {
            Self {
                name: name.clone(),
                dsp,
                dsp_tx,
                dsp_rx,
            }
        };
        let mut state = State {
            updates: HashMap::with_capacity(params.len()),
            state: HashMap::with_capacity(params.len()),
        };

        let mut params_by_path = BTreeMap::new();
        for (idx, node) in params.iter() {
            params_by_path.insert(node.path(), *idx);
            state.state.insert(*idx, node.widget_type().init_value());
        }

        let state_handle = StateHandle {
            name,
            meta,
            params,
            params_by_path,
            state,
            main_tx,
            main_rx,
        };
        (this, state_handle)
    }

    pub fn update_and_compute(
        &mut self,
        count: i32,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) {
        let mut state = if let Ok(state) = self.dsp_rx.pop() {
            self.update_params_from_state(&state);
            Some(state)
        } else {
            None
        };

        // Potentially improves the performance of SIMD floating-point math
        // by flushing denormals/underflow to zero.
        // See: https://gist.github.com/GabrielMajeri/545042ee4f956d5b2141105eb6a505a9
        // See: https://github.com/grame-cncm/faust/blob/master-dev/architecture/faust/dsp/dsp.h#L236
        let mask = if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
            1 << 24 // FZ
        } else if cfg!(any(target_feature = "sse2")) {
            0x8040
        } else if cfg!(any(target_feature = "sse")) {
            0x8000
        } else {
            0x0000
        };
        // Set fp status register to masked value
        let fpsr = self.get_fp_status_register();
        if let Some(fpsr) = fpsr {
            self.set_fp_status_register(fpsr | mask);
        }

        self.compute(count, inputs, outputs);

        // Reset fp status register to old value
        if let Some(fpsr) = fpsr {
            self.set_fp_status_register(fpsr);
        }

        if !self.dsp_tx.is_full() && state.is_some() {
            let mut state = state.take().unwrap();
            self.update_state_from_params(&mut state);
            let _ = self.dsp_tx.push(state);
        }
    }

    // Gets the fp status register.
    // Needed for flushing denormals
    fn get_fp_status_register(&self) -> Option<u32> {
        unsafe {
            if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
                use std::arch::asm;
                let fspr: u32;
                asm!("msr fpcr, {0:r}", out(reg) fspr);
                Some(fspr)
            } else if cfg!(target_feature = "sse") {
                use std::arch::x86_64::*;
                Some(_mm_getcsr())
            } else {
                None
            }
        }
    }

    // Sets the fp status register.
    // Needed for flushing denormals
    fn set_fp_status_register(&self, fspr: u32) {
        unsafe {
            if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
                use std::arch::asm;
                asm!("mrs {0:r}, fpcr", in(reg) fspr);
            } else if cfg!(target_feature = "sse") {
                use std::arch::x86_64::*;
                _mm_setcsr(fspr)
            }
        }
    }

    pub fn update_params_from_state(&mut self, state: &State) {
        for (idx, value) in state.updates.iter() {
            let idx = ParamIndex(*idx);
            self.dsp.set_param(idx, *value);
        }
    }

    pub fn update_state_from_params(&self, state: &mut State) {
        for (idx, value) in state.state.iter_mut() {
            let idx = ParamIndex(*idx);
            if let Some(new_value) = self.dsp.get_param(idx) {
                *value = new_value;
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // fn get_param(&self, param: ParamIndex) -> Option<Self::T>;
    // fn set_param(&mut self, param: ParamIndex, value: Self::T);
    pub fn compute(&mut self, count: i32, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
        self.dsp.compute(count, inputs, outputs)
    }

    pub fn num_inputs(&self) -> usize {
        self.dsp.get_num_inputs() as usize
    }

    pub fn num_outputs(&self) -> usize {
        self.dsp.get_num_outputs() as usize
    }

    pub fn init(&mut self, sample_rate: i32) {
        self.dsp.init(sample_rate)
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub state: HashMap<i32, f32>,
    pub updates: HashMap<i32, f32>,
}

impl State {
    pub fn insert(&mut self, idx: i32, value: f32) {
        self.updates.insert(idx, value);
        self.state.insert(idx, value);
    }
}

#[derive(Debug)]
pub struct StateHandle {
    name: String,
    pub state: State,
    meta: HashMap<String, String>,
    params: HashMap<i32, Node>,
    params_by_path: BTreeMap<String, i32>,
    main_rx: Consumer<State>,
    main_tx: Producer<State>,
}

impl StateHandle {
    pub fn set_param(&mut self, idx: i32, value: f32) {
        self.state.insert(idx, value);
    }

    pub fn get_param(&self, idx: i32) -> Option<&f32> {
        self.state.state.get(&idx)
    }

    pub fn set_by_path(&mut self, path: &str, value: f32) -> Result<(), String> {
        let idx = if let Some(idx) = self.params_by_path.get(path) {
            Some(*idx)
        } else {
            return Err("No such path".into());
        };
        if let Some(idx) = idx {
            self.set_param(idx, value);
        }
        Ok(())
    }

    pub fn get_by_path(&self, path: &str) -> Option<&f32> {
        if let Some(idx) = self.params_by_path.get(path) {
            self.get_param(*idx)
        } else {
            None
        }
    }

    pub fn send(&mut self) {
        self.update();
    }

    pub fn update(&mut self) {
        if let Ok(state) = self.main_rx.pop() {
            self.state.state = state.state;
        }
        if !self.main_tx.is_full() {
            let state = self.state.clone();
            if let Err(e) = self.main_tx.push(state) {
                eprintln!("error sending state update: {}", e);
            } else {
                self.state.updates.clear();
            }
        }
    }

    pub fn params(&self) -> &HashMap<i32, Node> {
        &self.params
    }

    pub fn params_by_path(&self) -> impl Iterator<Item = (&String, Option<&f32>)> {
        self.params_by_path
            .iter()
            .map(move |(path, idx)| (path, self.get_param(*idx).clone()))
    }

    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

struct MetaBuilder {
    inner: HashMap<String, String>,
}

impl MetaBuilder {
    fn from_dsp<T>(dsp: &dyn FaustDsp<T = T>) -> HashMap<String, String> {
        let mut metadata = Self {
            inner: HashMap::new(),
        };
        dsp.metadata(&mut metadata);
        metadata.inner
    }
}

impl faust_types::Meta for MetaBuilder {
    fn declare(&mut self, key: &str, value: &str) {
        self.inner.insert(key.into(), value.into());
    }
}

#[derive(Debug)]
struct ParamsBuilder {
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
    typ: WidgetType,
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

    pub fn widget_type(&self) -> &WidgetType {
        &self.typ
    }
}

/// General types of widgets declared in the DSP
#[derive(Debug, Clone)]
pub enum WidgetType {
    /// Only has metadata
    /// There should not be any after building the DSP.
    Unknown,
    /// Temporary on button.
    Button,
    /// Stable on/off button.
    Toggle,
    /// Vertical slider
    VerticalSlider(RangedInput),
    /// Horizontal slider
    HorizontalSlider(RangedInput),
    /// Numeric entry
    NumEntry(RangedInput),
    /// Horizontal bargraph
    HorizontalBarGraph(RangedOutput),
    /// Vertical bargraph
    VerticalBargraph(RangedOutput),
}

impl Default for WidgetType {
    fn default() -> Self {
        WidgetType::Unknown
    }
}

impl WidgetType {
    /// Retrieve the init value for this widget
    pub fn init_value(&self) -> f32 {
        match self {
            WidgetType::VerticalSlider(input) => input.init,
            WidgetType::HorizontalSlider(input) => input.init,
            WidgetType::NumEntry(input) => input.init,
            // Buttons and checkboxes are off by default.
            // Passive widgets will need an update from the DSP before having a value
            _ => 0.0,
        }
    }
}

/// A ranged input controlled by the user.
#[derive(Debug, Clone)]
pub struct RangedInput {
    /// Initial value defined in the DSP
    pub init: f32,
    /// Available range defined in the DSP
    /// This range is declared but not enforced
    pub range: RangeInclusive<f32>,
    /// Precision of the value
    /// This value is declared but not enforced
    pub step: f32,
}

impl RangedInput {
    pub fn new(init: f32, min: f32, max: f32, step: f32) -> Self {
        Self {
            init,
            range: min..=max,
            step,
        }
    }
}

/// A ranged output value controlled by the DSP.
#[derive(Debug, Clone)]
pub struct RangedOutput {
    /// Declared range of the widget
    /// This value is declared but not enforced
    pub range: RangeInclusive<f32>,
}

impl RangedOutput {
    pub fn new(min: f32, max: f32) -> Self {
        Self { range: min..=max }
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
    fn from_dsp<D>(dsp: &mut D) -> HashMap<i32, Node>
    where
        D: FaustDsp<T = f32>,
    {
        let mut builder = Self::new();
        dsp.build_user_interface(&mut builder);
        builder.inner
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
        typ: WidgetType,
        metadata: Option<Vec<[String; 2]>>,
    ) {
        let prefix = self.prefix[..].join("/").to_string();
        let idx = idx.0;
        if self.inner.contains_key(&idx) {
            let node = self.inner.get_mut(&idx).unwrap();
            node.label = label.to_string();
            node.typ = typ;
            if let Some(mut metadata) = metadata {
                node.metadata.append(metadata.as_mut());
            }
        } else {
            let node = Node {
                idx,
                label: label.to_string(),
                prefix,
                typ,
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
        self.add_or_update_widget(label, param, WidgetType::Button, None);
    }
    fn add_check_button(&mut self, label: &str, param: ParamIndex) {
        self.add_or_update_widget(label, param, WidgetType::Toggle, None);
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
        let typ = WidgetType::VerticalSlider(RangedInput::new(init, min, max, step));
        self.add_or_update_widget(label, param, typ, None);
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
        let typ = WidgetType::HorizontalSlider(RangedInput::new(init, min, max, step));
        self.add_or_update_widget(label, param, typ, None);
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
        let typ = WidgetType::NumEntry(RangedInput::new(init, min, max, step));
        self.add_or_update_widget(label, param, typ, None);
    }

    // -- passive widgets
    fn add_horizontal_bargraph(&mut self, label: &str, param: ParamIndex, min: f32, max: f32) {
        let typ = WidgetType::HorizontalBarGraph(RangedOutput::new(min, max));
        self.add_or_update_widget(label, param, typ, None);
    }
    fn add_vertical_bargraph(&mut self, label: &str, param: ParamIndex, min: f32, max: f32) {
        let typ = WidgetType::VerticalBargraph(RangedOutput::new(min, max));
        self.add_or_update_widget(label, param, typ, None);
    }

    // -- metadata declarations
    fn declare(&mut self, param: Option<ParamIndex>, key: &str, value: &str) {
        if let Some(param_index) = param {
            if !self.inner.contains_key(&param_index.0) {
                self.add_or_update_widget(
                    "Unknown",
                    param_index,
                    WidgetType::default(),
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
