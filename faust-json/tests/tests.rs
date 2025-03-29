use deserialize::FaustJson;
use faust_json::*;

use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

fn parse_file(p: PathBuf) {
    dbg!(&p);
    let file = File::open(p).expect("Failed to open file");
    let reader = BufReader::new(file);
    let result: Result<FaustJson, _> = serde_json::from_reader(reader);
    match &result {
        Ok(_f) => {
            //   dbg!(f);
        }
        Err(err) => {
            println!("{}", err);
        }
    };

    assert!(result.is_ok());
}

#[test]
fn parse_all_impulse_test_dsps() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = Path::new(&dir);
    let xml_path: &Path =
        &dir.join("../../../faust4rust/faust-test/faust/tests/impulse-tests/dsp/");
    let entries = fs::read_dir(xml_path).unwrap();
    entries.filter_map(|entry| entry.ok()).for_each(|x| {
        let p = x.path();
        if let Some(e) = p.extension() {
            if e == "json" {
                parse_file(p);
            };
        };
    });
}

#[test]
fn hand_written() {
    let f = r##"
{
	"name": "volumecontrol",
	"filename": "volume.dsp",
	"version": "2.76.0",
	"compile_options": "-lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0",
	"library_list": [],
	"include_pathnames": [],
	"size": 48,
	"inputs": 2,
	"outputs": 2,
	"meta": [
		{ "author": "Franz Heinzmann" },
		{ "basics.lib/name": "Faust Basic Element Library" },
		{ "basics.lib/tabulateNd": "Copyright (C) 2023 Bart Brouns <bart@magnetophon.nl>" },
		{ "basics.lib/version": "1.19.1" },
		{ "compile_options": "-lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0" },
		{ "filename": "volume.dsp" },
		{ "license": "BSD" },
		{ "maths.lib/author": "GRAME" },
		{ "maths.lib/copyright": "GRAME" },
		{ "maths.lib/license": "LGPL with exception" },
		{ "maths.lib/name": "Faust Math Library" },
		{ "maths.lib/version": "2.8.0" },
		{ "name": "volumecontrol" },
		{ "options": "[osc:on]" },
		{ "platform.lib/name": "Generic Platform Library" },
		{ "platform.lib/version": "1.3.0" },
		{ "signals.lib/name": "Faust Signal Routing Library" },
		{ "signals.lib/version": "1.6.0" },
		{ "version": "1.0" }
	],
	"ui": [
		{
			"type": "vgroup",
			"label": "volumecontrol",
			"items": [ ] },
				{
					"type": "vbargraph",
					"label": "level",
					"shortname": "level",
					"address": "/volumecontrol/level",
					"meta": [
						{ "2": "" },
						{ "style": "dB" },
						{ "unit": "dB" }
					],
					"min": -60,
					"max": 5
				},
				{
					"type": "vslider",
					"label": "volume",
					"shortname": "volume",
					"address": "/volumecontrol/volume",
					"init": 0,
					"min": -70,
					"max": 4,
					"step": 0.1
				}
	]
}"##;
    let result: Result<FaustJson, _> = serde_json::from_str(f);
    match &result {
        Ok(f) => {
            dbg!(f);
        }
        Err(err) => {
            println!("{}", err);
        }
    };

    assert!(result.is_ok());
}

#[test]
fn hand_written_broken() {
    let f = r##"
{
	"name": "volumecontrol",
	"filename": "volume.dsp",
	"version": "2.76.0",
	"compile_options": "-lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0",
	"library_list": [],
	"include_pathnames": [],
	"size": 48,
	"inputs": 2,
	"outputs": 2,
	"meta": [
		{ "author": "Franz Heinzmann" },
		{ "basics.lib/name": "Faust Basic Element Library" },
		{ "basics.lib/tabulateNd": "Copyright (C) 2023 Bart Brouns <bart@magnetophon.nl>" },
		{ "basics.lib/version": "1.19.1" },
		{ "compile_options": "-lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0" },
		{ "filename": "volume.dsp" },
		{ "license": "BSD" },
		{ "maths.lib/author": "GRAME" },
		{ "maths.lib/copyright": "GRAME" },
		{ "maths.lib/license": "LGPL with exception" },
		{ "maths.lib/name": "Faust Math Library" },
		{ "maths.lib/version": "2.8.0" },
		{ "name": "volumecontrol" },
		{ "options": "[osc:on]" },
		{ "platform.lib/name": "Generic Platform Library" },
		{ "platform.lib/version": "1.3.0" },
		{ "signals.lib/name": "Faust Signal Routing Library" },
		{ "signals.lib/version": "1.6.0" },
		{ "version": "1.0" }
	],
	"ui": [
		{
			"type": "vgroup",
			"label": "volumecontrol",
			"items": [
				{
					"type": "vbargraph",
					"label": "level",
					"Bla": "blub",
					"shortname": "level",
					"address": "/volumecontrol/level",
					"meta": [
						{ "2": "" },
						{ "style": "dB" },
						{ "unit": "dB" }
					],
					"min": -60,
					"max": 5
				},
				{
					"type": "vslider",
					"label": "volume",
					"shortname": "volume",
					"address": "/volumecontrol/volume",
					"init": 0,
					"min": -70,
					"max": 4,
					"step": 0.1
				}
			]
		}
	]
}"##;
    let result: Result<FaustJson, _> = serde_json::from_str(f);
    assert!(result.is_err());

    // match one {
    //     Ok(f) => {
    //         dbg!(f);
    //     }
    //     Err(err) => {
    //         println!("{}", err);
    //     }
    // };

    // let f = &mut serde_json::from_reader(f.as_bytes());
    // let result: Result<Faust, _> = serde_path_to_error::deserialize(f);
    // match &result {
    //     Ok(f) => {
    //         dbg!(f);
    //     }
    //     Err(err) => {
    //         println!("{}", err);
    //         let path = err.path().to_string();
    //         println!("{}", path);
    //     }
    // };
    // assert!(!result.is_ok());
    // result.unwrap();
}
