#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::missing_panics_doc)]

use faust_build::{
    codegen::{get_flags_token, get_name_token},
    faust_arg::FaustArg,
    FaustBuilder,
};
use faust_description_json::deserialize::FaustDescriptionJson;
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

fn architecture(builder: &mut FaustBuilder) -> proc_macro2::TokenStream {
    let dsp_code: String = "".into();
    let ui_code: String = "".into();
    let ui_reexport: String = "".into();
    quote! {
        mod <<moduleName>> {
            #![allow(clippy::all)]
            #![allow(unused_parens)]
            #![allow(non_snake_case)]
            #![allow(non_camel_case_types)]
            #![allow(dead_code)]
            #![allow(unused_variables)]
            #![allow(unused_mut)]
            #![allow(non_upper_case_globals)]
            use faust_types::*;
            <<includeIntrinsic>>
            <<includeclass>>
        }

        pub use <<moduleName>>::<<structName>>;
    }
}

#[must_use]
pub fn faust_build(input: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let faust_code = format!("{input}").replace(';', ";\n");

    let name = get_name_token(input.clone());
    let module_name = format_ident!("dsp_{}", name);
    let struct_name = format_ident!("{}", name);

    let flags = get_flags_token(input.clone());
    let dsp_code = FaustBuilder::new_from_string(faust_code)
        .args([
            FaustArg::DebugWarnings,
            FaustArg::StructName(struct_name.to_string()),
        ])
        .args(flags)
        .set_architecture_fn(architecture)
        .build(vec![]);

    dsp_code.into_token_stream()
}
