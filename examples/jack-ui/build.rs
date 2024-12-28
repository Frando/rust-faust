use faust_build::builder::FaustBuilder;

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let b = FaustBuilder::default_for_file_with_ui("dsp/volume.dsp", "src/dsp.rs");
    b.build();
}
