use std::vec;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

use crate::deserialize::FaustJson;
use crate::deserialize::LayoutItem;

const UIENUMPREFIX: &str = "UI";
const UIENUMPOSTFIX: &str = "Shortname";
const UIENUMACTIVE: &str = "Active";
const UIENUMPASSIVE: &str = "Passive";

#[must_use]
pub fn reexport_active_tokenstream(has_active: bool, module_name: &Ident) -> TokenStream {
    let ui_enum_active = active_ident();
    if has_active {
        quote! {
            pub use #module_name::#ui_enum_active;
        }
    } else {
        proc_macro2::TokenStream::new()
    }
}

#[must_use]
pub fn reexport_passive_tokenstream(has_passive: bool, module_name: &Ident) -> TokenStream {
    let ui_enum_passive = passive_ident();
    if has_passive {
        quote! {
            pub use #module_name::#ui_enum_passive;
        }
    } else {
        proc_macro2::TokenStream::new()
    }
}

#[must_use]
pub fn active_ident() -> Ident {
    format_ident!("{UIENUMPREFIX}{UIENUMACTIVE}{UIENUMPOSTFIX}")
}

#[must_use]
pub fn passive_ident() -> Ident {
    format_ident!("{UIENUMPREFIX}{UIENUMPASSIVE}{UIENUMPOSTFIX}")
}

struct ParamInfo {
    is_active: bool,
    shortname: Ident,
    varname: Ident,
}

impl ParamInfo {
    fn active(shortname: &str, varname: &str) -> Vec<Self> {
        vec![Self {
            is_active: true,
            shortname: format_ident!("{shortname}"),
            varname: format_ident!("{varname}"),
        }]
    }
    fn passive(shortname: &str, varname: &str) -> Vec<Self> {
        vec![Self {
            is_active: false,
            shortname: format_ident!("{shortname}"),
            varname: format_ident!("{varname}"),
        }]
    }
}
trait GetParmInfo {
    fn get_param_info(&self) -> Vec<ParamInfo>;
}

impl GetParmInfo for FaustJson {
    fn get_param_info(&self) -> Vec<ParamInfo> {
        self.ui
            .iter()
            .flat_map(GetParmInfo::get_param_info)
            .collect()
    }
}

impl GetParmInfo for LayoutItem {
    fn get_param_info(&self) -> Vec<ParamInfo> {
        match self {
            Self::HGroup { items, .. }
            | Self::VGroup { items, .. }
            | Self::TGroup { items, .. } => {
                items.iter().flat_map(GetParmInfo::get_param_info).collect()
            }
            Self::VSlider {
                shortname, varname, ..
            }
            | Self::HSlider {
                shortname, varname, ..
            }
            | Self::NEntry {
                shortname, varname, ..
            }
            | Self::Button {
                shortname, varname, ..
            }
            | Self::CheckBox {
                shortname, varname, ..
            } => ParamInfo::active(shortname, varname),
            Self::VBarGraph {
                shortname, varname, ..
            }
            | Self::HBarGraph {
                shortname, varname, ..
            } => ParamInfo::passive(shortname, varname),
            Self::Soundfile {
                address, varname, ..
            } => ParamInfo::active(address, varname),
        }
    }
}

fn create_qualified_enum(infos: &[&ParamInfo], is_active: bool) -> TokenStream {
    let enumname = if is_active {
        active_ident()
    } else {
        passive_ident()
    };
    let i: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| format_ident!("{}", param_info.shortname).to_token_stream())
        .collect();
    quote! {
        #[derive(Debug, Clone, Copy,EnumIter, EnumCount)]
        pub enum #enumname {
            #(#i ),*
        }
    }
}

fn create_active_impl(infos: &[&ParamInfo], dsp_name: &Ident) -> TokenStream {
    let enum_name = active_ident();
    let matches: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            let varname = format_ident!("{}", param_info.varname);
            quote! { #enum_name::#shortname => dsp.#varname = value}
        })
        .collect();
    quote! {
        impl UISet<#dsp_name,FaustFloat> for #enum_name {
            fn set(&self, dsp: &mut #dsp_name, value: FaustFloat) {
                match self {
                    #(#matches ),*
                }
            }
        }
    }
}

fn create_passive_impl(infos: &[&ParamInfo], dsp_name: &Ident) -> TokenStream {
    let enum_name = passive_ident();
    let dsp_name = format_ident!("{dsp_name}");
    let matches: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            let varname = format_ident!("{}", param_info.varname);
            quote! { #enum_name::#shortname => dsp.#varname}
        })
        .collect();
    quote! {
        impl UIGet<#dsp_name> for #enum_name {
            type F = FaustFloat;
            fn get(&self, dsp: & #dsp_name) -> Self::F {
                match self {
                #(#matches ),*
                }
            }
        }
    }
}

fn create_from_paraminfo(v: &[ParamInfo], dsp_name: &Ident) -> (TokenStream, bool, bool) {
    let active: Vec<&ParamInfo> = v.iter().filter(|i| i.is_active).collect();
    let passive: Vec<&ParamInfo> = v.iter().filter(|i| !i.is_active).collect();
    let (active_enum, active_impl, has_active) = if active.is_empty() {
        (TokenStream::new(), TokenStream::new(), false)
    } else {
        (
            create_qualified_enum(&active, true),
            create_active_impl(&active, dsp_name),
            true,
        )
    };
    let (passive_enum, passive_impl, has_passive) = if passive.is_empty() {
        (TokenStream::new(), TokenStream::new(), false)
    } else {
        (
            create_qualified_enum(&passive, false),
            create_passive_impl(&passive, dsp_name),
            true,
        )
    };
    (
        quote::quote! {
            use strum_macros::{EnumIter,EnumCount};

            #active_enum
            #active_impl
            #passive_enum
            #passive_impl
        },
        has_active,
        has_passive,
    )
}

#[must_use]
pub fn create(dsp_json: &FaustJson, dsp_name: &Ident) -> (TokenStream, bool, bool) {
    let param_info = dsp_json.get_param_info();
    create_from_paraminfo(&param_info, dsp_name)
}
