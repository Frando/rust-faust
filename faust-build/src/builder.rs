#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use crate::{
    code_option::{CodeOption, CodeOptionDiscriminants},
    compile_options::CompileOptions,
    macro_lib::get_name_token,
    option_map::CodeOptionMap,
    ArchitectureInterface, FaustArgsToCommandArgs,
};
use core::{panic, str};
use faust_json::deserialize::FaustJson;
use heck::{CamelCase, SnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    env,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};
use syn::parse_str;
use tempfile::{NamedTempFile, TempPath};

static DEFAULT_MODULE_NAME: &str = "dsp";
pub struct ArchitectureDefault {}

impl ArchitectureInterface for ArchitectureDefault {
    fn wrap(&self, _builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream {
        quote! {
                #![allow(clippy::all)]
                #![allow(unused_parens)]
                #![allow(non_snake_case)]
                #![allow(non_camel_case_types)]
                #![allow(dead_code)]
                #![allow(unused_variables)]
                #![allow(unused_mut)]
                #![allow(non_upper_case_globals)]
                use faust_types::*;
                #dsp_code
        }
    }
}

#[derive(Debug, Clone)]
pub enum DspPath {
    File(PathBuf),
    Temp(Rc<TempPath>),
}

impl Deref for DspPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::File(path_buf) => path_buf,
            Self::Temp(rc) => rc,
        }
    }
}

impl PartialEq for DspPath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(l0), Self::File(r0)) => l0 == r0,
            (Self::Temp(_l0), Self::Temp(_r0)) => false,
            _ => false,
        }
    }
}
impl Eq for DspPath {}

pub struct ArchitectureUI {}

impl ArchitectureInterface for ArchitectureUI {
    fn wrap(&self, builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream {
        let module_name = builder.get_module_name();
        let struct_name = builder.get_struct_name();
        let json_path = builder.get_json_path();
        match fs::exists(&json_path) {
            Ok(b) => {
                assert!(b, "json file not found at path: {:?}", json_path);
            }
            Err(err) => panic!("json file not found at path: {:?}", err),
        }
        let (ui_code, ui_reexport) =
            FaustBuilder::get_ui_from_json(&json_path, module_name, struct_name);

        let module_name = format_ident!("{}", module_name);
        let struct_name = format_ident!("{}", struct_name);
        quote! {
            mod #module_name {
                #![allow(clippy::all)]
                #![allow(unused_parens)]
                #![allow(non_snake_case)]
                #![allow(non_camel_case_types)]
                #![allow(dead_code)]
                #![allow(unused_variables)]
                #![allow(unused_mut)]
                #![allow(non_upper_case_globals)]
                use faust_types::*;
                #dsp_code
                #ui_code
            }

            pub use #module_name::#struct_name;
            #ui_reexport
        }
    }
}

pub enum Architecture {
    None,
    Function(bool), //todo
    Object(Box<dyn ArchitectureInterface>),
    File(PathBuf),
}
impl Architecture {
    #[must_use]
    pub fn to_command_arg(&self) -> Option<&Path> {
        match self {
            Self::File(arch_file) => Some(arch_file),
            _ => None,
        }
    }
}

pub struct FaustBuilder {
    faust_path: PathBuf,
    code_gen_options: CodeOptionMap,
    module_name: Option<String>,
    out_path: Option<PathBuf>, //out_path is not used in the compile options!
    compile_options: CompileOptions,
}

impl Default for FaustBuilder {
    fn default() -> Self {
        Self {
            faust_path: "faust".into(),
            code_gen_options: CodeOptionMap::default(),
            module_name: Some("dsp".to_owned()),
            out_path: None,
            compile_options: CompileOptions::default(),
        }
    }
}

impl FaustBuilder {
    #[must_use]
    pub fn get_code_gen_option(&self, key: &CodeOptionDiscriminants) -> Option<&CodeOption> {
        self.code_gen_options.get(key)
    }

    pub fn set_code_option(&mut self, arg: CodeOption) -> Option<CodeOption> {
        self.code_gen_options.insert(arg)
    }

    pub fn set_faust_path(&mut self, faust_path: impl Into<PathBuf>) {
        self.faust_path = faust_path.into();
    }

    pub fn set_out_path(&mut self, out_path: impl Into<PathBuf>) {
        self.out_path = Some(out_path.into());
    }

    pub fn set_module_name(&mut self, module_name: impl Into<String>) {
        self.module_name = Some(module_name.into());
    }

    pub fn set_architecture(&mut self, arch: Architecture) {
        self.compile_options.architecture = arch;
    }

    pub fn set_dsp_path(&mut self, dsp_path: impl Into<PathBuf>) {
        self.compile_options.dsp_path = Some(DspPath::File(dsp_path.into()));
    }
    pub fn set_dsp_temp_path(&mut self, temp_path: impl Into<TempPath>) {
        self.compile_options.dsp_path = Some(DspPath::Temp(temp_path.into().into()));
    }

    pub fn set_xml(&mut self) {
        self.compile_options.xml = true;
    }

    pub fn set_json(&mut self) {
        self.compile_options.json = true;
    }

