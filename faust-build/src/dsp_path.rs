use std::{
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};
use tempfile::TempPath;

#[derive(Debug, Clone)]
pub enum DspPath {
    File(PathBuf),
    Temp(Rc<TempPath>),
}

impl Deref for DspPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::File(path_buf) => path_buf,
            Self::Temp(rc) => rc,
        }
    }
}

impl PartialEq for DspPath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(l0), Self::File(r0)) => l0 == r0,
            (Self::Temp(_l0), Self::Temp(_r0)) => false,
            _ => false,
        }
    }
}
impl Eq for DspPath {}
