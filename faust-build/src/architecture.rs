use crate::builder::FaustBuilder;
use core::panic;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use syn::parse_str;

pub enum Architecture {
    None,
    Function(&'static dyn Fn(&FaustBuilder, TokenStream) -> TokenStream),
    Object(Box<dyn ObjectInterface>),
    File(PathBuf),
}

impl Architecture {
    #[must_use]
    pub fn ui() -> Self {
        Self::Function(&ui)
    }
    #[must_use]
    pub fn to_command_arg(&self) -> Option<&Path> {
        match self {
            Self::File(arch_file) => Some(arch_file),
            _ => None,
        }
    }

    fn apply_arch_function(
        builder: &FaustBuilder,
        dsp_code: &str,
        arch: &dyn Fn(&FaustBuilder, TokenStream) -> TokenStream,
    ) -> TokenStream {
        let ts = parse_str::<TokenStream>(dsp_code).expect("Failed to parse string into tokens");
        arch(builder, ts)
    }

    fn fix_arch_wrap(dsp_code: &str, struct_name: &str) -> String {
        let dsp_code = dsp_code.replace("<<moduleName>>", "dsp");
        dsp_code.replace("<<structName>>", struct_name)
    }

    pub(crate) fn apply(&self, builder: &FaustBuilder, dsp_code: &str) -> TokenStream {
        match self {
            Self::None => {
                //or would it be better to do really no architecture?
                Self::apply_arch_function(builder, dsp_code, &default)
            }
            Self::Function(architecture_function) => {
                Self::apply_arch_function(builder, dsp_code, architecture_function)
            }
            Self::Object(architecture_interface) => {
                let ts =
                    parse_str::<TokenStream>(dsp_code).expect("Failed to parse string into tokens");
                architecture_interface.apply(builder, ts)
            }
            Self::File(_path_buf) => {
                let dsp_code = Self::fix_arch_wrap(dsp_code, builder.get_struct_name());
                parse_str::<TokenStream>(&dsp_code).expect("Failed to parse string into tokens")
            }
        }
    }
}

impl Default for Architecture {
    fn default() -> Self {
        Self::Function(&default)
    }
}
pub trait ObjectInterface {
    fn apply(&self, builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream;
}

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn default(_builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream {
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

#[must_use]
#[allow(clippy::needless_pass_by_value)]
fn ui(builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream {
    let module_name = builder.get_module_name();
    let struct_name = builder.get_struct_name();
    let json_path = builder.get_json_path();
    match fs::exists(&json_path) {
        Ok(b) => {
            assert!(b, "json file not found at path: {:?}", json_path);
        }
        Err(err) => panic!("json file not found at path: {:?}", err),
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

// Architecture Object needs a usecase first
// pub struct ArchitectureUI {}

// impl ArchitectureInterface for ArchitectureUI {
//     fn wrap(&self, builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream {
//         let module_name = builder.get_module_name();
//         let struct_name = builder.get_struct_name();
//         let json_path = builder.get_json_path();
//         match fs::exists(&json_path) {
//             Ok(b) => {
//                 assert!(b, "json file not found at path: {:?}", json_path);
//             }
//             Err(err) => panic!("json file not found at path: {:?}", err),
//         }
//         let (ui_code, ui_reexport) =
//             FaustBuilder::get_ui_from_json(&json_path, module_name, struct_name);

//         let module_name = format_ident!("{}", module_name);
//         let struct_name = format_ident!("{}", struct_name);
//         quote! {
//             mod #module_name {
//                 #![allow(clippy::all)]
//                 #![allow(unused_parens)]
//                 #![allow(non_snake_case)]
//                 #![allow(non_camel_case_types)]
//                 #![allow(dead_code)]
//                 #![allow(unused_variables)]
//                 #![allow(unused_mut)]
//                 #![allow(non_upper_case_globals)]
//                 use faust_types::*;
//                 #dsp_code
//                 #ui_code
//             }

//             pub use #module_name::#struct_name;
//             #ui_reexport
//         }
//     }
// }
