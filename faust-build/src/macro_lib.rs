#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::missing_panics_doc)]

use crate::{builder::FaustBuilder, code_option::CodeOption, option_map::CodeOptionMap};
use heck::SnakeCase;
use proc_macro2::TokenStream;
use std::{env, iter::FromIterator, path::PathBuf, str::FromStr};
use syn::{parse::Parse, parse_str, Expr, ExprArray, LitStr, Token};
use syn::{Error, ExprPath};

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

fn get_flags_token(ts: proc_macro2::TokenStream) -> Vec<String> {
    get_declared_value("flags", ts).map_or_else(std::vec::Vec::new, |s| {
        s.split_whitespace()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    })
}

pub struct FileMacroArgs {
    pub dsp_path: LitStr,
    pub flags: Vec<CodeOption>,
}

impl FileMacroArgs {
    fn parse_enums(input_expr: ExprArray) -> syn::Result<Vec<CodeOption>> {
        let elems = input_expr.elems;
        elems
            .iter()
            .map(|expr| {
                let Expr::Path(ExprPath { path, .. }) = expr else {
                    return Result::Err(Error::new_spanned(
                        expr,
                        "Can not parse Array Element as Enum Variant",
                    ));
                };

                let Some(name) = path.get_ident().map(std::string::ToString::to_string) else {
                    return Result::Err(Error::new_spanned(
                        path,
                        "Can not parse Array Element as CodeGenerationOption Enum Variant",
                    ));
                };
                let Ok(fa) = CodeOption::from_str(&name) else {
                    return Result::Err(Error::new_spanned(
                        path,
                        format!("Can not parse Array Element as CodeGenerationOption Enum Variant {name}"),
                    ));
                };
                Ok(fa)
            })
            .collect()
    }
}

impl Parse for FileMacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dsp_path = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let flags = Self::parse_enums(input.parse()?)?;
        Ok(Self { dsp_path, flags })
    }
}

#[must_use]
pub fn build_faust_file_from_macro(args: FileMacroArgs) -> proc_macro2::TokenStream {
    let source_file = env::var("CARGO_MANIFEST_DIR").unwrap();
    let folder: PathBuf = source_file.into();
    let flags = CodeOptionMap::from_iter(args.flags);
    let relative_dsp_path: PathBuf = args.dsp_path.value().into();
    let dsp_path = folder.join(&relative_dsp_path);
    assert!(
        dsp_path.exists(),
        "dsp file does not exist at: {:?}",
        dsp_path
    );

    let builder = FaustBuilder::default_for_file_macro(dsp_path, flags);
    let build = builder.build();
    //a bit stupid to parse the string again to a token stream
    parse_str::<TokenStream>(&build).unwrap()
}

#[must_use]
pub fn build_dsp_code_from_macro(input: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let faust_code = format!("{input}").replace(';', ";\n");

    let flags = get_flags_token(input.clone());
    let flags = CodeOption::arg_map_from_str_iter(flags.iter());

    let builder = FaustBuilder::default_for_dsp_macro(&faust_code, flags);

    builder.write_debug_dsp_file(&builder.get_struct_name().to_snake_case());
    let build = builder.build();
    builder.write_debug_json_file(&builder.get_struct_name().to_snake_case());
    //a bit stupid to parse the string again to a token stream
    parse_str::<TokenStream>(&build).unwrap()
}
