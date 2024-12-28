use crate::{option_map::CompileOptionMap, FaustArgsToCommandArgsRef};
use std::{
    ffi::OsStr,
    iter::once,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};
use strum::{EnumDiscriminants, EnumIs};
use tempfile::TempPath;

// Custom(String),
#[derive(Debug, Clone, Eq, EnumDiscriminants, EnumIs)]
#[strum_discriminants(derive(Hash))]
pub enum CompileOption {
    // Input options:
    // ---------------------------------------
    //   -a <file>                               wrapper architecture file.
    ArchFile(PathBuf),
    //   -i        --inline-architecture-files   inline architecture files.
    //   -A <dir>  --architecture-dir <dir>      add the directory <dir> to the architecture search path.
    //   -I <dir>  --import-dir <dir>            add the directory <dir> to the libraries search path.
    ImportDir(PathBuf),
    //   -L <file> --library <file>              link with the LLVM module <file>.
    // Output options:
    // ---------------------------------------
    //   -o <file>                               the output file.
    // OutPath(PathBuf),//don't implement to give it special treatment
    //   -e        --export-dsp                  export expanded DSP (with all included libraries).
    //   -uim      --user-interface-macros       add user interface macro definitions to the output code.
    //   -xml                                    generate an XML description file.
    Xml,
    //   -json                                   generate a JSON description file.
    Json,
    //   -O <dir>  --output-dir <dir>            specify the relative directory of the generated output code and of additional generated files (SVG, XML...).
    // ExtraOutputDir(PathBuf),
    // Code generation options:
    // ---------------------------------------
    //   -lang <lang> --language                 select output language,
    Lang(String),
    // ..
    //   -wall       --warning-all               print all warnings.
    DebugWarnings,
    //   -t <sec>    --timeout <sec>             abort compilation after <sec> seconds (default 120).
    Timeout(String),
    DspPath(DspPath),
}

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

impl CompileOption {
    #[must_use]
    pub fn default_lang() -> Self {
        Self::Lang("rust".to_owned())
    }

    #[must_use]
    pub fn default_timeout() -> Self {
        Self::Timeout("0".to_owned())
    }

    pub fn dsp_path(path: impl Into<PathBuf>) -> Self {
        Self::DspPath(DspPath::File(path.into()))
    }

    pub fn temp_dsp_path(path: impl Into<TempPath>) -> Self {
        Self::DspPath(DspPath::Temp(Rc::new(path.into())))
    }

    // #[must_use]
    // pub fn default_arch_file() -> Self {
    //     let default_template = NamedTempFile::new().expect("failed creating temporary file");
    //     let default_template_path = default_template.path().to_path_buf();
    //     let default_template_code = include_str!("../faust-template.rs");

    //     fs::write(&default_template_path, default_template_code)
    //         .expect("failed writing temporary architecture file");
    //     Self::ArchFile(default_template_path)
    // }

    // #[must_use]
    // pub fn json_path(&self) -> PathBuf {
    //     let Self::DspPath(dsp_path) = self else {
    //         panic!("json_path can only be got from FaustArg::DspFile enum variant")
    //     };
    //     faust_utils::json_path_from_dsp_path(dsp_path)
    // }

    // #[must_use]
    // pub fn xml_path(&self) -> PathBuf {
    //     let Self::DspPath(dsp_path) = self else {
    //         panic!("xml_path can only be got from FaustArg::DspFile enum variant")
    //     };
    //     faust_utils::xml_path_from_dsp_path(dsp_path)
    // }

