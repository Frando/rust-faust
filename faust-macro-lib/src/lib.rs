#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::missing_panics_doc)]

use faust_build::{
    faust_arg::{FaustArg, FaustArgsToCommandArgs},
    faust_utils::json_path_from_dsp_path,
};
use faust_json::deserialize::FaustJson;
use quote::{format_ident, quote};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
    process::Command,
};
use syn::{parse_str, Ident};
use tempfile::NamedTempFile;

fn strip_quotes(name: &proc_macro2::TokenTree) -> String {
    name.to_string()
        .strip_prefix('\"')
        .expect("prefix is not \"")
        .strip_suffix('\"')
        .expect("postfix is not \"")
        .to_string()
}

fn get_declared_value(key: &str, ts: proc_macro2::TokenStream) -> Option<String> {
    // find the token that declares a key in the dsp file
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
pub fn get_name_token(ts: proc_macro2::TokenStream) -> String {
    get_declared_value("name", ts)
        .expect("name declaration is not found.\n Expect 'declare name NAMESTRING;' in faust code.")
}

pub fn get_flags_token(ts: proc_macro2::TokenStream) -> Vec<String> {
    get_declared_value("flags", ts).map_or_else(std::vec::Vec::new, |s| {
        s.split_whitespace()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    })
}

#[must_use]
pub fn write_temp_dsp_file(faust_code: &str) -> NamedTempFile {
    let temp_dsp = NamedTempFile::new().expect("failed creating temp dsp file");
    let mut f = BufWriter::new(temp_dsp);
    f.write_all(faust_code.as_bytes())
        .expect("Unable to write to temp dsp file");
    f.into_inner().expect("temp dsp error on flush")
}

pub fn faust_command(
    temp_dsp_path: &Path,
    struct_name: &impl ToString,
    flags: &[impl ToString],
) -> Command {
    let mut args: Vec<FaustArg> = vec![
        FaustArg::default_lang(),
        FaustArg::default_timeout(),
        FaustArg::DebugWarnings,
        FaustArg::StructName(struct_name.to_string()),
        FaustArg::Json(),
        FaustArg::DspPath(temp_dsp_path.to_path_buf()),
    ];
    for arg in flags {
        args.push(FaustArg::Custom(arg.to_string()));
    }

    let mut faust = Command::new("faust");
    faust.args(args.to_command_args());
    faust
}

#[must_use]
pub fn architecture(
    dsp_code: &proc_macro2::TokenStream,
    ui_code: &proc_macro2::TokenStream,
    ui_reexport: &proc_macro2::TokenStream,
    module_name: &Ident,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
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

pub fn write_debug_dsp_file(name: &str, temp_dsp_path: &Path) {
    // define paths for .dsp and .json files that help debugging
    let debug_dsp = Path::new(".")
        .join("target")
        .join("DEBUG_".to_owned() + name)
        .with_extension("dsp");
    if cfg!(debug_assertions) {
        fs::copy(temp_dsp_path, &debug_dsp).expect("temp dsp file cannot be copied to target");
    } else {
        let _ignore_error = fs::remove_file(&debug_dsp);
    }
}

pub fn write_debug_json_file(name: &str, temp_json_path: &Path) {
    let debug_json = Path::new(".")
        .join("target")
        .join("DEBUG_".to_owned() + name)
        .with_extension("json");
    if cfg!(debug_assertions) {
        fs::copy(temp_json_path, &debug_json).expect("temp json file cannot be copied to target");
    } else {
        let _ignore_error = fs::remove_file(&debug_json);
    }
}

fn write_debug_rs_file(name: &str, dsp_code: &str) {
    let debug_rs = Path::new(".")
        .join("target")
        .join("DEBUG_".to_owned() + name)
        .with_extension("rs");
    if cfg!(debug_assertions) {
        fs::write(debug_rs, dsp_code).expect("failed to write debug rs file");
    } else {
        let _ignore_error = fs::remove_file(debug_rs);
    }
}

#[must_use]
pub fn parse_dsp_code(name: &str, stdout: Vec<u8>) -> proc_macro2::TokenStream {
    let dsp_code: String = String::from_utf8(stdout).expect("could not parse stdout from command");
    write_debug_rs_file(name, &dsp_code);
    let dsp_code: proc_macro2::TokenStream =
        parse_str(&dsp_code).expect("Failed to parse generated by faust rust code into tokens");
    dsp_code
}

fn get_ui_from_json(
    temp_json_path: &Path,
    module_name: &Ident,
    struct_name: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let json_file = File::open(temp_json_path).expect("Failed to open json file");
    let json_reader = BufReader::new(json_file);
    let faust_json: FaustJson = serde_json::from_reader(json_reader).unwrap_or_else(|err| {
        panic!("json parsing error: {err}");
    });
    faust_json.ui(module_name, struct_name)
}

#[must_use]
pub fn faust_build(input: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let faust_code = format!("{input}").replace(';', ";\n");
    let temp_dsp = write_temp_dsp_file(&faust_code);
    let temp_dsp_path = temp_dsp.path();

    let temp_json_path = &json_path_from_dsp_path(temp_dsp_path);

    let name = get_name_token(input.clone());
    let module_name = format_ident!("dsp_{}", name);
    let struct_name = format_ident!("{}", name);
    write_debug_dsp_file(&name, temp_dsp_path);

    let flags = get_flags_token(input.clone());
    let mut faust = faust_command(temp_dsp_path, &name, &flags);
    let faust_result = faust.output().expect("Failed to execute faust");
    assert!(
        faust_result.status.success(),
        "faust compilation failed: {}",
        String::from_utf8(faust_result.stderr).unwrap()
    );
    write_debug_json_file(&name, temp_json_path);
    let stderr = String::from_utf8(faust_result.stderr).unwrap();
    assert!(
        !stderr.contains("WARNING"),
        "Fail on warning in stderr: \n{stderr}"
    );

    let dsp_code = parse_dsp_code(&name, faust_result.stdout);
    let (ui_code, ui_reexport) = get_ui_from_json(temp_json_path, &module_name, &struct_name);

    architecture(
        &dsp_code,
        &ui_code,
        &ui_reexport,
        &module_name,
        &struct_name,
    )
}
