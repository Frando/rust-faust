use faust_build::{
    builder::{ArchitectureUI, FaustBuilder},
    code_option::CodeOption,
    compile_option::CompileOption,
};

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let mut b = FaustBuilder::default();
    b.set_dsp_path("dsp/volume.dsp");
    b.set_out_path("src/dsp.rs");
    b.struct_name_from_dsp_name();
    b.module_name_from_dsp_file_path();
    b.set_compile_option(CompileOption::Json);
    b.set_architecture(Box::new(ArchitectureUI {}));
    b.set_compile_option(CompileOption::Xml);
    b.set_code_option(CodeOption::Double);
    b.build();
}
