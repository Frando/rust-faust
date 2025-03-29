pub fn strip_quotes(name: &proc_macro2::TokenTree) -> String {
    name.to_string()
        .strip_prefix('\"')
        .expect("prefix is not \"")
        .strip_suffix('\"')
        .expect("postfix is not \"")
        .to_string()
}

pub fn get_declared_value(key: &str, ts: proc_macro2::TokenStream) -> Option<String> {
    // find the token that declares a key in the dsp file
    let mut ii = ts.into_iter();
    while let Some(n) = ii.next() {
        if n.to_string() == "declare" {
            if let Some(n) = ii.next() {
                if n.to_string() == key {
                    if let Some(value) = ii.next() {
                        if let Some(semicolon) = ii.next() {
                            if semicolon.to_string() == ";" {
                                return Some(strip_quotes(&value));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[must_use]
pub fn get_name_token(ts: proc_macro2::TokenStream) -> String {
    get_declared_value("name", ts)
        .expect("name declaration is not found.\n Expect 'declare name NAMESTRING;' in faust code.")
}

pub fn get_flags_token(ts: proc_macro2::TokenStream) -> Vec<String> {
    get_declared_value("flags", ts).map_or_else(std::vec::Vec::new, |s| {
        s.split_whitespace()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    })
}
