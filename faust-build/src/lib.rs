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

#[cfg(feature = "codegen")]
pub mod json;

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
    FaustBuilder::new(dsp_file).build_to_out_file(dest_path);
}

pub fn build_dsp_to_destination(dsp_file: &str, dest_path: &str) {
    FaustBuilder::new(dsp_file).build_to_out_file(dest_path);
}

#[derive(Default)]
pub enum Architecture {
    #[default]
    Default,
    File(PathBuf),
    #[cfg(feature = "codegen")]
    Fn(Box<dyn Fn(&mut FaustBuilder) -> proc_macro2::TokenStream>),
}

pub enum Input {
    File(PathBuf),
    String(String),
}

impl Default for Input {
    fn default() -> Self {
        Input::String("".into())
    }
}

pub struct FaustBuilder {
    faust_path: Option<PathBuf>,
    input: Input,
    /// Module name the dsp code will be encapsulated in. By default is "dsp".
    module_name: String,
    /// Name for the DSP struct. If None, we use camel cased file name.
    struct_name: Option<String>,
    use_double: bool,
    faust_args: Vec<FaustArg>,

    architecture: Option<Architecture>,
}

impl Default for FaustBuilder {
    fn default() -> Self {
        Self {
            faust_path: None,
            input: Input::default(),
            struct_name: None,
            module_name: "dsp".into(),
            use_double: false,
            faust_args: vec![],
            architecture: Some(Architecture::Default),
        }
    }
}

impl FaustBuilder {
    pub fn new(in_file: impl Into<PathBuf>) -> Self {
        Self {
            input: Input::File(in_file.into()),
            ..Default::default()
        }
    }

    pub fn new_from_string(code: String) -> Self {
        Self {
            input: Input::String(code),
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
    #[deprecated(note = "please use `set_architecture_file`")]
    pub fn set_arch_file(self, arch_file: impl Into<PathBuf>) -> Self {
        self.set_architecture_file(arch_file)
    }

    #[must_use]
    pub fn set_architecture_file(mut self, arch_file: impl Into<PathBuf>) -> Self {
        self.architecture = Some(Architecture::File(arch_file.into()));
        self
    }

    #[must_use]
    pub fn set_default_architecture(mut self) -> Self {
        self.architecture = Some(Architecture::Default);
        self
    }

    pub fn set_architecture_fn(
        mut self,
        architecture_fn: impl Fn(&mut FaustBuilder) -> proc_macro2::TokenStream + 'static,
    ) -> Self {
        self.architecture = Some(Architecture::Fn(Box::new(architecture_fn)));
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
            || match &self.input {
                Input::File(path) => faust_utils::struct_name_from_dsp_path(&path),
                Input::String(_) => todo!(),
            },
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

        let dsp_path = match &self.input {
            Input::File(path_buf) => path_buf.to_owned(),
            Input::String(code) => {
                let input_tempfile = NamedTempFile::new()
                    .expect("failed creating temporary file")
                    .keep()
                    .expect("Failed marking the temp file as `keep`")
                    .1;
                fs::write(&input_tempfile, code).expect("failed writing temporary file");
                input_tempfile
            }
        };
        args.push(FaustArg::DspPath(dsp_path));

        args.extend(extra_flags);
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

    pub fn extra_args_for_to_out_file(&mut self, intermediate_out_file: PathBuf) -> Vec<FaustArg> {
        let (architecture_file_path, _temp_file) = match self
            .architecture
            .take()
            .expect("Architecture already taken. Did you call `build` twice?")
        {
            Architecture::Default => {
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
            Architecture::File(path_buf) => (path_buf.to_owned(), None),
            Architecture::Fn(architecture_fn) => {
                let arch_tempfile = NamedTempFile::new().expect("failed creating temporary file");
                let architecture_code = architecture_fn(self);
                fs::write(arch_tempfile.path(), architecture_code.to_string())
                    .expect("failed writing temporary file");
                (arch_tempfile.path().into(), Some(arch_tempfile))
            }
        };

        vec![
            FaustArg::OutPath(intermediate_out_file),
            FaustArg::ArchFile(architecture_file_path.clone()),
        ]
    }

    pub fn build_to_out_file(&mut self, out_file: impl Into<PathBuf>) {
        let out_file: PathBuf = out_file.into();

        let dsp_code = self.build(vec![]);
        fs::write(&out_file, dsp_code).expect("failed to write to destination path");

        eprintln!("Wrote module:\n{}", out_file.to_str().unwrap());
    }

    #[must_use]
    pub fn build(&self, extra_flags: Vec<FaustArg>) -> String {
        let dsp_code = run_faust_command(self.to_command(extra_flags));
        let dsp_code = dsp_code.replace("<<moduleName>>", &self.module_name);
        let dsp_code = dsp_code.replace("<<structName>>", &self.get_struct_name());
        dsp_code
    }

    // pub fn build_xml_at_file(&self, out: &str) {
    //     let gen_xml_path: PathBuf = FaustArg::DspPath(self.in_file.clone()).xml_path();
    //     self.build_xml();
    //     fs::rename(&gen_xml_path, out).unwrap_or_else(|_| {
    //         panic!(
    //             "rename of xml file failed from '{:?}' to '{:?}'",
    //             gen_xml_path, out
    //         )
    //     });
    // }
}

pub fn run_faust_command(mut command: Command) -> String {
    let faust_output = command.output().expect("Failed to execute command");
    assert!(
        faust_output.status.success(),
        "faust compilation failed: {}",
        String::from_utf8(faust_output.stderr).unwrap()
    );
    String::from_utf8(faust_output.stdout).expect("could not parse stdout from command")
}
