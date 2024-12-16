use faust_build::FaustBuilder;

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let b = FaustBuilder::new("dsp/volume.dsp", "src/dsp.rs").set_use_double(true);
    b.build();
    b.build_xml();
}
