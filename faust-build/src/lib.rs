#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::or_fun_call)]

use builder::FaustBuilder;
use proc_macro2::TokenStream;
use std::ffi::OsStr;

pub mod builder;
pub mod code_option;
pub mod compile_option;
pub mod macro_lib;
pub mod option_map;

pub trait ArchitectureInterface {
    fn wrap(&self, builder: &FaustBuilder, dsp_code: TokenStream) -> TokenStream;
}

/// Trait to transform a Vector o`FaustArgs`gs into a Vector`OsStr`sStr references.
///
/// `FaustArgs` cannot simply be tranlated into an &`OsStr` because one enum variant might produce two command arguments
pub trait FaustArgsToCommandArgs<'a> {
    fn to_command_args(self) -> Vec<&'a OsStr>;
}

pub trait FaustArgsToCommandArgsRef<'a> {
    fn to_command_args(&'a self) -> Vec<&'a OsStr>;
}

impl<'a, T, V> FaustArgsToCommandArgs<'a> for T
where
    T: IntoIterator<Item = &'a V>,
    V: FaustArgsToCommandArgsRef<'a> + 'a,
{
    fn to_command_args(self) -> Vec<&'a OsStr> {
        let i = self.into_iter();
        let m = i.flat_map(FaustArgsToCommandArgsRef::to_command_args);
        m.collect()
    }
}
