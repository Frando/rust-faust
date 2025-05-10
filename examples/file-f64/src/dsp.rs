#![allow(clippy::all)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]
use faust_types::*;
pub type FaustFloat = F64;
mod ffi {
    use std::os::raw::c_double;
    #[cfg_attr(not(target_os = "windows"), link(name = "m"))]
    extern {
        pub fn remainder(from: c_double, to: c_double) -> c_double;
        pub fn rint(val: c_double) -> c_double;
    }
}
fn remainder_f64(from: f64, to: f64) -> f64 {
    unsafe { ffi::remainder(from, to) }
}
fn rint_f64(val: f64) -> f64 {
    unsafe { ffi::rint(val) }
}
pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 2;
pub const FAUST_PASSIVES: usize = 2;
#[cfg_attr(feature = "default-boxed", derive(default_boxed::DefaultBoxed))]
#[repr(C)]
pub struct Amplifer {
    fSampleRate: i32,
    fConst0: F64,
    fConst1: F64,
    fConst2: F64,
    fConst3: F64,
    fVslider0: F64,
    fRec1: [F64; 2],
    fRec0: [F64; 2],
    fVbargraph0: F64,
    iConst4: i32,
    fVslider1: F64,
    fRec3: [F64; 2],
    fRec2: [F64; 2],
    fVbargraph1: F64,
    iConst5: i32,
}
impl Amplifer {
    pub fn new() -> Amplifer {
        Amplifer {
            fSampleRate: 0,
            fConst0: 0.0,
            fConst1: 0.0,
            fConst2: 0.0,
            fConst3: 0.0,
            fVslider0: 0.0,
            fRec1: [0.0; 2],
            fRec0: [0.0; 2],
            fVbargraph0: 0.0,
            iConst4: 0,
            fVslider1: 0.0,
            fRec3: [0.0; 2],
            fRec2: [0.0; 2],
            fVbargraph1: 0.0,
            iConst5: 0,
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
            r"-lang rust -ct 1 -cn Amplifer -es 1 -mcd 16 -mdd 1024 -mdy 33 -double -ftz 0",
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
        self.fVslider1 = 0.0;
    }
    pub fn instance_clear(&mut self) {
        for l0 in 0..2 {
            self.fRec1[l0 as usize] = 0.0;
        }
        for l1 in 0..2 {
            self.fRec0[l1 as usize] = 0.0;
        }
        for l2 in 0..2 {
            self.fRec3[l2 as usize] = 0.0;
        }
        for l3 in 0..2 {
            self.fRec2[l3 as usize] = 0.0;
        }
    }
    pub fn instance_constants(&mut self, sample_rate: i32) {
        self.fSampleRate = sample_rate;
        self.fConst0 = F64::min(1.92e+05, F64::max(1.0, (self.fSampleRate) as F64));
        self.fConst1 = 1.0 / self.fConst0;
        self.fConst2 = 44.1 / self.fConst0;
        self.fConst3 = 1.0 - self.fConst2;
        self.iConst4 = 0;
        self.iConst5 = 0;
    }
    pub fn instance_init(&mut self, sample_rate: i32) {
        self.instance_constants(sample_rate);
        self.instance_reset_params();
        self.instance_clear();
    }
    pub fn init(&mut self, sample_rate: i32) {
        Amplifer::class_init(sample_rate);
        self.instance_init(sample_rate);
    }
    pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
        Self::build_user_interface_static(ui_interface);
    }
    pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
        ui_interface.open_vertical_box("volume");
        ui_interface.open_vertical_box("channel_0");
        ui_interface.declare(Some(ParamIndex(0)), "2", "");
        ui_interface.declare(Some(ParamIndex(0)), "style", "dB");
        ui_interface.declare(Some(ParamIndex(0)), "unit", "dB");
        ui_interface.add_vertical_bargraph("level", ParamIndex(0), -6e+01, 5.0);
        ui_interface.add_vertical_slider("volume", ParamIndex(1), 0.0, -7e+01, 4.0, 0.1);
        ui_interface.close_box();
        ui_interface.open_vertical_box("channel_1");
        ui_interface.declare(Some(ParamIndex(2)), "2", "");
        ui_interface.declare(Some(ParamIndex(2)), "style", "dB");
        ui_interface.declare(Some(ParamIndex(2)), "unit", "dB");
        ui_interface.add_vertical_bargraph("level", ParamIndex(2), -6e+01, 5.0);
        ui_interface.add_vertical_slider("volume", ParamIndex(3), 0.0, -7e+01, 4.0, 0.1);
        ui_interface.close_box();
        ui_interface.close_box();
    }
    pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
        match param.0 {
            0 => Some(self.fVbargraph0),
            2 => Some(self.fVbargraph1),
            1 => Some(self.fVslider0),
            3 => Some(self.fVslider1),
            _ => None,
        }
    }
    pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
        match param.0 {
            0 => self.fVbargraph0 = value,
            2 => self.fVbargraph1 = value,
            1 => self.fVslider0 = value,
            3 => self.fVslider1 = value,
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
        let mut fSlow0: F64 = self.fConst2 * F64::powf(1e+01, 0.05 * self.fVslider0);
        let mut fSlow1: F64 = self.fConst2 * F64::powf(1e+01, 0.05 * self.fVslider1);
        let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
        for (((input0, input1), output0), output1) in zipped_iterators {
            self.fRec1[0] = fSlow0 + self.fConst3 * self.fRec1[1];
            self.fRec0[0] = F64::max(
                self.fRec0[1] - self.fConst1,
                F64::abs(*input0 * self.fRec1[0]),
            );
            self.fVbargraph0 = 2e+01
                * F64::log10(
                    F64::max(
                        2.2250738585072014e-308,
                        F64::max(0.00031622776601683794, self.fRec0[0]),
                    ),
                );
            *output0 = (self.iConst4) as F64;
            self.fRec3[0] = fSlow1 + self.fConst3 * self.fRec3[1];
            self.fRec2[0] = F64::max(
                self.fRec2[1] - self.fConst1,
                F64::abs(*input1 * self.fRec3[0]),
            );
            self.fVbargraph1 = 2e+01
                * F64::log10(
                    F64::max(
                        2.2250738585072014e-308,
                        F64::max(0.00031622776601683794, self.fRec2[0]),
                    ),
                );
            *output1 = (self.iConst5) as F64;
            self.fRec1[1] = self.fRec1[0];
            self.fRec0[1] = self.fRec0[0];
            self.fRec3[1] = self.fRec3[0];
            self.fRec2[1] = self.fRec2[0];
        }
    }
}
impl FaustDsp for Amplifer {
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
    Channel0Volume(FaustFloat),
    Channel1Volume(FaustFloat),
}
impl UISelfSet<Amplifer, FaustFloat> for UIActiveValue {
    fn set(&self, dsp: &mut Amplifer) {
        match self {
            UIActiveValue::Channel0Volume(value) => dsp.fVslider0 = *value,
            UIActiveValue::Channel1Volume(value) => dsp.fVslider1 = *value,
        }
    }
    fn get(&self) -> FaustFloat {
        match self {
            UIActiveValue::Channel0Volume(value) => *value,
            UIActiveValue::Channel1Volume(value) => *value,
        }
    }
}
impl UISet<Amplifer, FaustFloat> for UIActive {
    fn set(&self, dsp: &mut Amplifer, value: FaustFloat) {
        match self {
            UIActive::Channel0Volume => dsp.fVslider0 = value,
            UIActive::Channel1Volume => dsp.fVslider1 = value,
        }
    }
}
impl UIActive {
    pub fn value(&self, value: FaustFloat) -> UIActiveValue {
        match self {
            UIActive::Channel0Volume => UIActiveValue::Channel0Volume(value),
            UIActive::Channel1Volume => UIActiveValue::Channel1Volume(value),
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
    Channel0Level(FaustFloat),
    Channel1Level(FaustFloat),
}
impl UIGet<Amplifer> for UIPassive {
    type E = UIPassiveValue;
    type F = FaustFloat;
    fn get_value(&self, dsp: &Amplifer) -> Self::F {
        match self {
            UIPassive::Channel0Level => dsp.fVbargraph0,
            UIPassive::Channel1Level => dsp.fVbargraph1,
        }
    }
    fn get_enum(&self, dsp: &Amplifer) -> Self::E {
        match self {
            UIPassive::Channel0Level => UIPassiveValue::Channel0Level(dsp.fVbargraph0),
            UIPassive::Channel1Level => UIPassiveValue::Channel1Level(dsp.fVbargraph1),
        }
    }
}
impl UIPassive {
    pub fn value(&self, value: FaustFloat) -> UIPassiveValue {
        match self {
            UIPassive::Channel0Level => UIPassiveValue::Channel0Level(value),
            UIPassive::Channel1Level => UIPassiveValue::Channel1Level(value),
        }
    }
}
#[derive(Debug)]
pub struct DspUiVolume {
    pub channel_0: DspUiVolumeChannel0,
    pub channel_1: DspUiVolumeChannel1,
}
impl DspUiVolume {
    const fn static_ui() -> Self {
        Self {
            channel_0: DspUiVolumeChannel0::static_ui(),
            channel_1: DspUiVolumeChannel1::static_ui(),
        }
    }
}
#[derive(Debug)]
pub struct DspUiVolumeChannel0 {
    pub level: UIPassive,
    pub volume: UIActive,
}
impl DspUiVolumeChannel0 {
    const fn static_ui() -> Self {
        Self {
            level: UIPassive::Channel0Level,
            volume: UIActive::Channel0Volume,
        }
    }
}
#[derive(Debug)]
pub struct DspUiVolumeChannel1 {
    pub level: UIPassive,
    pub volume: UIActive,
}
impl DspUiVolumeChannel1 {
    const fn static_ui() -> Self {
        Self {
            level: UIPassive::Channel1Level,
            volume: UIActive::Channel1Volume,
        }
    }
}
pub static DSP_UI: DspUiVolume = DspUiVolume::static_ui();
pub mod meta {
    pub const AUTHOR: &'static str = "Franz Heinzmann";
    pub const COMPILE_OPTIONS: &'static str = "-lang rust -ct 1 -cn Amplifer -es 1 -mcd 16 -mdd 1024 -mdy 33 -double -ftz 0";
    pub const FILENAME: &'static str = "volume.dsp";
    pub const LICENSE: &'static str = "BSD";
    pub const NAME: &'static str = "volume";
    pub const OPTIONS: &'static str = "[osc:on]";
    pub const VERSION: &'static str = "1.0";
    pub mod libs {
        pub mod basics {
            pub const NAME: &'static str = "Faust Basic Element Library";
            pub const TABULATEND: &'static str = "Copyright (C) 2023 Bart Brouns <bart@magnetophon.nl>";
            pub const VERSION: &'static str = "1.21.0";
        }
        pub mod maths {
            pub const AUTHOR: &'static str = "GRAME";
            pub const COPYRIGHT: &'static str = "GRAME";
            pub const LICENSE: &'static str = "LGPL with exception";
            pub const NAME: &'static str = "Faust Math Library";
            pub const VERSION: &'static str = "2.8.1";
        }
        pub mod platform {
            pub const NAME: &'static str = "Generic Platform Library";
            pub const VERSION: &'static str = "1.3.0";
        }
        pub mod signals {
            pub const NAME: &'static str = "Faust Signal Routing Library";
            pub const VERSION: &'static str = "1.6.0";
        }
    }
}
