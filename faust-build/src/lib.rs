#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::or_fun_call)]

pub mod faust_arg;
pub mod faust_utils;

use faust_arg::{FaustArg, FaustArgsToCommandArgs};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    vec,
};
use tempfile::NamedTempFile;

pub fn build_dsp(dsp_file: &str) {
    let out_dir = env::var_os("OUT_DIR").expect("Environment Variable OUT_DIR is not defined");
    let dest_path = Path::new(&out_dir).join("dsp.rs");
    FaustBuilder::new(dsp_file, dest_path).build();
}

pub fn build_dsp_to_destination(dsp_file: &str, dest_path: &str) {
    FaustBuilder::new(dsp_file, dest_path).build();
}

pub struct FaustBuilder {
    faust_path: Option<PathBuf>,
    in_file: PathBuf,
    out_file: PathBuf,
    arch_file: Option<PathBuf>,
    /// Module name the dsp code will be encapsulated in. By default is "dsp".
    module_name: String,
    /// Name for the DSP struct. If None, we use camel cased file name.
    struct_name: Option<String>,
    use_double: bool,
    faust_args: Vec<FaustArg>,
}

impl Default for FaustBuilder {
    fn default() -> Self {
        Self {
            faust_path: None,
            in_file: "".into(),
            out_file: "".into(),
            arch_file: None,
            struct_name: None,
            module_name: "dsp".into(),
            use_double: false,
            faust_args: vec![],
        }
    }
}

impl FaustBuilder {
    pub fn new(in_file: impl Into<PathBuf>, out_file: impl Into<PathBuf>) -> Self {
        Self {
            in_file: in_file.into(),
            out_file: out_file.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn set_struct_name(mut self, struct_name: impl Into<String>) -> Self {
        self.struct_name = Some(struct_name.into());
        self
    }
    #[must_use]
    pub fn set_module_name(mut self, module_name: impl Into<String>) -> Self {
        self.module_name = module_name.into();
        self
    }

    #[must_use]
    pub fn set_use_double(mut self, use_double: bool) -> Self {
        self.use_double = use_double;
        self
    }

    #[must_use]
    pub fn set_faust_path(mut self, faust_path: impl Into<PathBuf>) -> Self {
        self.faust_path = Some(faust_path.into());
        self
    }
    #[must_use]
    pub fn set_arch_file(mut self, arch_file: impl Into<PathBuf>) -> Self {
        self.arch_file = Some(arch_file.into());
        self
    }

    /// Add additionals args to the faust build command
    #[must_use]
    pub fn faust_arg(mut self, arg: impl Into<FaustArg>) -> Self {
        self.faust_args.push(arg.into());
        self
    }

    #[must_use]
    pub fn get_module_name(&self) -> String {
        self.module_name.clone()
    }

    #[must_use]
    pub fn get_struct_name(&self) -> String {
        self.struct_name.as_ref().map_or_else(
            || faust_utils::struct_name_from_dsp_path(&self.in_file),
            |struct_name| (*struct_name).clone(),
        )
    }

    #[must_use]
    pub fn to_faust_args(&self, extra_flags: Vec<FaustArg>) -> Vec<FaustArg> {
        let mut args: Vec<FaustArg> = Vec::new();

        args.extend([
            FaustArg::default_lang(),
            FaustArg::default_timeout(),
            FaustArg::StructName(self.get_struct_name()),
        ]);

        if self.use_double {
            args.push(FaustArg::Double);
        }

        for arg in &self.faust_args {
            args.push(arg.clone());
        }

        args.push(FaustArg::DspPath(self.in_file.clone()));

        for arg in extra_flags {
            args.push(arg);
        }
        args
    }

    pub fn build(&self) {
        let target_file = NamedTempFile::new().expect("failed creating temporary file");

        let extra_flags = vec![FaustArg::OutPath(target_file.path().to_path_buf())];
        let mut args = self.to_faust_args(extra_flags);

        // keep this block until we remove use of architecture files in the examples
        let default_arch = NamedTempFile::new().expect("failed creating temporary file");
        let template_code = include_str!("../faust-template.rs");
        fs::write(default_arch.path(), template_code).expect("failed writing temporary file");
        let default_template = &default_arch.path().into();
        let template_file = self
            .arch_file
            .as_ref()
            .map_or(default_template, |arch_file| arch_file);
        args.push(FaustArg::ArchFile(template_file.clone()));

        let faust_path = self.faust_path.clone().unwrap_or("faust".into());
        let faust_output = Command::new(faust_path)
            .args(args.to_command_args())
            .output()
            .expect("Failed to execute command");
        assert!(
            faust_output.status.success(),
            "faust compilation failed: {}",
            String::from_utf8(faust_output.stderr).unwrap()
        );

        let dsp_code = fs::read(target_file).unwrap();
        let dsp_code = String::from_utf8(dsp_code).unwrap();
        let dsp_code = dsp_code.replace("<<moduleName>>", &self.module_name);
        let dsp_code = dsp_code.replace("<<structName>>", &self.get_struct_name());

        let dest_path = &self.out_file;
        fs::write(dest_path, dsp_code).expect("failed to write to destination path");

        eprintln!("Wrote module:\n{}", dest_path.to_str().unwrap());
    }

    #[must_use]
    pub fn build_to_stdout(&self, extra_flags: Vec<FaustArg>) -> String {
        let args = self.to_faust_args(extra_flags);
        let faust_path = self.faust_path.clone().unwrap_or("faust".into());

        let faust_output = Command::new(faust_path)
            .args(args.to_command_args())
            .output()
            .expect("Failed to execute command");
        assert!(
            faust_output.status.success(),
            "faust compilation failed: {}",
            String::from_utf8(faust_output.stderr).unwrap()
        );
        String::from_utf8(faust_output.stdout).expect("could not parse stdout from command")
    }

    pub fn build_xml(&self) {
        let _ = self.build_to_stdout(vec![FaustArg::Xml()]);
    }

    pub fn build_xml_at_file(&self, out: &str) {
        let gen_xml_path: PathBuf = FaustArg::DspPath(self.in_file.clone()).xml_path();
        self.build_xml();
        fs::rename(&gen_xml_path, out).unwrap_or_else(|_| {
            panic!(
                "rename of xml file failed from '{:?}' to '{:?}'",
                gen_xml_path, out
            )
        });
    }

    pub fn build_json(&self) {
        let _ = self.build_to_stdout(vec![FaustArg::Json()]);
    }

    pub fn build_json_at_file(&self, out: &str) {
        let gen_json_path = faust_utils::json_path_from_dsp_path(&self.in_file);
        self.build_json();
        fs::rename(&gen_json_path, out).unwrap_or_else(|_| {
            panic!(
                "rename of json file failed from '{:?}' to '{:?}'",
                gen_json_path, out
            )
        });
    }
}