    pub fn default_for_file_with_ui(
        dsp_path: impl Into<PathBuf>,
        out_path: impl Into<PathBuf>,
    ) -> Self {
        let mut b = Self::default();
        b.set_dsp_path(dsp_path);
        b.set_out_path(out_path);
        b.struct_name_from_dsp_name();
        b.module_name_from_dsp_file_path();
        b.set_json();
        b.set_architecture(Architecture::Object(Box::new(ArchitectureUI {})));
        b
    }

    pub fn default_for_file(dsp_path: impl Into<PathBuf>, out_path: impl Into<PathBuf>) -> Self {
        let mut b = Self::default();
        b.set_dsp_path(dsp_path);
        b.set_out_path(out_path);
        b.struct_name_from_dsp_name();
        b.module_name_from_dsp_file_path();
        b
    }

    #[must_use]
    pub fn default_for_file_macro(dsp_path: PathBuf, extra_flags: CodeOptionMap) -> Self {
        let mut builder = Self::default();
        builder.set_json();
        builder.set_dsp_path(dsp_path);
        builder.struct_name_from_dsp_name();
        builder.module_name_from_dsp_file_path();
        builder.set_architecture(Architecture::Object(Box::new(ArchitectureUI {})));
        builder.extend_code_gen_options(extra_flags);
        builder
    }

    #[must_use]
    pub fn default_for_dsp_macro(faust_code: &str, extra_flags: CodeOptionMap) -> Self {
        let mut builder = Self::default();
        builder.write_temp_dsp_file(faust_code);
        builder.set_json();
        builder.struct_name_from_dsp_name();
        builder.module_name_from_struct_name();
        builder.set_architecture(Architecture::Object(Box::new(ArchitectureUI {})));
        builder.extend_code_gen_options(extra_flags);
        builder
    }

    // #[must_use]
    // pub fn compile_options_to_command_args(&self) -> Vec<&OsStr> {
    //     let mut r = Vec::<&OsStr>::new();
    //     if let Some(arch_file) = self.architecture.to_command_arg() {
    //         r.push("-a".as_ref());
    //         r.push(arch_file.as_ref());
    //     }
    //     if let Some(import_dir) = &self.import_dir {
    //         r.push("-I".as_ref());
    //         r.push(import_dir.as_ref());
    //     }
    //     if self.xml {
    //         r.push("-xml".as_ref());
    //     }
    //     if self.json {
    //         r.push("-json".as_ref());
    //     }
    //     r.push("-lang".as_ref());
    //     r.push(self.lang.as_ref());

    //     if self.debug_warnings {
    //         r.push("-wall".as_ref());
    //     }
    //     // 120 is default
    //     if let Some(timeout) = &self.timeout {
    //         r.push("-t".as_ref());
    //         r.push(timeout.as_ref());
    //     }
    //     if let Some(dsp_path) = &self.dsp_path {
    //         r.push(dsp_path.as_ref());
    //     } else {
    //         panic!("No Path to DSP file provided")
    //     }
    //     r
    // }

    #[must_use]
    pub fn run_faust(&self) -> String {
        let faust_result = Command::new(&self.faust_path)
            .args(self.compile_options.to_command_args())
            .args(FaustArgsToCommandArgs::to_command_args(
                &self.code_gen_options,
            ))
            .output()
            .expect("Failed to execute faust");
        let stderr =
            str::from_utf8(&faust_result.stderr).expect("could not parse stderr from command");

        assert!(
            faust_result.status.success(),
            "faust compilation failed: {}",
            stderr
        );

        assert!(
            !stderr.contains("WARNING"),
            "Fail on warning in stderr: \n{}",
            stderr
        );
        String::from_utf8(faust_result.stdout).expect("could not parse stdout from command")
    }

    fn fix_arch_wrap(dsp_code: &str, struct_name: &str) -> String {
        let dsp_code = dsp_code.replace("<<moduleName>>", "dsp");
        dsp_code.replace("<<structName>>", struct_name)
    }

    fn pretty(ts: TokenStream) -> String {
        let st = syn::parse2(ts).expect("Failed to parse Rust code in wrapper");
        prettyplease::unparse(&st)
    }

    fn apply_arch_obj(&self, dsp_code: &str, arch: &dyn ArchitectureInterface) -> String {
        let ts = parse_str::<TokenStream>(dsp_code).expect("Failed to parse string into tokens");
        let ts = arch.wrap(self, ts);
        Self::pretty(ts)
    }

    #[allow(clippy::must_use_candidate)]
    pub fn build(&self) -> String {
        let dsp_code = self.run_faust();
        let dsp_code = match &self.compile_options.architecture {
            Architecture::None => {
                //or really none?
                let architecture_default = ArchitectureDefault {};
                self.apply_arch_obj(&dsp_code, &architecture_default)
            }
            Architecture::Function(_) => todo!(),
            Architecture::Object(architecture_interface) => {
                self.apply_arch_obj(&dsp_code, architecture_interface.as_ref())
            }
            Architecture::File(_path_buf) => {
                let dsp_code = Self::fix_arch_wrap(&dsp_code, self.get_struct_name());

                let ts = parse_str::<TokenStream>(&dsp_code)
                    .expect("Failed to parse string into tokens");
                Self::pretty(ts)
            }
        };

        if let Some(out_path) = &self.out_path {
            fs::write(out_path, &dsp_code).expect("failed to write to destination path");
        }
        dsp_code
    }

