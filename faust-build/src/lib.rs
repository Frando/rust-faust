use heck::CamelCase;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{env, path::PathBuf};
use tempfile::NamedTempFile;

pub fn build_dsp(dsp_file: &str) {
    eprintln!("cargo:rerun-if-changed={}", dsp_file);
    let dsp_path = PathBuf::from(dsp_file);
    let dsp_name = dsp_path.file_stem().unwrap();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("dsp.rs");

    let template_code = include_str!("../faust-template.rs");
    let template_file = NamedTempFile::new().expect("failed creating temporary file");
    let target_file = NamedTempFile::new().expect("failed creating temporary file");

    fs::write(template_file.path(), template_code).expect("failed writing temporary file");

    // faust -a $ARCHFILE -lang rust "$SRCDIR/$f" -o "$SRCDIR/$dspName/src/main.rs"
    let output = Command::new("faust")
        .arg("-a")
        .arg(template_file.path())
        .arg("-lang")
        .arg("rust")
        .arg(&dsp_file)
        .arg("-o")
        .arg(target_file.path())
        .output()
        .expect("Failed to execute command");
    // eprintln!(
    //     "Wrote temp module:\n{}",
    //     target_file.path().to_str().unwrap()
    // );
    if !output.status.success() {
        panic!(
            "faust compilation failed: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }

    let dsp_code = fs::read(target_file).unwrap();
    let dsp_code = String::from_utf8(dsp_code).unwrap();

    let dsp_code = dsp_code.replace(
        "pub struct mydsp",
        "#[derive(Debug,Clone)]\npub struct mydsp",
    );

    let struct_name = dsp_name.to_str().unwrap().to_camel_case();

    let module_code = format!(
        r#"mod dsp {{
    {}
}}
pub use dsp::mydsp as {};
pub use dsp::enable_flush_denormals_to_zero; 
"#,
        dsp_code, struct_name
    );

    fs::write(&dest_path, module_code).expect("failed to write to destination path");

    // TODO: rustfmt hangs on the created file.
    // Command::new("rustfmt")
    //     .arg(&dest_path)
    //     .output()
    //     .expect("failed to run rustfmt");

    eprintln!("Wrote module:\n{}", dest_path.to_str().unwrap());
}
