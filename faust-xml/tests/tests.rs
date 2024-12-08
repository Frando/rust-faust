use faust_xml::*;
use quick_xml::de::from_str;

use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

fn parse_file(p: PathBuf) {
    let file = File::open(p).expect("Failed to open file");
    let reader = BufReader::new(file);
    let f = &mut quick_xml::de::Deserializer::from_reader(reader);
    let result: Result<Faust, _> = serde_path_to_error::deserialize(f);
    match &result {
        Ok(_f) => {
            // dbg!(f);
        }
        Err(err) => {
            println!("{}", err);
            let path = err.path().to_string();
            println!("{}", path);
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
        dbg!(&p);
        if let Some(e) = p.extension() {
            if e == "xml" {
                parse_file(p);
            };
        };
    });
}

#[test]
fn hand_written() {
    let f = r##"
<faust>
    <name>VolumeControl</name>
    <author>Franz Heinzmann</author>
    <copyright></copyright>
    <license>BSD</license>
    <version>1.0</version>
    <classname>VolumeControl</classname>
    <inputs>2</inputs>
    <outputs>2</outputs>
    <meta key="basics.lib/name">Faust Basic Element Library</meta>
    <meta key="basics.lib/tabulateNd">Copyright (C) 2023 Bart Brouns &lt;bart@magnetophon.nl&gt;</meta>
    <meta key="basics.lib/version">1.19.1</meta>
    <meta key="compile_options">-a /tmp/.tmpYJHonf -lang rust -ct 1 -cn VolumeControl -es 1 -mcd 16 -mdd 1024 -mdy 33 -uim -double -ftz 0</meta>
    <meta key="filename">volume.dsp</meta>
    <meta key="maths.lib/author">GRAME</meta>
    <meta key="maths.lib/copyright">GRAME</meta>
    <meta key="maths.lib/license">LGPL with exception</meta>
    <meta key="maths.lib/name">Faust Math Library</meta>
    <meta key="maths.lib/version">2.8.0</meta>
    <meta key="options"></meta>
    <meta key="platform.lib/name">Generic Platform Library</meta>
    <meta key="platform.lib/version">1.3.0</meta>
    <meta key="signals.lib/name">Faust Signal Routing Library</meta>
    <meta key="signals.lib/version">1.6.0</meta>
    <ui>
        <activewidgets>
            <count>2</count>
            <widget type="vslider" id="1">
                <label>volume</label>
                <varname>fVslider0</varname>
                <init>0.0</init>
                <min>-7e+01</min>
                <max>4.0</max>
                <step>0.1</step>
                <meta key="1"></meta>
            </widget>
            <widget type="hslider" id="3">
                <label>volume</label>
                <varname>fVslider1</varname>
                <init>0.0</init>
                <min>-7e+01</min>
                <max>4.0</max>
                <step>0.1</step>
                <meta key="2"></meta>
            </widget>
        </activewidgets>

        <passivewidgets>
            <count>1</count>
            <widget type="vbargraph" id="2">
                <label>level</label>
                <varname>fVbargraph0</varname>
                <min>-6e+01</min>
                <max>5.0</max>
                <meta key="2"></meta>
                <meta key="style">dB</meta>
                <meta key="unit">dB</meta>
            </widget>
        </passivewidgets>

        <soundfilewidgets>
            <count>0</count>
        </soundfilewidgets>

        <layout>
            <group type="vgroup">
                <group type="vgroup">
                    <label>left</label>
                    <widgetref id="1" />
                </group>
                <widgetref id="2" />
                <group type="vgroup">
                    <label>right</label>
                    <widgetref id="3" />
                </group>
                <label>0x00</label>
            </group>
        </layout>
        </ui>
</faust>
    "##;
    let result: Result<Faust, _> = from_str(f);
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
<faust>
    <name>VolumeControl</name>
    <author>Franz Heinzmann</author>
    <copyright></copyright>
    <license>BSD</license>
    <version>1.0</version>
    <classname>VolumeControl</classname>
    <inputs>2</inputs>
    <outputs>2</outputs>
    <meta key="basics.lib/name">Faust Basic Element Library</meta>
    <meta key="basics.lib/tabulateNd">Copyright (C) 2023 Bart Brouns &lt;bart@magnetophon.nl&gt;</meta>
    <meta key="basics.lib/version">1.19.1</meta>
    <meta key="compile_options">-a /tmp/.tmpYJHonf -lang rust -ct 1 -cn VolumeControl -es 1 -mcd 16 -mdd 1024 -mdy 33 -uim -double -ftz 0</meta>
    <meta key="filename">volume.dsp</meta>
    <meta key="maths.lib/author">GRAME</meta>
    <meta key="maths.lib/copyright">GRAME</meta>
    <meta key="maths.lib/license">LGPL with exception</meta>
    <meta key="maths.lib/name">Faust Math Library</meta>
    <meta key="maths.lib/version">2.8.0</meta>
    <meta key="options"></meta>
    <meta key="platform.lib/name">Generic Platform Library</meta>
    <meta key="platform.lib/version">1.3.0</meta>
    <meta key="signals.lib/name">Faust Signal Routing Library</meta>
    <meta key="signals.lib/version">1.6.0</meta>
    <ui>
        <activewidgets>
            <count>2</count>
            <widget type="vslider" id="1">
                <label>volume</label>
                <varname>fVslider0</varname>
                <init>0.0</init>
                <min>-7e+01</min>
                <max>4.0</max>
                <step>0.1</step>
                <meta key="1"></meta>
            </widget>
            <widget type="hslider" id="3">
                <label>volume</label>
                <varname>fVslider1</varname>
                <init>0.0</init>
                <min>-7e+01</min>
                <max>4.0</max>
                <step>0.1</step>
                <meta key="2"></meta>
            </widget>
        </activewidgets>

        <passivewidgets>
            <count>1</count>
            <widget type="vbargraph" id="2">
                <label>level</label>
                <varname>fVbargraph0</varname>
                <min>-6e+01</min>
                <max>5.0</max>
                <meta key="2"></meta>
                <meta key="style">dB</meta>
                <meta key="unit">dB</meta>
            </widget>
        </passivewidgets>

        <soundfilewidgets>
            <count>0</count>
        </soundfilewidgets>

        <layout>
            <group type="vgroup">
                <group type="vgroup">
                    <label>left</label>
                    <widgetref id="1" />
                </group>
                <widgetref id="2" />
                <group type="vgroup">
                    <label>right</label>
                    <widgetref id="3" BLALBLA="blub"/>
                </group>
                <label>0x00</label>
            </group>
        </layout>
        </ui>
</faust>
    "##;
    let one: Result<Faust, _> = from_str(f);
    match one {
        Ok(f) => {
            dbg!(f);
        }
        Err(err) => {
            println!("{}", err);
        }
    };

    let f = &mut quick_xml::de::Deserializer::from_reader(f.as_bytes());
    let result: Result<Faust, _> = serde_path_to_error::deserialize(f);
    match &result {
        Ok(f) => {
            dbg!(f);
        }
        Err(err) => {
            println!("{}", err);
            let path = err.path().to_string();
            println!("{}", path);
        }
    };
    assert!(result.is_err());
}
