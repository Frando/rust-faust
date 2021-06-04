use faust_build::build_dsp;

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    build_dsp("dsp/volume.dsp");
}
