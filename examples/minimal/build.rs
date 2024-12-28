use faust_build::{architecture::Architecture, builder::FaustBuilder};

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let mut b = FaustBuilder::default_for_file("dsp/volume.dsp", "src/dsp.rs");
    b.set_architecture(Architecture::default());
    b.build();
}