    fn from_tuple(key: &str, val: &str) -> Self {
        Self::from_str_iter(key, &mut once(&val))
    }
    pub fn arg_map_from_str_iter(
        iteratable: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> CompileOptionMap {
        let mut r = CompileOptionMap::new();
        let mut str_iter = iteratable.into_iter();
        while let Some(key) = str_iter.next() {
            let fa = Self::from_str_iter(key.as_ref(), &mut str_iter);
            r.insert(fa);
        }
        r
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::match_same_arms)]
    pub fn from_str_iter(key: &str, str_iter: &mut impl Iterator<Item = impl AsRef<str>>) -> Self {
        let impl_msg = format!("FaustArg Key not implemented {key}");
        let unknown_msg = format!("FaustArg Key not found {key}");

        let fa = match key {
            // Input options:
            // ---------------------------------------
            "-a" => Self::ArchFile(
                str_iter
                    .next()
                    .unwrap_or_else(|| panic!("Missing Argument after {}", key))
                    .as_ref()
                    .into(),
            ), //wrapper architecture file.
            "-i" | "--inline-architecture-files" => panic!("{}", impl_msg), //inline architecture files.
            "-A" | "--architecture-dir" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //add the directory <dir> to the architecture search path.
            "-I" | "--import-dir" => Self::ImportDir(
                str_iter
                    .next()
                    .unwrap_or_else(|| panic!("Missing Argument after {}", key))
                    .as_ref()
                    .into(),
            ), //add the directory <dir> to the libraries search path.
            "-L" | "--library" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //link with the LLVM module <file>.

            // Output options:
            // ---------------------------------------
            "-o" => {
                panic!("{}", impl_msg)
            }
            //  Self::OutPath(
            //     str_iter
            //         .next()
            //         .unwrap_or_else(|| panic!("Missing Argument after {}", key))
            //         .as_ref()
            //         .into(),
            // ), //the output file.
            "-e" | "--export-dsp" => panic!("{}", impl_msg), //export expanded DSP (with all included libraries).
            "-uim" | "--user-interface-macros" => panic!("{}", impl_msg), //add user interface macro definitions to the output code.
            "-xml" => Self::Xml, //generate an XML description file.

            "-json" => Self::Json, //generate a JSON description file.

            "-O" | "--output-dir" => {
                panic!("{}", impl_msg)
            }
            // Self::ExtraOutputDir(
            //     str_iter
            //         .next()
            //         .unwrap_or_else(|| panic!("Missing Argument after {}", key))
            //         .as_ref()
            //         .into(),
            // ), //specify the relative directory of the generated output code and of additional generated files (SVG, XML...).

            // Block diagram options:
            // ---------------------------------------
            "-ps" | "--postscript" => panic!("{}", impl_msg), //print block-diagram to a postscript file.
            "-svg" | "--svg" => panic!("{}", impl_msg),       //print block-diagram to a svg file.
            "-sd" | "--simplify-diagrams" => panic!("{}", impl_msg), //try to further simplify diagrams before drawing.
            "-drf" | "--draw-route-frame" => panic!("{}", impl_msg), //draw route frames instead of simple cables.
            "-f" | "--fold" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //threshold to activate folding mode during block-diagram generation (default 25 elements).
            "-fc" | "--fold-complexity" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //complexity threshold to fold an expression in folding mode (default 2).
            "-mns" | "--max-name-size" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //threshold during block-diagram generation (default 40 char).
            "-sn" | "--simple-names" => panic!("{}", impl_msg), //use simple names (without arguments) during block-diagram generation.
            "-blur" | "--shadow-blur" => panic!("{}", impl_msg), //add a shadow blur to SVG boxes.
            "-sc" | "--scaled-svg" => panic!("{}", impl_msg),   //automatic scalable SVG.

            // Math doc options:
            // ---------------------------------------
            "-mdoc" | "--mathdoc" => panic!("{}", impl_msg), //print math documentation of the Faust program in LaTeX format in a -mdoc folder.
            "-mdlang" | "--mathdoc-lang" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //if translation file exists (<l> = en, fr, ...).
            "-stripmdoc" | "--strip-mdoc-tags" => panic!("{}", impl_msg), //strip mdoc tags when printing Faust -mdoc listings.

            // Debug options:
            // ---------------------------------------
            "-d" | "--details" => panic!("{}", impl_msg), //print compilation details.
            "-time" | "--compilation-time" => panic!("{}", impl_msg), //display compilation phases timing information.
            "-flist" | "--file-list" => panic!("{}", impl_msg), //print file list (including libraries) used to eval process.
            "-tg" | "--task-graph" => panic!("{}", impl_msg), //print the internal task graph in dot format.
            "-sg" | "--signal-graph" => panic!("{}", impl_msg), //print the internal signal graph in dot format.
            "-norm" | "--normalized-form" => panic!("{}", impl_msg), //print signals in normalized form and exit.
            "-me" | "--math-exceptions" => panic!("{}", impl_msg), //check / for 0 as denominator and remainder, fmod, sqrt, log10, log, acos, asin functions domain.
            "-sts" | "--strict-select" => panic!("{}", impl_msg), //generate strict code for 'selectX' even for stateless branches (both are computed).
            "-wall" | "--warning-all" => Self::DebugWarnings,     //print all warnings.

            "-t" | "--timeout" => Self::Timeout(
                str_iter
                    .next()
                    .unwrap_or_else(|| panic!("Missing Argument after {}", key))
                    .as_ref()
                    .into(),
            ), //abort compilation after <sec> seconds (default 120).

            // Information options:
            // ---------------------------------------
            "-h" | "--help" => panic!("{}", impl_msg), //print this help message.
            "-v" | "--version" => panic!("{}", impl_msg), //print version information and embedded backends list.
            "-libdir" | "--libdir" => panic!("{}", impl_msg), //print directory containing the Faust libraries.
            "-includedir --includedir" => panic!("{}", impl_msg), //print directory containing the Faust headers.
            "-archdir" | "--archdir" => panic!("{}", impl_msg), //print directory containing the Faust architectures.
            "-dspdir" | "--dspdir" => panic!("{}", impl_msg), //print directory containing the Faust dsp libraries.
            "-pathslist" | "--pathslist" => panic!("{}", impl_msg), //print the architectures and dsp library paths.

            // Example:
            // ---------------------------------------
            // faust -a jack-gtk.cpp -o myfx.cpp myfx.dsp
            _ => panic!("{}", unknown_msg),
        };
        fa
    }
}

