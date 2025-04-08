use faust_json::{FaustJson, LayoutItem};
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::vec;
use syn::Ident;

const UIENUMPREFIX: &str = "UI";
const UIENUMVALUE: &str = "Value";
const UIENUMACTIVE: &str = "Active";
const UIENUMPASSIVE: &str = "Passive";

#[must_use]
pub fn enum_active_value_ident() -> Ident {
    format_ident!("{UIENUMPREFIX}{UIENUMACTIVE}{UIENUMVALUE}")
}

#[must_use]
pub fn enum_passive_value_ident() -> Ident {
    format_ident!("{UIENUMPREFIX}{UIENUMPASSIVE}{UIENUMVALUE}")
}

#[must_use]
pub fn enum_active_discriminants_ident() -> Ident {
    format_ident!("{UIENUMPREFIX}{UIENUMACTIVE}")
}

#[must_use]
pub fn enum_passive_discriminants_ident() -> Ident {
    format_ident!("{UIENUMPREFIX}{UIENUMPASSIVE}")
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
            } => ParamInfo::active(&shortname.to_camel_case(), varname),
            Self::VBarGraph {
                shortname, varname, ..
            }
            | Self::HBarGraph {
                shortname, varname, ..
            } => ParamInfo::passive(&shortname.to_camel_case(), varname),
            Self::Soundfile {
                address, varname, ..
            } => ParamInfo::active(address, varname),
        }
    }
}

fn create_qualified_enum(infos: &[&ParamInfo], is_active: bool) -> TokenStream {
    let enum_name = if is_active {
        enum_active_value_ident()
    } else {
        enum_passive_value_ident()
    };
    let discriminants_name = if is_active {
        enum_active_discriminants_ident()
    } else {
        enum_passive_discriminants_ident()
    };
    let i: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| format_ident!("{}", param_info.shortname).to_token_stream())
        .collect();
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter, EnumCount,EnumDiscriminants,VariantNames)]
        #[strum_discriminants(derive(Display,EnumIter, EnumCount,IntoStaticStr,VariantArray,VariantNames,Hash))]
        #[strum_discriminants(name(#discriminants_name))]
        pub enum #enum_name {
            #(#i(FaustFloat)),*
        }
    }
}

fn create_active_impl(infos: &[&ParamInfo], dsp_name: &Ident) -> TokenStream {
    let enum_name = enum_active_value_ident();
    let matches_set: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            let varname = format_ident!("{}", param_info.varname);
            quote! { #enum_name::#shortname(value) => dsp.#varname = *value}
        })
        .collect();
    let matches_get: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            quote! { #enum_name::#shortname(value) => *value}
        })
        .collect();
    let enum_name_discriminant = enum_active_discriminants_ident();
    let matches_discriminant: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            let varname = format_ident!("{}", param_info.varname);
            quote! { #enum_name_discriminant::#shortname => dsp.#varname = value}
        })
        .collect();
    let matches_value: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            quote! { #enum_name_discriminant::#shortname => #enum_name::#shortname(value)}
        })
        .collect();
    quote! {
        impl UISelfSet<#dsp_name,FaustFloat> for #enum_name {
            fn set(&self, dsp: &mut #dsp_name) {
                match self {
                    #(#matches_set ),*
                }
            }
            fn get(&self) -> FaustFloat {
                match self {
                    #(#matches_get ),*
                }
            }
        }
        impl UISet<#dsp_name,FaustFloat> for #enum_name_discriminant {
            fn set(&self, dsp: &mut #dsp_name, value: FaustFloat) {
                match self {
                    #(#matches_discriminant ),*
                }
            }
        }
        impl #enum_name_discriminant {
            pub fn value(&self, value: FaustFloat) -> #enum_name {
                match self {
                    #(#matches_value ),*
                }
            }
        }
    }
}

fn create_passive_impl(infos: &[&ParamInfo], dsp_name: &Ident) -> TokenStream {
    let enum_name: Ident = enum_passive_value_ident();

    let enum_name_discriminant = enum_passive_discriminants_ident();
    let dsp_name = format_ident!("{dsp_name}");
    let matches_dsp_value: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            let varname = format_ident!("{}", param_info.varname);
            quote! { #enum_name_discriminant::#shortname => dsp.#varname}
        })
        .collect();

    let matches_enum: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            let varname = format_ident!("{}", param_info.varname);
            quote! { #enum_name_discriminant::#shortname => #enum_name::#shortname(dsp.#varname)}
        })
        .collect();
    let matches_value: Vec<TokenStream> = infos
        .iter()
        .map(|param_info| {
            let shortname = format_ident!("{}", param_info.shortname);
            quote! { #enum_name_discriminant::#shortname => #enum_name::#shortname(value)}
        })
        .collect();

    quote! {
        impl UIGet<#dsp_name> for #enum_name_discriminant {
            type E = #enum_name;
            type F = FaustFloat;
            fn get_value(&self, dsp: & #dsp_name) -> Self::F {
                match self {
                #(#matches_dsp_value ),*
                }
            }
            fn get_enum(&self, dsp: & #dsp_name) -> Self::E {
                match self {
                #(#matches_enum ),*
                }
            }
        }
        impl #enum_name_discriminant {
            pub fn value(&self, value: FaustFloat) -> #enum_name {
                match self {
                    #(#matches_value ),*
                }
            }
        }
    }
}

fn create_from_paraminfo(v: &[ParamInfo], dsp_name: &Ident) -> TokenStream {
    let active: Vec<&ParamInfo> = v.iter().filter(|i| i.is_active).collect();
    let passive: Vec<&ParamInfo> = v.iter().filter(|i| !i.is_active).collect();
    let (active_enum, active_impl) = if active.is_empty() {
        (TokenStream::new(), TokenStream::new())
    } else {
        (
            create_qualified_enum(&active, true),
            create_active_impl(&active, dsp_name),
        )
    };
    let (passive_enum, passive_impl) = if passive.is_empty() {
        (TokenStream::new(), TokenStream::new())
    } else {
        (
            create_qualified_enum(&passive, false),
            create_passive_impl(&passive, dsp_name),
        )
    };
    quote::quote! {
        use strum::{Display,EnumIter,EnumCount,EnumDiscriminants,IntoStaticStr,VariantArray,VariantNames};

        #active_enum
        #active_impl
        #passive_enum
        #passive_impl
    }
}

#[must_use]
pub fn create(dsp_json: &FaustJson, dsp_name: &Ident) -> TokenStream {
    let param_info = dsp_json.get_param_info();
    create_from_paraminfo(&param_info, dsp_name)
}
