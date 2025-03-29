use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use heck::CamelCase;
use tempfile::NamedTempFile;

#[must_use]
pub fn struct_name_from_dsp_path(dsp_path: &Path) -> String {
    dsp_path
        .file_stem()
        .expect("dsp_path does not end with a filename")
        .to_str()
        .expect("dsp path is not utf8")
        .to_camel_case()
}

#[must_use]
pub fn write_temp_dsp_file(faust_code: &str) -> NamedTempFile {
    let temp_dsp = NamedTempFile::new().expect("failed creating temp dsp file");
    let mut f = BufWriter::new(temp_dsp);
    f.write_all(faust_code.as_bytes())
        .expect("Unable to write to temp dsp file");
    f.into_inner().expect("temp dsp error on flush")
}

#[must_use]
pub fn xml_path_from_dsp_path(dsp_path: &Path) -> PathBuf {
    let gen_xml_fn = dsp_path.to_str().expect("dsp path is not utf8").to_owned() + ".xml";
    PathBuf::from(gen_xml_fn)
}

#[must_use]
pub fn json_path_from_dsp_path(dsp_path: &Path) -> PathBuf {
    let gen_json_fn = dsp_path.to_str().expect("dsp path is not utf8").to_owned() + ".json";
    PathBuf::from(gen_json_fn)
}
