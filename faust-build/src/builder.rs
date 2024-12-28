#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use crate::{
    code_option::{CodeOption, CodeOptionDiscriminants},
    compile_option::{CompileOption, CompileOptionDiscriminants, DspPath},
    macro_lib::get_name_token,
    option_map::{CodeOptionMap, CompileOptionMap},
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
    iter::FromIterator,
    path::{Path, PathBuf},
    process::Command,
};
use syn::parse_str;
use tempfile::NamedTempFile;

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

pub struct ArchitectureUI {}

impl ArchitectureInterface for ArchitectureUI {
    fn wrap(&self, builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream {
        let module_name = builder.module_name.as_ref().map_or("dsp", |v| v);
        let struct_name = builder.get_struct_name();
        let json_path = builder.get_json_path();
        if fs::exists(&json_path).unwrap_or(false) {
            builder.build_json();
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

pub struct FaustBuilder {
    faust_path: PathBuf,
    compile_options: CompileOptionMap,
    code_gen_options: CodeOptionMap,
    out_path: Option<PathBuf>,
    module_name: Option<String>,
    architecture: Option<Box<dyn ArchitectureInterface>>,
}

impl Default for FaustBuilder {
    fn default() -> Self {
        Self {
            faust_path: "faust".into(),
            compile_options: CompileOptionMap::default(),
            code_gen_options: CodeOptionMap::default(),
            out_path: None,
            architecture: None,
            module_name: Some("dsp".to_owned()),
        }
    }
}

impl FaustBuilder {
    #[must_use]
    pub fn get_compile_option(&self, key: &CompileOptionDiscriminants) -> Option<&CompileOption> {
        self.compile_options.get(key)
    }

    #[must_use]
    pub fn get_code_gen_option(&self, key: &CodeOptionDiscriminants) -> Option<&CodeOption> {
        self.code_gen_options.get(key)
    }

    pub fn set_compile_option(&mut self, arg: CompileOption) -> Option<CompileOption> {
        self.compile_options.insert(arg)
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

    pub fn set_architecture(&mut self, arch: Box<dyn ArchitectureInterface>) {
        self.architecture = Some(arch);
    }

    pub fn set_dsp_path(&mut self, dsp_path: impl Into<PathBuf>) {
        self.set_compile_option(CompileOption::dsp_path(dsp_path));
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
        b.set_compile_option(CompileOption::Json);
        b.set_architecture(Box::new(ArchitectureUI {}));
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
        builder.extend_compile_options([
            CompileOption::Json,
            CompileOption::DspPath(DspPath::File(dsp_path)),
        ]);
        builder.struct_name_from_dsp_name();
        builder.module_name_from_dsp_file_path();
        builder.architecture = Some(Box::new(ArchitectureUI {}));

        builder.extend_code_gen_options(extra_flags);
        builder
    }

    #[must_use]
    pub fn default_for_dsp_macro(faust_code: &str, extra_flags: CodeOptionMap) -> Self {
        let mut builder = Self::default();
        builder.write_temp_dsp_file(faust_code);
        builder.extend_compile_options([CompileOption::Json]);
        builder.struct_name_from_dsp_name();
        builder.module_name_from_struct_name();
        builder.architecture = Some(Box::new(ArchitectureUI {}));
        builder.extend_code_gen_options(extra_flags);
        builder
    }

    #[must_use]
    pub fn run_faust(&self) -> String {
        let faust_result = Command::new(&self.faust_path)
            .args(FaustArgsToCommandArgs::to_command_args(
                &self.compile_options,
            ))
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
        let dsp_code = match (
            self.get_compile_option(&CompileOptionDiscriminants::ArchFile),
            &self.architecture,
        ) {
            (Some(_), Some(_)) => panic!("Architecture File and Object are both specified"),
            (None, None) => {
                let architecture_default = ArchitectureDefault {};
                self.apply_arch_obj(&dsp_code, &architecture_default)
            }
            (None, Some(arch)) => self.apply_arch_obj(&dsp_code, arch.as_ref()),
            (Some(_), None) => {
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

    #[must_use]
    pub fn build_to_stdout_with_extra_args(&self, extra_flags: &CompileOptionMap) -> String {
        let faust_output = Command::new(&self.faust_path)
            .args(self.compile_options.to_command_args_merge(extra_flags))
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
        let _ = self
            .build_to_stdout_with_extra_args(&CompileOptionMap::from_iter([CompileOption::Xml]));
    }

    pub fn build_json(&self) {
        let _ = self
            .build_to_stdout_with_extra_args(&CompileOptionMap::from_iter([CompileOption::Json]));
    }

    pub(crate) fn extend_compile_options(
        &mut self,
        flags: impl IntoIterator<Item = CompileOption>,
    ) {
        self.compile_options.extend(flags);
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
        let msg = "DspPath is not set";
        let CompileOption::DspPath(path) = self
            .get_compile_option(&CompileOptionDiscriminants::DspPath)
            .expect(msg)
        else {
            panic!("{}", msg)
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
        self.set_compile_option(CompileOption::DspPath(DspPath::Temp(temp_path.into())));
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