    pub(crate) fn extend_code_gen_options(&mut self, flags: impl IntoIterator<Item = CodeOption>) {
        self.code_gen_options.extend(flags);
    }

    pub fn struct_name_from_dsp_name(&mut self) {
        let path = self.get_dsp_path();
        let faust_code = fs::read(path).unwrap();
        let faust_code = String::from_utf8(faust_code).unwrap();
        let ts: proc_macro2::TokenStream = faust_code.parse().unwrap();
        let sn = get_name_token(ts);
        let sn = sn.to_camel_case();
        self.set_code_option(CodeOption::StructName(sn));
    }

    pub fn module_name_from_dsp_file_path(&mut self) -> &str {
        let module_name = self
            .get_dsp_path()
            .file_stem()
            .expect("dsp_path does not end with a filename")
            .to_str()
            .expect("dsp path is not utf8")
            .to_snake_case();

        self.module_name = Some(module_name);
        self.module_name.as_ref().unwrap()
    }

    pub fn module_name_from_struct_name(&mut self) {
        let struct_name = self.get_struct_name();
        self.module_name = Some(format!("dsp_{}", struct_name.to_string().to_snake_case()));
    }

    #[must_use]
    pub fn get_dsp_path(&self) -> &Path {
        let Some(path) = &self.compile_options.dsp_path else {
            panic!("DspPath is not set")
        };
        path
    }

    fn get_ui_from_json(
        json_path: &Path,
        module_name: impl AsRef<str>,
        struct_name: impl AsRef<str>,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let json_file = File::open(json_path).expect("Failed to open json file");
        let json_reader = BufReader::new(json_file);
        let faust_json: FaustJson = serde_json::from_reader(json_reader).unwrap_or_else(|err| {
            panic!("json parsing error: {}", err);
        });
        faust_json.ui(module_name, struct_name)
    }

    #[must_use]
    pub fn get_struct_name(&self) -> &String {
        let CodeOption::StructName(struct_name) = self
            .get_code_gen_option(&CodeOptionDiscriminants::StructName)
            .unwrap()
        else {
            panic!()
        };
        struct_name
    }

    #[must_use]
    pub fn get_module_name(&self) -> &str {
        self.module_name
            .as_ref()
            .map_or(DEFAULT_MODULE_NAME, |module_name| module_name)
    }

    #[must_use]
    pub fn get_json_path(&self) -> PathBuf {
        let dsp_path = self.get_dsp_path();
        let gen_json_fn = dsp_path.to_str().expect("dsp path is not utf8").to_owned() + ".json";
        PathBuf::from(gen_json_fn)
    }

    #[must_use]
    pub fn xml_path_from_dsp_path(&self) -> PathBuf {
        let dsp_path = self.get_dsp_path();
        let mut extension = dsp_path.extension().unwrap_or_default().to_owned();
        extension.push(".xml");
        dsp_path.with_extension(extension)
    }

    pub fn write_temp_dsp_file(&mut self, faust_code: &str) {
        let temp_dsp = NamedTempFile::new().expect("failed creating temp dsp file");
        let mut f = BufWriter::new(temp_dsp);
        f.write_all(faust_code.as_bytes())
            .expect("Unable to write to temp dsp file");
        let temp_path = f
            .into_inner()
            .expect("temp dsp error on flush")
            .into_temp_path();
        self.set_dsp_temp_path(temp_path);
    }

    pub fn write_debug_dsp_file(&self, name: &str) {
        // define paths for .dsp files that help debugging
        let debug_dsp = Path::new(
            &(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var not set")),
        )
        .join("DEBUG_".to_owned() + name)
        .with_extension("dsp");
        if cfg!(debug_assertions) {
            fs::copy(self.get_dsp_path(), &debug_dsp)
                .expect("temp dsp file cannot be copied to target");
        } else {
            let _ignore_error = fs::remove_file(&debug_dsp);
        }
    }

    pub fn write_debug_json_file(&self, name: &str) {
        let debug_json = Path::new(
            &(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var not set")),
        )
        .join("DEBUG_".to_owned() + name)
        .with_extension("json");
        if cfg!(debug_assertions) {
            fs::copy(self.get_json_path(), &debug_json)
                .expect("temp json file cannot be copied to target");
        } else {
            let _ignore_error = fs::remove_file(&debug_json);
        }
    }

    pub fn write_debug_rs_file(&self, name: &str, dsp_code: &str) {
        let debug_rs = Path::new(
            &(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var not set")),
        )
        .join("DEBUG_".to_owned() + name)
        .with_extension("rs");
        if cfg!(debug_assertions) {
            fs::write(debug_rs, dsp_code).expect("failed to write debug rs file");
        } else {
            let _ignore_error = fs::remove_file(debug_rs);
        }
    }
}
