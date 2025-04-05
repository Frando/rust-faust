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
#![allow(clippy::missing_const_for_fn)]

use std::ffi::OsStr;

pub mod architecture;
pub mod builder;
pub mod code_option;
pub mod compile_options;
pub mod dsp_path;
#[cfg(feature = "faust-ui")]
pub mod macro_lib;

/// Trait to transform a Vector o`FaustArgs`gs into a Vector`OsStr`sStr references.
///
/// `FaustArgs` cannot simply be translated into an &`OsStr` because one enum variant might produce two command arguments
pub trait CodeOptionToCommandArgs<'a> {
    fn to_command_args(self) -> Vec<&'a OsStr>;
}

pub trait CodeOptionsToCommandArgsRef<'a> {
    fn to_command_args(&'a self) -> Vec<&'a OsStr>;
}

impl<'a, T, V> CodeOptionToCommandArgs<'a> for T
where
    T: IntoIterator<Item = &'a V>,
    V: CodeOptionsToCommandArgsRef<'a> + 'a,
{
    fn to_command_args(self) -> Vec<&'a OsStr> {
        let i = self.into_iter();
        let m = i.flat_map(CodeOptionsToCommandArgsRef::to_command_args);
        m.collect()
    }
}
