#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]

use faust_macro_lib::faust_build;

#[proc_macro]
pub fn faust(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    faust_build(&input.into()).into()
}