impl PartialEq for CompileOption {
    fn eq(&self, other: &Self) -> bool {
        let a: CompileOptionDiscriminants = self.into();
        let b: CompileOptionDiscriminants = other.into();
        a == b
    }
}

impl std::convert::From<(&str, &str)> for CompileOption {
    fn from(val: (&str, &str)) -> Self {
        Self::from_tuple(val.0, val.1)
    }
}

impl std::convert::From<[&str; 2]> for CompileOption {
    fn from(val: [&str; 2]) -> Self {
        Self::from_tuple(val[0], val[1])
    }
}

impl<'a> FaustArgsToCommandArgsRef<'a> for CompileOption {
    fn to_command_args(&'a self) -> Vec<&'a OsStr> {
        match self {
            // Self::Custom(arg) => vec![arg.as_ref()],
            Self::ArchFile(path_buf) => vec!["-a".as_ref() as &OsStr, path_buf.as_ref()],
            Self::ImportDir(path_buf) => vec!["-I".as_ref() as &OsStr, path_buf.as_ref()],
            // Self::OutPath(path_buf) => vec!["-o".as_ref() as &OsStr, path_buf.as_ref()],
            Self::Xml => vec!["-xml".as_ref() as &OsStr],
            Self::Json => vec!["-json".as_ref() as &OsStr],
            // Self::ExtraOutputDir(path_buf) => vec!["-O".as_ref() as &OsStr, path_buf.as_ref()],
            Self::Lang(name) => vec!["-lang".as_ref() as &OsStr, name.as_ref()],
            Self::DebugWarnings => vec!["-wall".as_ref() as &OsStr],
            Self::Timeout(t) => {
                vec!["-t".as_ref() as &OsStr, t.as_ref()]
            }
            Self::DspPath(path_buf) => {
                let vec = match path_buf {
                    DspPath::File(path_buf) => path_buf.as_os_str(),
                    DspPath::Temp(rc) => rc.as_os_str(),
                };
                vec![vec]
            }
        }
    }
}
