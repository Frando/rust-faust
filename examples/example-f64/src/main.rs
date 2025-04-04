use crate::dsp::Volume;
use faust_types::ParamIndex;
pub mod dsp;

fn main() {
    println!("Hello, world!");
    let mut dsp = Volume::new();
    dsp.init(44_100);
    dsp.set_param(ParamIndex(0), 10.0_f64);
}
