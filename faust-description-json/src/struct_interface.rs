use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    deserialize::{FaustDescriptionJson, LayoutItem},
    enum_interface,
};

#[derive(Clone)]
enum StructInfo {
    GroupInfo {
        label: Ident,
        type_name: Ident,
        items: Vec<StructInfo>,
    },
    UIInfo {
        label: Ident,
        type_name: Ident,
        shortname: Ident,
    },
}

impl StructInfo {
    pub fn active(label: &str, shortname: &str) -> Self {
        Self::UIInfo {
            type_name: enum_interface::alias_active_ident(),
            label: format_ident!("{label}"),
            shortname: format_ident!("{shortname}"),
        }
    }
    pub fn passive(label: &str, shortname: &str) -> Self {
        Self::UIInfo {
            type_name: enum_interface::alias_passive_ident(),
            label: format_ident!("{label}"),
            shortname: format_ident!("{shortname}"),
        }
    }
}

trait GetGroupInfo {
    fn get_ui_structure(&self, parent_type: &Ident) -> StructInfo;
}

impl GetGroupInfo for LayoutItem {
    fn get_ui_structure(&self, parent_type: &Ident) -> StructInfo {
        match self {
            Self::TGroup { label, items, .. } => {
                let type_name = format_ident!("{parent_type}{}", label.to_camel_case());
                let label = format_ident!("{label}");
                StructInfo::GroupInfo {
                    type_name: type_name.clone(),
                    label,
                    items: items
                        .iter()
                        .map(|items| items.get_ui_structure(&type_name))
                        .collect(),
                }
            }
            Self::VGroup { label, items, .. } => {
                let type_name = format_ident!("{parent_type}{}", label.to_camel_case());
                let label = format_ident!("{label}");
                StructInfo::GroupInfo {
                    type_name: type_name.clone(),
                    label,
                    items: items
                        .iter()
                        .map(|items| items.get_ui_structure(&type_name))
                        .collect(),
                }
            }
            Self::HGroup { label, items, .. } => {
                let type_name = format_ident!("{parent_type}{}", label.to_camel_case());
                let label = format_ident!("{label}");
                StructInfo::GroupInfo {
                    type_name: type_name.clone(),
                    label,
                    items: items
                        .iter()
                        .map(|items| items.get_ui_structure(&type_name))
                        .collect(),
                }
            }
            Self::VSlider {
                label, shortname, ..
            }
            | Self::HSlider {
                label, shortname, ..
            }
            | Self::NEntry {
                label, shortname, ..
            }
            | Self::Button {
                label, shortname, ..
            }
            | Self::CheckBox {
                label, shortname, ..
            } => StructInfo::active(label, &shortname.to_camel_case()),
            Self::VBarGraph {
                label, shortname, ..
            }
            | Self::HBarGraph {
                label, shortname, ..
            } => StructInfo::passive(label, &shortname.to_camel_case()),
            Self::Soundfile { address, .. } => StructInfo::active(address, address),
        }
    }
}

fn flat_ui_infos(i: &[StructInfo]) -> Vec<StructInfo> {
    let mut ii = i.to_vec();
    ii.append(
        &mut i
            .iter()
            .flat_map(|i| match i {
                StructInfo::GroupInfo { items, .. } => flat_ui_infos(items.as_slice()),
                StructInfo::UIInfo { .. } => vec![],
            })
            .collect(),
    );
    ii
}

fn create_struct_fields(items: &[StructInfo]) -> Vec<TokenStream> {
    items
        .iter()
        .map(|i: &StructInfo| match i {
            StructInfo::UIInfo {
                label, type_name, ..
            }
            | StructInfo::GroupInfo {
                label, type_name, ..
            } => {
                quote! {
                    pub #label : #type_name
                }
            }
        })
        .collect()
}

fn create_struct_defaults(items: &[StructInfo]) -> Vec<TokenStream> {
    items
        .iter()
        .map(|i: &StructInfo| match i {
            StructInfo::GroupInfo {
                label, type_name, ..
            } => {
                quote! {
                    #label : #type_name::static_ui()
                }
            }
            StructInfo::UIInfo {
                label,
                type_name,
                shortname,
            } => {
                quote! {
                    #label: #type_name::#shortname
                }
            }
        })
        .collect()
}

fn create_structs(si: &StructInfo) -> TokenStream {
    match si {
        StructInfo::GroupInfo {
            items, type_name, ..
        } => {
            let fields: Vec<TokenStream> = create_struct_fields(items);
            let defaults: Vec<TokenStream> = create_struct_defaults(items);
            quote::quote! {
                #[derive(Debug)]
                pub struct #type_name { #(#fields),* }

                impl #type_name {
                    const fn static_ui() -> Self {
                        Self {
                            #(#defaults),*
                        }
                    }
                }

            }
        }
        StructInfo::UIInfo { .. } => {
            quote::quote! {}
        }
    }
}

pub fn create(
    dsp_json: &FaustDescriptionJson,
    ui_static_name: &Ident,
    ui_type: &Ident,
) -> TokenStream {
    let ui_info_tree = vec![StructInfo::GroupInfo {
        type_name: ui_type.clone(),
        label: ui_static_name.clone(),
        items: dsp_json
            .ui
            .iter()
            .map(|items| items.get_ui_structure(ui_type))
            .collect(),
    }];
    // i need to unroll the tree into a list of structs
    let ui_info_list = flat_ui_infos(&ui_info_tree);
    let ui_structs: TokenStream = ui_info_list.iter().map(create_structs).collect();
    let ui_global = quote! {
        pub static #ui_static_name: #ui_type = #ui_type::static_ui();
    };
    quote! {
        #ui_structs
        #ui_global
    }
}
