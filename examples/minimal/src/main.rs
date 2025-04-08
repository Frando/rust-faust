use crate::dsp::Volumecontrol;
use faust_types::ParamIndex;
pub mod dsp;

fn main() {
    let mut dsp = Volumecontrol::new();
    dsp.init(44_100);
    dsp.set_param(ParamIndex(1), 10.0_f32);
    let ib = [[1.0f32], [1.0f32]];
    let mut ob = [[0f32], [0f32]];
    dsp.compute(1, &ib, &mut ob);
    println!("messured volume:{}", dsp.get_param(ParamIndex(0)).unwrap());
}
