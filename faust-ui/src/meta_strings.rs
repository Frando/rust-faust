use faust_json::Meta;
use proc_macro2::TokenStream;
use quote::format_ident;
use std::collections::BTreeMap;

pub(crate) fn create(faust_json: &faust_json::FaustJson) -> TokenStream {
    let non_lib = get_non_lib_items(faust_json);
    let libs = lib_map_to_tokenstream(&get_lib_item_map(faust_json));

    quote::quote! {
        pub mod meta{
            #(#non_lib ) *
            pub mod libs{
                #(#libs ) *
            }
        }
    }
}

fn lib_map_to_tokenstream(lib_map: &BTreeMap<&str, Vec<&Meta>>) -> Vec<TokenStream> {
    let libs = lib_map
        .iter()
        .map(|(lib, v)| {
            let lib = format_ident!("{}", lib);
            let liblines = v
                .iter()
                .map(|m| {
                    let key = m
                        .key
                        .split(".lib/")
                        .nth(1)
                        .unwrap_or_else(|| panic!("empty tail in metadata lib key: {}", m.key));
                    let key = format_ident!("{}", key.replace(['.', ':', '/'], "_").to_uppercase());
                    let v = m.value.clone();
                    quote::quote!(pub const #key: &'static str = #v;)
                })
                .collect::<Vec<_>>();
            quote::quote! {
                pub mod #lib{
                    #(#liblines) *
                }
            }
        })
        .collect::<Vec<_>>();
    libs
}

fn get_lib_item_map(faust_json: &faust_json::FaustJson) -> BTreeMap<&str, Vec<&Meta>> {
    let mut lib_map = BTreeMap::<&str, Vec<&Meta>>::new();

    faust_json
        .meta
        .iter()
        .filter(|m| m.key.contains(".lib/"))
        .for_each(|m| {
            let lib = m
                .key
                .split(".lib/")
                .next()
                .unwrap_or_else(|| panic!("empty tail in metadata lib key: {}", m.key));
            lib_map
                .entry(lib)
                .and_modify(|v| v.push(m))
                .or_insert_with(|| vec![m]);
        });
    lib_map
}

fn get_non_lib_items(faust_json: &faust_json::FaustJson) -> Vec<TokenStream> {
    let non_lib = faust_json
        .meta
        .iter()
        .filter(|m| !m.key.contains(".lib/"))
        .map(|m| {
            let k = format_ident!("{}", m.key.replace(['.', ':', '/'], "_").to_uppercase());
            let v = m.value.clone();
            quote::quote!(pub const #k: &'static str = #v;)
        })
        .collect::<Vec<_>>();
    non_lib
}
