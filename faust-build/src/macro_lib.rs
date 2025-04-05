#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::missing_panics_doc)]

use crate::{
    builder::{get_declared_value, FaustBuilder},
    code_option::CodeOption,
};
use heck::SnakeCase;
use std::{env, iter::FromIterator, path::PathBuf, str::FromStr};
use syn::{parse::Parse, Error, Expr, ExprArray, ExprPath, LitStr, Token};

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
        if input.is_empty() {
            Ok(Self {
                dsp_path,
                flags: Vec::new(),
            })
        } else {
            let _comma: Token![,] = input.parse()?;
            let flags = Self::parse_enums(input.parse()?)?;
            Ok(Self { dsp_path, flags })
        }
    }
}

#[cfg(feature = "faust-ui")]
#[must_use]
pub fn build_faust_file_from_macro(args: FileMacroArgs) -> proc_macro2::TokenStream {
    use crate::code_option::CodeOptionMap;

    let source_file =
        env::var("CARGO_MANIFEST_DIR").expect("environment variable CARGO_MANIFEST_DIR is not set");
    let folder: PathBuf = source_file.into();
    let flags = CodeOptionMap::from_iter(args.flags);
    let relative_dsp_path: PathBuf = args.dsp_path.value().into();
    let dsp_path = folder.join(&relative_dsp_path);
    assert!(
        dsp_path.exists(),
        "dsp file does not exist at: {:?}",
        dsp_path
    );

    let builder = FaustBuilder::default_for_include_macro(dsp_path, flags);
    builder.build()
}

#[cfg(feature = "faust-ui")]
#[must_use]
pub fn build_dsp_code_from_macro(input: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let faust_code = format!("{input}").replace(';', ";\n");

    let flags = get_flags_token(input.clone());
    let flags = CodeOption::arg_map_from_str_iter(flags.iter());

    let builder = FaustBuilder::default_for_dsp_macro(&faust_code, flags);

    builder.write_debug_dsp_file(&builder.get_struct_name().to_snake_case());
    let dsp_code = builder.build();
    builder.write_debug_json_file(&builder.get_struct_name().to_snake_case());
    dsp_code
}
