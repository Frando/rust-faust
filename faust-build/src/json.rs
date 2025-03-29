use std::fs;

use faust_json::deserialize::FaustJson;

use crate::{faust_arg::FaustArg, faust_utils, FaustBuilder};

impl FaustBuilder {
    pub fn build_json(&self) -> FaustJson {
        let json = self.build_to_stdout(vec![FaustArg::Json()]);
        serde_json::from_str(&json).expect("Failed parsing json")
    }

    // pub fn build_json_at_file(&self, out: &str) {
    //     let gen_json_path = faust_utils::json_path_from_dsp_path(&self.in_file);
    //     self.build_json();
    //     fs::rename(&gen_json_path, out).unwrap_or_else(|_| {
    //         panic!(
    //             "rename of json file failed from '{:?}' to '{:?}'",
    //             gen_json_path, out
    //         )
    //     });
    // }
}
