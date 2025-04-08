use crate::{architecture::Architecture, dsp_path::DspPath};
use core::panic;
use std::{ffi::OsStr, path::PathBuf};

// the best thing would be a serde_command_args serializer
pub struct CompileOptions {
    // Input options:
    // ---------------------------------------
    //   -a <file>                               wrapper architecture file.
    pub architecture: Architecture,
    //   -i        --inline-architecture-files   inline architecture files.
    //   -A <dir>  --architecture-dir <dir>      add the directory <dir> to the architecture search path.
    //   -I <dir>  --import-dir <dir>            add the directory <dir> to the libraries search path.
    pub import_dir: Option<PathBuf>,
    //   -L <file> --library <file>              link with the LLVM module <file>.
    // Output options:
    // ---------------------------------------
    //   -o <file>                               the output file.
    //   -e        --export-dsp                  export expanded DSP (with all included libraries).
    //   -uim      --user-interface-macros       add user interface macro definitions to the output code.
    //   -xml                                    generate an XML description file.
    pub xml: bool,
    //   -json                                   generate a JSON description file.
    pub json: bool,
    //   -O <dir>  --output-dir <dir>            specify the relative directory of the generated output code and of additional generated files (SVG, XML...).
    // ExtraOutputDir(PathBuf),
    // Code generation options:
    // ---------------------------------------
    //   -lang <lang> --language                 select output language,
    pub lang: String,
    // ..
    //   -wall       --warning-all               print all warnings.
    pub debug_warnings: bool,
    //   -t <sec>    --timeout <sec>             abort compilation after <sec> seconds (default 120).
    pub timeout: Option<String>,
    pub dsp_path: Option<DspPath>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            architecture: Architecture::None,
            import_dir: None,
            xml: false,
            json: false,
            lang: "rust".to_owned(),
            debug_warnings: true,
            timeout: None,
            dsp_path: None,
        }
    }
}

impl CompileOptions {
    #[must_use]
    pub fn to_command_args(&self) -> Vec<&OsStr> {
        let mut r = Vec::<&OsStr>::new();
        if let Some(arch_file) = self.architecture.get_file_path() {
            r.push("-a".as_ref());
            r.push(arch_file.as_ref());
        }
        if let Some(import_dir) = &self.import_dir {
            r.push("-I".as_ref());
            r.push(import_dir.as_ref());
        }
        if self.xml {
            r.push("-xml".as_ref());
        }
        if self.json {
            r.push("-json".as_ref());
        }
        r.push("-lang".as_ref());
        r.push(self.lang.as_ref());

        if self.debug_warnings {
            r.push("-wall".as_ref());
        }
        // 120 is default
        if let Some(timeout) = &self.timeout {
            r.push("-t".as_ref());
            r.push(timeout.as_ref());
        }
        if let Some(dsp_path) = &self.dsp_path {
            r.push(dsp_path.as_ref());
        } else {
            panic!("No Path to DSP file provided")
        }
        r
    }
}
