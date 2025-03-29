use faust_build::FaustBuilder;

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let b = FaustBuilder::new("dsp/volume.dsp")
        .set_use_double(true)
        .build_to_out_file("src/dsp.rs")
        .build_xml();
}
