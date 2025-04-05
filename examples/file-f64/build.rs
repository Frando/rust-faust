use faust_build::{architecture::Architecture, builder::FaustBuilder, code_option::CodeOption};

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    //example of setting up compilation
    // without any conveniens functions
    let mut b = FaustBuilder::default();
    b.set_dsp_path("dsp/volume.dsp");
    b.set_out_path("src/dsp.rs");
    b.write_json_file();
    b.write_xml_file();
    b.set_code_option(CodeOption::StructName("Amplifer".to_owned()));
    b.set_code_option(CodeOption::Double);
    b.set_architecture(Architecture::ui());
    b.build();
}
