#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_const_for_fn)]

use crate::{
    architecture::Architecture,
    code_option::{CodeOption, CodeOptionDiscriminants, CodeOptionMap},
    compile_options::CompileOptions,
    dsp_path::DspPath,
    CodeOptionToCommandArgs,
};
use heck::{CamelCase, SnakeCase};
use proc_macro2::TokenStream;
use std::{
    env,
    fs::{self},
    io::{BufWriter, Write},
    panic,
    path::{Path, PathBuf},
    process::Command,
    str,
};
use tempfile::{NamedTempFile, TempPath};

pub struct FaustBuilder {
    faust_path: PathBuf,
    code_gen_options: CodeOptionMap,
    module_name: Option<String>,
    out_path: Option<PathBuf>,
    compile_options: CompileOptions,
}

impl Default for FaustBuilder {
    fn default() -> Self {
        Self {
            faust_path: "faust".into(),
            code_gen_options: CodeOptionMap::default(),
            module_name: None,
            out_path: None,
            compile_options: CompileOptions::default(),
        }
    }
}

impl FaustBuilder {
    #[must_use]
    pub fn get_code_option(&self, key: &CodeOptionDiscriminants) -> Option<&CodeOption> {
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

    pub fn write_xml_file(&mut self) {
        self.compile_options.xml = true;
    }

    pub fn write_json_file(&mut self) {
        self.compile_options.json = true;
    }

    #[cfg(feature = "faust-ui")]
    pub fn default_for_file_with_ui(
        dsp_path: impl Into<PathBuf>,
        out_path: impl Into<PathBuf>,
    ) -> Self {
        let mut b = Self::default();
        b.set_dsp_path(dsp_path);
        b.set_out_path(out_path);
        b.struct_name_from_dsp_name();
        b.write_json_file();
        b.set_architecture(Architecture::ui());
        b
    }

    pub fn default_for_file(dsp_path: impl Into<PathBuf>, out_path: impl Into<PathBuf>) -> Self {
        let mut b = Self::default();
        b.set_dsp_path(dsp_path);
        b.set_out_path(out_path);
        b.struct_name_from_dsp_name();
        b
    }

    #[cfg(feature = "faust-ui")]
    #[must_use]
    pub fn default_for_include_macro(dsp_path: PathBuf, extra_flags: CodeOptionMap) -> Self {
        let mut builder = Self::default();
        builder.write_json_file();
        builder.set_dsp_path(dsp_path);
        builder.struct_name_from_dsp_name();
        builder.module_name_from_dsp_file_path();
        builder.set_architecture(Architecture::mod_ui());
        builder.extend_code_options(extra_flags);
        builder
    }

    #[cfg(feature = "faust-ui")]
    #[must_use]
    pub fn default_for_dsp_macro(faust_code: &str, extra_flags: CodeOptionMap) -> Self {
        let mut builder = Self::default();
        builder.write_temp_dsp_file(faust_code);
        builder.write_json_file();
        builder.struct_name_from_dsp_name();
        builder.module_name_from_struct_name();
        builder.set_architecture(Architecture::mod_ui());
        builder.extend_code_options(extra_flags);
        builder
    }

    #[must_use]
    pub fn run_faust(&self) -> String {
        let faust_result = Command::new(&self.faust_path)
            .args(self.compile_options.to_command_args())
            .args(CodeOptionToCommandArgs::to_command_args(
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

    fn pretty(ts: TokenStream) -> String {
        let st = syn::parse2(ts).expect("Failed to parse Rust code in wrapper");
        prettyplease::unparse(&st)
    }

    #[allow(clippy::must_use_candidate)]
    pub fn build(&self) -> TokenStream {
        let dsp_code = self.run_faust();
        let ts = self.compile_options.architecture.apply(self, &dsp_code);
        let dsp_code = Self::pretty(ts.clone());
        if let Some(out_path) = &self.out_path {
            fs::write(out_path, &dsp_code).expect("failed to write to destination path");
        }
        ts
    }

    pub fn extend_code_options(&mut self, flags: impl IntoIterator<Item = CodeOption>) {
        self.code_gen_options.extend(flags);
    }

    pub fn struct_name_from_dsp_name(&mut self) {
        let msg = "generated rust code could";
        let path = self.get_dsp_path();
        let faust_code = fs::read(path)
            .unwrap_or_else(|_| panic!("{} not be read at path: {}", msg, path.to_string_lossy()));
        let faust_code = String::from_utf8(faust_code)
            .unwrap_or_else(|_| panic!("{} interpreted as String", msg));
        let ts: proc_macro2::TokenStream = faust_code
            .parse()
            .unwrap_or_else(|_| panic!("{} parse as TokenStream", msg));
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
        self.module_name.as_ref().expect("cannot fail")
    }

    pub fn module_name_from_struct_name(&mut self) {
        let struct_name = self.get_struct_name();
        self.module_name = Some(struct_name.to_snake_case());
    }

    #[must_use]
    pub fn get_dsp_path(&self) -> &Path {
        let Some(path) = &self.compile_options.dsp_path else {
            panic!("DspPath is not set")
        };
        path
    }

    #[cfg(feature = "faust-ui")]
    pub fn generate_ui_from_json(
        json_path: &Path,
        struct_name: impl AsRef<str>,
    ) -> proc_macro2::TokenStream {
        let json_file = std::fs::File::open(json_path).expect("Failed to open json file");
        let json_reader = std::io::BufReader::new(json_file);
        let faust_json: faust_json::FaustJson = serde_json::from_reader(json_reader)
            .unwrap_or_else(|err| {
                panic!("json parsing error: {}", err);
            });
        faust_ui::generate_ui_code(&faust_json, struct_name)
    }

    #[must_use]
    pub fn get_struct_name(&self) -> &String {
        let msg = "No Struct Name defined";
        let CodeOption::StructName(struct_name) = self
            .get_code_option(&CodeOptionDiscriminants::StructName)
            .expect(msg)
        else {
            panic!("{}", msg)
        };
        struct_name
    }

    #[must_use]
    pub fn get_module_name(&self) -> &Option<String> {
        &self.module_name
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

fn strip_quotes(name: &proc_macro2::TokenTree) -> String {
    name.to_string()
        .strip_prefix('\"')
        .expect("prefix is not \"")
        .strip_suffix('\"')
        .expect("postfix is not \"")
        .to_string()
}

/// find the token that declares a key in the dsp file
pub(crate) fn get_declared_value(key: &str, ts: proc_macro2::TokenStream) -> Option<String> {
    let mut ii = ts.into_iter();
    while let Some(n) = ii.next() {
        if n.to_string() == "declare" {
            if let Some(n) = ii.next() {
                if n.to_string() == key {
                    if let Some(value) = ii.next() {
                        if let Some(semicolon) = ii.next() {
                            if semicolon.to_string() == ";" {
                                return Some(strip_quotes(&value));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[must_use]
pub(crate) fn get_name_token(ts: proc_macro2::TokenStream) -> String {
    get_declared_value("name", ts)
        .expect("name declaration is not found.\n Expect 'declare name NAMESTRING;' in faust code.")
}
