use dsp::{Amplifer, UIActive, UIActiveValue, UIPassive, DSP_UI};
use faust_types::{UIGet, UISelfSet, UISet};
pub mod dsp;

fn main() {
    let mut dsp = Amplifer::new();
    dsp.init(44_100);
    UIActive::Channel0Volume.set(&mut dsp, 1.0f64);
    UIActiveValue::Channel1Volume(10.0f64).set(&mut dsp);
    let ib = [[1.0f64], [10.0f64]];
    let mut ob = [[0f64], [0f64]];
    dsp.compute(1, &ib, &mut ob);
    //two ways to access values returned from the dsp:
    println!("{:?}", UIPassive::Channel0Level.get_enum(&dsp));
    println!("{}", DSP_UI.channel_1.level.get_value(&dsp));
}
