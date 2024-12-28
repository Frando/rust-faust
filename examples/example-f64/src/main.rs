use dsp::{UIActive, UIActiveValue, UIPassive, Volume, UI};
use faust_types::{UIGet, UISelfSet, UISet};
pub mod dsp;

fn main() {
    let mut dsp = Volume::new();
    dsp.init(44_100);
    UIActive::Channel0Volume.set(&mut dsp, 1.0f64);
    UIActiveValue::Channel1Volume(10.0f64).set(&mut dsp);
    let ib = [[1.0f64], [10.0f64]];
    let mut ob = [[0f64], [0f64]];
    dsp.compute(1, &ib, &mut ob);
    println!("{:?}", UIPassive::Channel0Level.get_enum(&dsp));
    println!("{}", UI.volume.channel_1.level.get_value(&dsp));
}
