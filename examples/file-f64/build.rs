use faust_build::{
    builder::{Architecture, ArchitectureUI, FaustBuilder},
    code_option::CodeOption,
};

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let mut b = FaustBuilder::default();
    b.set_dsp_path("dsp/volume.dsp");
    b.set_out_path("src/dsp.rs");
    b.struct_name_from_dsp_name();
    b.module_name_from_dsp_file_path();
    b.set_json();
    b.set_xml();
    b.set_code_option(CodeOption::Double);
    b.set_architecture(Architecture::Object(Box::new(ArchitectureUI {})));
    b.build();
}
