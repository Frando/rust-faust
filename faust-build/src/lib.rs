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

pub struct FaustBuilder {
    in_file: String,
    out_file: String,
    /// Module name the dsp code will be encapsulated in. By default is "dsp".
    module_name: String,
    /// Name for the DSP struct. If None, we use CamelCased file name.
    struct_name: Option<String>,
    use_double: bool,
    faust_args: Vec<String>,
}

impl Default for FaustBuilder {
    fn default() -> Self {
        Self {
            in_file: "".into(),
            out_file: "".into(),
            struct_name: None,
            module_name: "dsp".into(),
            use_double: false,
            faust_args: vec![],
        }
    }
}

impl FaustBuilder {
    pub fn new(in_file: &str, out_file: &str) -> Self {
        Self {
            in_file: in_file.to_string(),
            out_file: out_file.to_string(),
            ..Default::default()
        }
    }

    pub fn set_struct_name(mut self, struct_name: Option<String>) -> Self {
        self.struct_name = struct_name;
        self
    }
    pub fn set_module_name(mut self, module_name: String) -> Self {
        self.module_name = module_name;
        self
    }

    pub fn set_use_double(mut self, use_double: bool) -> Self {
        self.use_double = use_double;
        self
    }

    /// Add additionals args to the faust build command
    pub fn faust_arg(mut self, arg: String) -> Self {
        self.faust_args.push(arg);
        self
    }

    pub fn build(self) {
        let dsp_file = self.in_file;
        let dest_path = self.out_file;
        eprintln!("cargo:rerun-if-changed={}", dsp_file);

        let dest_path = PathBuf::from(dest_path);

        let template_code = include_str!("../faust-template.rs");
        let template_file = NamedTempFile::new().expect("failed creating temporary file");
        let target_file = NamedTempFile::new().expect("failed creating temporary file");

        fs::write(template_file.path(), template_code).expect("failed writing temporary file");

        // faust -a $ARCHFILE -lang rust "$SRCDIR/$f" -o "$SRCDIR/$dspName/src/main.rs"
        let mut output = Command::new("faust");

        let struct_name = match &self.struct_name {
            Some(struct_name) => struct_name.clone(),
            None => {
                let dsp_path = PathBuf::from(&dsp_file);
                dsp_path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                    .to_camel_case()
            }
        };

        output
            .arg("-a")
            .arg(template_file.path())
            .arg("-lang")
            .arg("rust")
            .arg("-t")
            .arg("0")
            .arg("-cn")
            .arg(&struct_name);

        if self.use_double {
            output.arg("-double");
        }

        for arg in self.faust_args {
            output.arg(arg);
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
        let dsp_code = dsp_code.replace("<<moduleName>>", &self.module_name);
        let dsp_code = dsp_code.replace("<<structName>>", &struct_name);

        fs::write(&dest_path, dsp_code).expect("failed to write to destination path");

        eprintln!("Wrote module:\n{}", dest_path.to_str().unwrap());
    }
}
