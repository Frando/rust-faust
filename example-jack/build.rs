use faust_build::build_dsp;

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    // build_dsp("dsp/mixer.dsp");
    // build_dsp("dsp/volume.dsp");
    build_dsp("dsp/volume.dsp");
    // build_dsp("dsp/passthrough.dsp");
}
