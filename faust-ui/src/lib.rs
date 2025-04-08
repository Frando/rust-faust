#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
    unused_crate_dependencies,
    clippy::unwrap_used
)]
#![allow(clippy::missing_panics_doc)]

use faust_json::FaustJson;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub mod enum_interface;
pub mod struct_interface;

#[must_use]
pub fn generate_ui_code(faust_json: &FaustJson, struct_name: impl AsRef<str>) -> TokenStream {
    let struct_name = format_ident!("{}", struct_name.as_ref());

    let ui_static_name = format_ident!("DSP_UI");
    let ui_type = format_ident!("DspUi");

    let ui_enum = enum_interface::create(faust_json, &struct_name);
    let struct_interface = struct_interface::create(faust_json, &ui_static_name, &ui_type);

    let ui_code = quote! {
        #ui_enum
        #struct_interface
    };
    ui_code
}
