use std::{
    fs::{self, File},
    io::BufReader,
};

use faust_build::FaustBuilder;
use faust_json::deserialize::FaustJson;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_str, Ident};

fn architecture(
    dsp_code: &TokenStream,
    ui_code: &TokenStream,
    ui_reexport: &TokenStream,
    module_name: &Ident,
    struct_name: &Ident,
) -> TokenStream {
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

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    let b = FaustBuilder::new("dsp/volume.dsp", "src/dsp.rs")
        .faust_arg("-xml")
        .faust_arg("-json")
        .set_use_double(true);

    let dsp_code: String = b.build_to_stdout(Vec::new());
    let dsp_code = parse_str::<TokenStream>(&dsp_code).expect("Failed to parse string into tokens");

    let file = File::open("dsp/volume.dsp.json").expect("Failed to open file");
    let reader = BufReader::new(file);
    let dsp_json: FaustJson = serde_json::from_reader(reader).unwrap_or_else(|err| {
        panic!("{}", err);
    });

    let module_name = format_ident!("{}", b.get_module_name());
    let struct_name = format_ident!("{}", b.get_struct_name());
    let (ui_code, ui_reexport) = dsp_json.ui(&module_name, &struct_name);

    let template = architecture(
        &dsp_code,
        &ui_code,
        &ui_reexport,
        &module_name,
        &struct_name,
    );

    println!("{}", &template.to_string());
    let parsed: syn::File = syn::parse_file(&template.to_string())
        .unwrap_or_else(|err| panic!("syn failed with: {}", err.into_compile_error()));
    let pp = prettyplease::unparse(&parsed);
    fs::write("src/dsp.rs", pp).expect("failed to write to destination path");
}
