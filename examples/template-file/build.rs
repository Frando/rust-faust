use faust_build::{architecture::Architecture, builder::FaustBuilder};
use std::{env, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let out_dir = env::var_os("OUT_DIR").expect("Environment Variable OUT_DIR is not defined");
    let dest_path = Path::new(&out_dir).join("dsp.rs");

    let mut b = FaustBuilder::default_for_file("dsp/volume.dsp", dest_path);
    b.set_architecture(Architecture::file(
        "../../faust-build/faust-template.rs".into(),
    ));
    b.struct_name_from_dsp_name();

    b.build();
}
