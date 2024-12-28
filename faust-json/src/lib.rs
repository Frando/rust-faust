#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]

use deserialize::FaustJson;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub mod deserialize;
pub mod enum_interface;
pub mod struct_interface;

impl FaustJson {
    #[must_use]
    pub fn ui(
        &self,
        module_name: impl AsRef<str>,
        struct_name: impl AsRef<str>,
    ) -> (TokenStream, TokenStream) {
        let module_name = format_ident!("{}", module_name.as_ref());
        let struct_name = format_ident!("{}", struct_name.as_ref());

        let ui_static_name = format_ident!("UI");
        let ui_type = format_ident!("NewUI");

        let (ui_enum, has_active, has_passive) = enum_interface::create(self, &struct_name);
        let active_line = enum_interface::reexport_active_tokenstream(has_active, &module_name);
        let passive_line = enum_interface::reexport_passive_tokenstream(has_passive, &module_name);
        let struct_interface = struct_interface::create(self, &ui_static_name, &ui_type);

        let ui_code = quote! {
            #ui_enum
            #struct_interface
        };
        let ui_reexport = quote! {
            #active_line
            #passive_line
            pub use #module_name::#ui_static_name;
        };
        (ui_code, ui_reexport)
    }
}
