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

#[cfg(feature = "codegen")]
pub mod codegen;

use faust_arg::{FaustArg, FaustArgsToCommandArgs};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::NamedTempFile;

pub fn build_dsp(dsp_file: &str) {
    let out_dir = env::var_os("OUT_DIR").expect("Environment Variable OUT_DIR is not defined");
    let dest_path = Path::new(&out_dir).join("dsp.rs");
    FaustBuilder::new(dsp_file).build(dest_path);
}

pub fn build_dsp_to_destination(dsp_file: &str, dest_path: &str) {
    FaustBuilder::new(dsp_file).build(dest_path);
}

pub struct FaustBuilder {
    faust_path: Option<PathBuf>,
    in_file: PathBuf,
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
            arch_file: None,
            struct_name: None,
            module_name: "dsp".into(),
            use_double: false,
            faust_args: vec![],
        }
    }
}

impl FaustBuilder {
    pub fn new(in_file: impl Into<PathBuf>) -> Self {
        Self {
            in_file: in_file.into(),
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
    #[deprecated(note = "please use `arg` instead")]
    pub fn faust_arg(self, arg: impl Into<FaustArg>) -> Self {
        self.arg(arg)
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<FaustArg>) -> Self {
        self.faust_args.push(arg.into());
        self
    }

    #[must_use]
    pub fn args<T: Into<FaustArg> + Clone>(mut self, args: impl AsRef<[T]>) -> Self {
        self.faust_args
            .extend(args.as_ref().iter().map(|a| a.to_owned().into()));
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

    pub fn faust_command_path(&self) -> PathBuf {
        self.faust_path.clone().unwrap_or("faust".into())
    }

    pub fn to_command(&self, extra_flags: Vec<FaustArg>) -> Command {
        let args = self.to_faust_args(extra_flags);

        let faust_path = self.faust_command_path();

        let mut command = Command::new(faust_path);
        command.args(args.to_command_args());
        command
    }

    pub fn build(&self, out_file: impl Into<PathBuf>) {
        let out_file: PathBuf = out_file.into();
        let intermediate_out_file = NamedTempFile::new().expect("failed creating temporary file");

        let (architecture_file_path, _temp_file) = match self.arch_file.to_owned() {
            Some(arch_file) => (arch_file, None),
            None => {
                let default_arch_tempfile =
                    NamedTempFile::new().expect("failed creating temporary file");
                let default_template_code = include_str!("../faust-template.rs");
                fs::write(default_arch_tempfile.path(), default_template_code)
                    .expect("failed writing temporary file");
                (
                    default_arch_tempfile.path().into(),
                    Some(default_arch_tempfile),
                )
            }
        };

        let extra_flags = vec![
            FaustArg::OutPath(intermediate_out_file.path().to_path_buf()),
            FaustArg::ArchFile(architecture_file_path.clone()),
        ];

        let faust_output = self
            .to_command(extra_flags)
            .output()
            .expect("Failed to execute command");
        assert!(
            faust_output.status.success(),
            "faust compilation failed: {}",
            String::from_utf8(faust_output.stderr).unwrap()
        );

        let dsp_code = fs::read(intermediate_out_file).expect("Failed reading target file");
        let dsp_code = String::from_utf8(dsp_code).expect("Failed reading target file as utf8");
        let dsp_code = dsp_code.replace("<<moduleName>>", &self.module_name);
        let dsp_code = dsp_code.replace("<<structName>>", &self.get_struct_name());

        fs::write(&out_file, dsp_code).expect("failed to write to destination path");

        eprintln!("Wrote module:\n{}", out_file.to_str().unwrap());
    }

    #[must_use]
    pub fn build_to_stdout(&self, extra_flags: Vec<FaustArg>) -> String {
        let faust_output = self
            .to_command(extra_flags)
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

    pub fn build_json(&self) -> String {
        self.build_to_stdout(vec![FaustArg::Json()])
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
