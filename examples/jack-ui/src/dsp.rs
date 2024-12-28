#![allow(clippy::all)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]
use faust_types::*;
pub type FaustFloat = F32;
mod ffi {
    use std::os::raw::c_float;
    #[cfg_attr(not(target_os = "windows"), link(name = "m"))]
    extern "C" {
        pub fn remainderf(from: c_float, to: c_float) -> c_float;
        pub fn rintf(val: c_float) -> c_float;
    }
}
fn remainder_f32(from: f32, to: f32) -> f32 {
    unsafe { ffi::remainderf(from, to) }
}
fn rint_f32(val: f32) -> f32 {
    unsafe { ffi::rintf(val) }
}
pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 1;
pub const FAUST_PASSIVES: usize = 1;
#[cfg_attr(feature = "default-boxed", derive(default_boxed::DefaultBoxed))]
#[repr(C)]
pub struct Volume {
    fSampleRate: i32,
    fConst0: F32,
    fConst1: F32,
    fConst2: F32,
    fVslider0: F32,
    fRec0: [F32; 2],
    fConst3: F32,
    fRec1: [F32; 2],
    fVbargraph0: F32,
    fConst4: F32,
}
impl Volume {
    pub fn new() -> Volume {
        Volume {
            fSampleRate: 0,
            fConst0: 0.0,
            fConst1: 0.0,
            fConst2: 0.0,
            fVslider0: 0.0,
            fRec0: [0.0; 2],
            fConst3: 0.0,
            fRec1: [0.0; 2],
            fVbargraph0: 0.0,
            fConst4: 0.0,
        }
    }
    pub fn metadata(&self, m: &mut dyn Meta) {
        m.declare("author", r"Franz Heinzmann");
        m.declare("basics.lib/name", r"Faust Basic Element Library");
        m.declare(
            "basics.lib/tabulateNd",
            r"Copyright (C) 2023 Bart Brouns <bart@magnetophon.nl>",
        );
        m.declare("basics.lib/version", r"1.21.0");
        m.declare(
            "compile_options",
            r"-lang rust -ct 1 -cn Volume -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0",
        );
        m.declare("filename", r"volume.dsp");
        m.declare("license", r"BSD");
        m.declare("maths.lib/author", r"GRAME");
        m.declare("maths.lib/copyright", r"GRAME");
        m.declare("maths.lib/license", r"LGPL with exception");
        m.declare("maths.lib/name", r"Faust Math Library");
        m.declare("maths.lib/version", r"2.8.1");
        m.declare("name", r"volume");
        m.declare("options", r"[osc:on]");
        m.declare("platform.lib/name", r"Generic Platform Library");
        m.declare("platform.lib/version", r"1.3.0");
        m.declare("signals.lib/name", r"Faust Signal Routing Library");
        m.declare("signals.lib/version", r"1.6.0");
        m.declare("version", r"1.0");
    }
    pub fn get_sample_rate(&self) -> i32 {
        self.fSampleRate as i32
    }
    pub fn class_init(sample_rate: i32) {}
    pub fn instance_reset_params(&mut self) {
        self.fVslider0 = 0.0;
    }
    pub fn instance_clear(&mut self) {
        for l0 in 0..2 {
            self.fRec0[l0 as usize] = 0.0;
        }
        for l1 in 0..2 {
            self.fRec1[l1 as usize] = 0.0;
        }
    }
    pub fn instance_constants(&mut self, sample_rate: i32) {
        self.fSampleRate = sample_rate;
        self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
        self.fConst1 = 44.1 / self.fConst0;
        self.fConst2 = 1.0 - self.fConst1;
        self.fConst3 = 1.0 / self.fConst0;
        self.fConst4 = (0) as F32;
    }
    pub fn instance_init(&mut self, sample_rate: i32) {
        self.instance_constants(sample_rate);
        self.instance_reset_params();
        self.instance_clear();
    }
    pub fn init(&mut self, sample_rate: i32) {
        Volume::class_init(sample_rate);
        self.instance_init(sample_rate);
    }
    pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
        Self::build_user_interface_static(ui_interface);
    }
    pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
        ui_interface.open_vertical_box("volume");
        ui_interface.declare(Some(ParamIndex(0)), "2", "");
        ui_interface.declare(Some(ParamIndex(0)), "style", "dB");
        ui_interface.declare(Some(ParamIndex(0)), "unit", "dB");
        ui_interface.add_vertical_bargraph("level", ParamIndex(0), -6e+01, 5.0);
        ui_interface.add_vertical_slider("volume", ParamIndex(1), 0.0, -7e+01, 4.0, 0.1);
        ui_interface.close_box();
    }
    pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
        match param.0 {
            0 => Some(self.fVbargraph0),
            1 => Some(self.fVslider0),
            _ => None,
        }
    }
    pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
        match param.0 {
            0 => self.fVbargraph0 = value,
            1 => self.fVslider0 = value,
            _ => {}
        }
    }
    pub fn compute(
        &mut self,
        count: usize,
        inputs: &[impl AsRef<[FaustFloat]>],
        outputs: &mut [impl AsMut<[FaustFloat]>],
    ) {
        let [inputs0, inputs1, ..] = inputs.as_ref() else {
            panic!("wrong number of input buffers");
        };
        let inputs0 = inputs0.as_ref()[..count].iter();
        let inputs1 = inputs1.as_ref()[..count].iter();
        let [outputs0, outputs1, ..] = outputs.as_mut() else {
            panic!("wrong number of output buffers");
        };
        let outputs0 = outputs0.as_mut()[..count].iter_mut();
        let outputs1 = outputs1.as_mut()[..count].iter_mut();
        let mut fSlow0: F32 = self.fConst1 * F32::powf(1e+01, 0.05 * self.fVslider0);
        let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
        for (((input0, input1), output0), output1) in zipped_iterators {
            self.fRec0[0] = fSlow0 + self.fConst2 * self.fRec0[1];
            let mut fTemp0: F32 = *input0;
            let mut fTemp1: F32 = *input1;
            self.fRec1[0] = F32::max(
                self.fRec1[1] - self.fConst3,
                F32::abs(0.5 * self.fRec0[0] * (fTemp0 + fTemp1)),
            );
            self.fVbargraph0 = 2e+01
                * F32::log10(
                    F32::max(1.1754944e-38, F32::max(0.00031622776, self.fRec1[0])),
                );
            *output0 = self.fConst4 + fTemp0 * self.fRec0[0];
            *output1 = fTemp1 * self.fRec0[0];
            self.fRec0[1] = self.fRec0[0];
            self.fRec1[1] = self.fRec1[0];
        }
    }
}
impl FaustDsp for Volume {
    type T = FaustFloat;
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }
    fn metadata(&self, m: &mut dyn Meta) {
        self.metadata(m)
    }
    fn get_sample_rate(&self) -> i32 {
        self.get_sample_rate()
    }
    fn get_num_inputs(&self) -> i32 {
        FAUST_INPUTS as i32
    }
    fn get_num_outputs(&self) -> i32 {
        FAUST_OUTPUTS as i32
    }
    fn class_init(sample_rate: i32)
    where
        Self: Sized,
    {
        Self::class_init(sample_rate);
    }
    fn instance_reset_params(&mut self) {
        self.instance_reset_params()
    }
    fn instance_clear(&mut self) {
        self.instance_clear()
    }
    fn instance_constants(&mut self, sample_rate: i32) {
        self.instance_constants(sample_rate)
    }
    fn instance_init(&mut self, sample_rate: i32) {
        self.instance_init(sample_rate)
    }
    fn init(&mut self, sample_rate: i32) {
        self.init(sample_rate)
    }
    fn build_user_interface(&self, ui_interface: &mut dyn UI<Self::T>) {
        self.build_user_interface(ui_interface)
    }
    fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>)
    where
        Self: Sized,
    {
        Self::build_user_interface_static(ui_interface);
    }
    fn get_param(&self, param: ParamIndex) -> Option<Self::T> {
        self.get_param(param)
    }
    fn set_param(&mut self, param: ParamIndex, value: Self::T) {
        self.set_param(param, value)
    }
    fn compute(
        &mut self,
        count: i32,
        inputs: &[&[Self::T]],
        outputs: &mut [&mut [Self::T]],
    ) {
        self.compute(count as usize, inputs, outputs)
    }
}
use strum::{
    Display, EnumIter, EnumCount, EnumDiscriminants, IntoStaticStr, VariantArray,
    VariantNames,
};
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Display,
    EnumIter,
    EnumCount,
    EnumDiscriminants,
    VariantNames
)]
#[strum_discriminants(
    derive(Display, EnumIter, EnumCount, IntoStaticStr, VariantArray, VariantNames, Hash)
)]
#[strum_discriminants(name(UIActive))]
pub enum UIActiveValue {
    Volume(FaustFloat),
}
impl UISelfSet<Volume, FaustFloat> for UIActiveValue {
    fn set(&self, dsp: &mut Volume) {
        match self {
            UIActiveValue::Volume(value) => dsp.fVslider0 = *value,
        }
    }
    fn get(&self) -> FaustFloat {
        match self {
            UIActiveValue::Volume(value) => *value,
        }
    }
}
impl UISet<Volume, FaustFloat> for UIActive {
    fn set(&self, dsp: &mut Volume, value: FaustFloat) {
        match self {
            UIActive::Volume => dsp.fVslider0 = value,
        }
    }
}
impl UIActive {
    pub fn value(&self, value: FaustFloat) -> UIActiveValue {
        match self {
            UIActive::Volume => UIActiveValue::Volume(value),
        }
    }
}
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Display,
    EnumIter,
    EnumCount,
    EnumDiscriminants,
    VariantNames
)]
#[strum_discriminants(
    derive(Display, EnumIter, EnumCount, IntoStaticStr, VariantArray, VariantNames, Hash)
)]
#[strum_discriminants(name(UIPassive))]
pub enum UIPassiveValue {
    Level(FaustFloat),
}
impl UIGet<Volume> for UIPassive {
    type E = UIPassiveValue;
    type F = FaustFloat;
    fn get_value(&self, dsp: &Volume) -> Self::F {
        match self {
            UIPassive::Level => dsp.fVbargraph0,
        }
    }
    fn get_enum(&self, dsp: &Volume) -> Self::E {
        match self {
            UIPassive::Level => UIPassiveValue::Level(dsp.fVbargraph0),
        }
    }
}
impl UIPassive {
    pub fn value(&self, value: FaustFloat) -> UIPassiveValue {
        match self {
            UIPassive::Level => UIPassiveValue::Level(value),
        }
    }
}
#[derive(Debug)]
pub struct DspUiVolume {
    pub level: UIPassive,
    pub volume: UIActive,
}
impl DspUiVolume {
    const fn static_ui() -> Self {
        Self {
            level: UIPassive::Level,
            volume: UIActive::Volume,
        }
    }
}
pub static DSP_UI: DspUiVolume = DspUiVolume::static_ui();
