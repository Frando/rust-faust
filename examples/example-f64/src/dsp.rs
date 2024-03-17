mod dsp {
    /* ------------------------------------------------------------
author: "Franz Heinzmann"
license: "BSD"
name: "volumecontrol"
version: "1.0"
Code generated with Faust 2.70.3 (https://faust.grame.fr)
Compilation options: -a /tmp/.tmp7BJYzb -lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -double -ftz 0
------------------------------------------------------------ */
#![allow(clippy::all)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]

use faust_types::*;



#[cfg_attr(feature = "default-boxed", derive(default_boxed::DefaultBoxed))]
#[repr(C)]
#[derive(Debug,Clone)]
pub struct mydsp {
	fSampleRate: i32,
	fConst1: F64,
	fConst2: F64,
	fVslider0: F64,
	fRec0: [F64;2],
	fConst3: F64,
	fRec1: [F64;2],
	fVbargraph0: F64,
	fConst4: F64,
}

impl FaustDsp for mydsp {
	type T = F64;
		
	fn new() -> mydsp { 
		mydsp {
			fSampleRate: 0,
			fConst1: 0.0,
			fConst2: 0.0,
			fVslider0: 0.0,
			fRec0: [0.0;2],
			fConst3: 0.0,
			fRec1: [0.0;2],
			fVbargraph0: 0.0,
			fConst4: 0.0,
		}
	}
	fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("author", r"Franz Heinzmann");
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/tabulateNd", r"Copyright (C) 2023 Bart Brouns <bart@magnetophon.nl>");
		m.declare("basics.lib/version", r"1.12.0");
		m.declare("compile_options", r"-a /tmp/.tmp7BJYzb -lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -double -ftz 0");
		m.declare("filename", r"volume.dsp");
		m.declare("license", r"BSD");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.7.0");
		m.declare("name", r"volumecontrol");
		m.declare("options", r"[osc:on]");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
		m.declare("signals.lib/name", r"Faust Signal Routing Library");
		m.declare("signals.lib/version", r"1.5.0");
		m.declare("version", r"1.0");
	}

	fn get_sample_rate(&self) -> i32 {
		return self.fSampleRate;
	}
	fn get_num_inputs(&self) -> i32 {
		return 2;
	}
	fn get_num_outputs(&self) -> i32 {
		return 2;
	}
	
	fn class_init(sample_rate: i32) {
	}
	fn instance_reset_params(&mut self) {
		self.fVslider0 = 0.0;
	}
	fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fRec0[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec1[l1 as usize] = 0.0;
		}
	}
	fn instance_constants(&mut self, sample_rate: i32) {
		self.fSampleRate = sample_rate;
		let mut fConst0: F64 = F64::min(1.92e+05, F64::max(1.0, (self.fSampleRate) as F64));
		self.fConst1 = 44.1 / fConst0;
		self.fConst2 = 1.0 - self.fConst1;
		self.fConst3 = 1.0 / fConst0;
		self.fConst4 = (0) as F64;
	}
	fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	fn init(&mut self, sample_rate: i32) {
		mydsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	fn build_user_interface(&self, ui_interface: &mut dyn UI<Self::T>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>) {
		ui_interface.open_vertical_box("volumecontrol");
		ui_interface.declare(Some(ParamIndex(0)), "2", "");
		ui_interface.declare(Some(ParamIndex(0)), "style", "dB");
		ui_interface.declare(Some(ParamIndex(0)), "unit", "dB");
		ui_interface.add_vertical_bargraph("level", ParamIndex(0), -6e+01, 5.0);
		ui_interface.add_vertical_slider("volume", ParamIndex(1), 0.0, -7e+01, 4.0, 0.1);
		ui_interface.close_box();
	}
	
	fn get_param(&self, param: ParamIndex) -> Option<Self::T> {
		match param.0 {
			0 => Some(self.fVbargraph0),
			1 => Some(self.fVslider0),
			_ => None,
		}
	}
	
	fn set_param(&mut self, param: ParamIndex, value: Self::T) {
		match param.0 {
			0 => { self.fVbargraph0 = value }
			1 => { self.fVslider0 = value }
			_ => {}
		}
	}
	
	fn compute(&mut self, count: i32, inputs: &[&[Self::T]], outputs: &mut[&mut[Self::T]]) {
		let (inputs0, inputs1) = if let [inputs0, inputs1, ..] = inputs {
			let inputs0 = inputs0[..count as usize].iter();
			let inputs1 = inputs1[..count as usize].iter();
			(inputs0, inputs1)
		} else {
			panic!("wrong number of inputs");
		};
		let (outputs0, outputs1) = if let [outputs0, outputs1, ..] = outputs {
			let outputs0 = outputs0[..count as usize].iter_mut();
			let outputs1 = outputs1[..count as usize].iter_mut();
			(outputs0, outputs1)
		} else {
			panic!("wrong number of outputs");
		};
		let mut fSlow0: F64 = self.fConst1 * F64::powf(1e+01, 0.05 * self.fVslider0);
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			self.fRec0[0] = fSlow0 + self.fConst2 * self.fRec0[1];
			let mut fTemp0: F64 = *input0;
			let mut fTemp1: F64 = *input1;
			self.fRec1[0] = F64::max(self.fRec1[1] - self.fConst3, F64::abs(0.5 * self.fRec0[0] * (fTemp0 + fTemp1)));
			self.fVbargraph0 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, F64::max(0.00031622776601683794, self.fRec1[0])));
			*output0 = self.fConst4 + fTemp0 * self.fRec0[0];
			*output1 = fTemp1 * self.fRec0[0];
			self.fRec0[1] = self.fRec0[0];
			self.fRec1[1] = self.fRec1[0];
		}
	}

}


}
pub use dsp::mydsp as Volume;
