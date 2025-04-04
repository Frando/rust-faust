use faust_build::FaustBuilder;
use proc_macro::{TokenStream, TokenTree};
use std::{
    fs::{self, read_to_string},
    io::{BufWriter, Write},
    path::{self, Path, PathBuf},
};
use tempfile::NamedTempFile;

fn strip_quotes(name: TokenTree) -> String {
    name.to_string()
        .strip_prefix('\"')
        .expect("prefix is not \"")
        .strip_suffix('\"')
        .expect("postfix is not \"")
        .to_string()
}

fn get_declared_value(key: &str, ts: TokenStream) -> Option<String> {
    // find the token that declares a key in the dsp file
    let mut ii = ts.into_iter();
    while let Some(n) = ii.next() {
        if n.to_string() == "declare" {
            if let Some(n) = ii.next() {
                if n.to_string() == key {
                    if let Some(value) = ii.next() {
                        if let Some(semicolon) = ii.next() {
                            if semicolon.to_string() == ";" {
                                return Some(strip_quotes(value));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn get_name_token(ts: TokenStream) -> String {
    get_declared_value("name", ts)
        .expect("name declaration is not found.\n Expect 'declare name NAMESTRING;' in faust code.")
}

fn get_architecture_token(ts: TokenStream) -> Option<String> {
    get_declared_value("architecture", ts)
}

fn get_flags_token(ts: TokenStream) -> Vec<String> {
    match get_declared_value("flags", ts) {
        None => vec![],
        Some(s) => s.split_whitespace().map(|s| s.to_owned()).collect(),
    }
}

fn write_temp_dsp_file(faust_code: String) -> NamedTempFile {
    let temp_dsp = NamedTempFile::new().expect("failed creating temp dsp file");
    let mut f = BufWriter::new(temp_dsp);
    f.write_all(faust_code.as_bytes())
        .expect("Unable to write to temp dsp file");
    f.into_inner().expect("temp dsp error on flush")
}

fn faust_build(
    faust_code: String,
    name: String,
    flags: Vec<String>,
    architecture: Option<String>,
) -> TokenStream {
    // define paths for .dsp and .rs files that help debugging
    let mut debug_dsp = Path::new(".")
        .join("target")
        .join("DEBUG_".to_owned() + &name)
        .with_extension("dsp");

    let debug_rs = Path::new(".")
        .join("target")
        .join("DEBUG_".to_owned() + &name)
        .with_extension("rs");

    let temp_rs = NamedTempFile::new().expect("failed creating temporary file");

    let temp_dsp = write_temp_dsp_file(faust_code);
    let temp_dsp_path = temp_dsp.path();
    let temp_dsp_path_str = temp_dsp_path
        .to_str()
        .expect("temp file dsp path contains non-UTF-8");
    let temp_rs_path_str = temp_rs
        .path()
        .to_str()
        .expect("temp file rs path contains non-UTF-8");

    if cfg!(debug_assertions) {
        fs::copy(temp_dsp_path, &debug_dsp).expect("temp dsp file cannot be copied to target");
    } else {
        let _ignore_error = fs::remove_file(&debug_dsp);
    }

    let mut b: FaustBuilder = FaustBuilder::new(temp_dsp_path_str, temp_rs_path_str)
        .set_struct_name(&name)
        .set_module_name(&("dsp_".to_owned() + &name));

    if let Some(a) = architecture {
        let p = PathBuf::from(&a);
        let ap = if p.is_absolute() {
            p
        } else {
            path::absolute(p).unwrap_or_else(|_| {
                panic!("could not make architecture path into a absolute path: '{a}'")
            })
        };
        b = b.set_arch_file(ap.to_str().unwrap());
    }

    let b = flags.into_iter().fold(b, |b, flag| b.faust_arg(flag));
    b.build();
    debug_dsp.set_extension("xml");

    b.build_xml_at_file(
        debug_dsp
            .to_str()
            .expect("debug path for xml is not a valid string"),
    );

    if cfg!(debug_assertions) {
        fs::copy(temp_rs_path_str, debug_rs).expect("rsfile cannot be copied to target");
    } else {
        let _ignore_error = fs::remove_file(debug_rs);
    }

    let stdout = read_to_string(temp_rs.path()).expect("rs file reading failed");
    stdout.parse().expect("rs file parsing failed")
}

#[proc_macro]
pub fn faust(input: TokenStream) -> TokenStream {
    let faust_code = format!("{}", input).replace(';', ";\n");
    let name = get_name_token(input.clone());
    let flags = get_flags_token(input.clone());
    let architecture = get_architecture_token(input);
    faust_build(faust_code, name, flags, architecture)
}
