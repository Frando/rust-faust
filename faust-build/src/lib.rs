use heck::CamelCase;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{env, path::PathBuf};
use tempfile::NamedTempFile;

pub fn build_dsp(dsp_file: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("dsp.rs");
    FaustBuilder::new(dsp_file, dest_path.to_str().unwrap()).build()
}

pub fn build_dsp_to_destination(dsp_file: &str, dest_path: &str) {
    FaustBuilder::new(dsp_file, dest_path).build()
}

#[derive(Default)]
pub struct FaustBuilder {
    in_file: String,
    out_file: String,
    use_double: bool,
}

impl FaustBuilder {
    pub fn new(in_file: &str, out_file: &str) -> Self {
        Self {
            in_file: in_file.to_string(),
            out_file: out_file.to_string(),
            ..Default::default()
        }
    }

    pub fn set_use_double(mut self, use_double: bool) -> Self {
        self.use_double = use_double;
        self
    }

    pub fn build(self) {
        let dsp_file = self.in_file;
        let dest_path = self.out_file;
        eprintln!("cargo:rerun-if-changed={}", dsp_file);
        let dsp_path = PathBuf::from(&dsp_file);
        let dsp_name = dsp_path.file_stem().unwrap();

        let dest_path = PathBuf::from(dest_path);

        let template_code = include_str!("../faust-template.rs");
        let template_file = NamedTempFile::new().expect("failed creating temporary file");
        let target_file = NamedTempFile::new().expect("failed creating temporary file");

        fs::write(template_file.path(), template_code).expect("failed writing temporary file");

        // faust -a $ARCHFILE -lang rust "$SRCDIR/$f" -o "$SRCDIR/$dspName/src/main.rs"
        let mut output = Command::new("faust");

        output
            .arg("-a")
            .arg(template_file.path())
            .arg("-lang")
            .arg("rust");

        if self.use_double {
            output.arg("-double");
        }
        output.arg(&dsp_file).arg("-o").arg(target_file.path());

        let output = output.output().expect("Failed to execute command");
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
}
