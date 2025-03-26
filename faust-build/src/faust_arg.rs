use crate::faust_utils;
use std::{ffi::OsStr, fs, path::PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug, Clone)]
pub enum FaustArg {
    Arg(String),
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
    OutPath(PathBuf),
    //   -e        --export-dsp                  export expanded DSP (with all included libraries).
    //   -uim      --user-interface-macros       add user interface macro definitions to the output code.
    //   -xml                                    generate an XML description file.
    Xml(),
    //   -json                                   generate a JSON description file.
    Json(),
    //   -O <dir>  --output-dir <dir>            specify the relative directory of the generated output code and of additional generated files (SVG, XML...).
    ExtraOutputDir(PathBuf),

    // Code generation options:
    // ---------------------------------------
    //   -lang <lang> --language                 select output language,
    Lang(String),
    //                                           'lang' should be c, cpp (default), cmajor, codebox, csharp, dlang, fir, interp, java, jax, jsfx, julia, llvm, ocpp, rust, vhdl or wast/wasm.
    //   -single     --single-precision-floats   use single precision floats for internal computations (default).
    Single,
    //   -double     --double-precision-floats   use double precision floats for internal computations.
    Double,
    //   -quad       --quad-precision-floats     use quad precision floats for internal computations.
    //   -fx         --fixed-point               use fixed-point for internal computations.
    //   -fx-size    --fixed-point-size          fixed-point number total size in bits (-1 is used to generate a unique fixpoint_t type).
    //   -es 1|0     --enable-semantics 1|0      use enable semantics when 1 (default), and simple multiplication otherwise.
    //   -lcc        --local-causality-check     check causality also at local level.
    //   -light      --light-mode                do not generate the entire DSP API.
    //   -clang      --clang                     when compiled with clang/clang++, adds specific #pragma for auto-vectorization.
    //   -nvi        --no-virtual                when compiled with the C++ backend, does not add the 'virtual' keyword.
    //   -fp         --full-parentheses          always add parentheses around binops.
    //   -cir        --check-integer-range       check float to integer range conversion.
    //   -exp10      --generate-exp10            pow(10,x) replaced by possibly faster exp10(x).
    //   -os         --one-sample                generate one sample computation.
    OneSample,
    //   -ec         --external-control          separated 'control' and 'compute' functions.
    ExternalControl,
    //   -it         --inline-table              inline rdtable/rwtable code in the main class.
    //   -cm         --compute-mix               mix in outputs buffers.
    ComputeMix,
    //   -ct         --check-table               check rtable/rwtable index range and generate safe access code [0/1: 1 by default].
    //   -cn <name>  --class-name <name>         specify the name of the dsp class to be used instead of mydsp.
    StructName(String),
    //   -scn <name> --super-class-name <name>   specify the name of the super class to be used instead of dsp.
    //   -pn <name>  --process-name <name>       specify the name of the dsp entry-point instead of process.
    ProcessName(String),
    //   -mcd <n>    --max-copy-delay <n>        use a copy delay up to max delay <n> and a dense delay above (ocpp only) or a ring buffer (defaut 16 samples).
    //   -mdd <n>    --max-dense-delay <n>       use a dense delay up to max delay <n> (if enough density) and a ring buffer delay above (ocpp only, default 1024).
    //   -mdy <n>    --min-density <n>           minimal density (100*number of delays/max delay) to use a dense delays (ocpp only, default 33).
    //   -dlt <n>    --delay-line-threshold <n>  use a mask-based ring buffer delays up to max delay <n> and a select based ring buffers above (default INT_MAX samples).
    //   -mem        --memory-manager            allocations done using a custom memory manager.
    //   -mem1       --memory-manager1           allocations done using a custom memory manager, using the iControl/fControl and iZone/fZone model.
    //   -mem2       --memory-manager2           use iControl/fControl, iZone/fZone model and no explicit memory manager.
    //   -mem3       --memory-manager3           use iControl/fControl, iZone/fZone model and no explicit memory manager with access as function parameters.
    //   -ftz <n>    --flush-to-zero <n>         code added to recursive signals [0:no (default), 1:fabs based, 2:mask based (fastest)].
    //   -rui        --range-ui                  whether to generate code to constraint vslider/hslider/nentry values in [min..max] range.
    //   -fui        --freeze-ui                 whether to freeze vslider/hslider/nentry to a given value (init value by default).
    //   -inj <f>    --inject <f>                inject source file <f> into architecture file instead of compiling a dsp file.
    //   -scal       --scalar                    generate non-vectorized code (default).
    //   -inpl       --in-place                  generates code working when input and output buffers are the same (scalar mode only).
    InPlace,
    //   -vec        --vectorize                 generate easier to vectorize code.
    //   -vs <n>     --vec-size <n>              size of the vector (default 32 samples).
    //   -lv <n>     --loop-variant <n>          [0:fastest, fixed vector size and a remaining loop (default), 1:simple, variable vector size, 2:fixed, fixed vector size].
    //   -omp        --openmp                    generate OpenMP pragmas, activates --vectorize option.
    //   -pl         --par-loop                  generate parallel loops in --openmp mode.
    //   -sch        --scheduler                 generate tasks and use a Work Stealing scheduler, activates --vectorize option.
    //   -ocl        --opencl                    generate tasks with OpenCL (experimental).
    //   -cuda       --cuda                      generate tasks with CUDA (experimental).
    //   -dfs        --deep-first-scheduling     schedule vector loops in deep first order.
    //   -g          --group-tasks               group single-threaded sequential tasks together when -omp or -sch is used.
    //   -fun        --fun-tasks                 separate tasks code as separated functions (in -vec, -sch, or -omp mode).
    //   -fm <file>  --fast-math <file>          use optimized versions of mathematical functions implemented in <file>, use 'faust/dsp/fastmath.cpp' when file is 'def', assume functions are defined in the architecture file when file is 'arch'.
    //   -mapp       --math-approximation        simpler/faster versions of 'floor/ceil/fmod/remainder' functions.
    //   -noreprc    --no-reprc                  (Rust only) Don't force dsp struct layout to follow C ABI.
    NoReprC,
    //   -ns <name>  --namespace <name>          generate C++ or D code in a namespace <name>.
    //   -vhdl-trace    --vhdl-trace             activate trace.
    //   -vhdl-float    --vhdl-float             uses IEEE-754 format for samples instead of fixed point.
    //   -vhdl-components <file> --vhdl-components <file>    path to a file describing custom components for the VHDL backend.
    //   -fpga-mem <n>  --fpga-mem <n>           FPGA block ram max size, used in -mem1/-mem2 mode.
    //   -wi <n>     --widening-iterations <n>   number of iterations before widening in signal bounding.
    //   -ni <n>     --narrowing-iterations <n>  number of iterations before stopping narrowing in signal bounding.

    // Block diagram options:
    // ---------------------------------------
    //   -ps        --postscript                 print block-diagram to a postscript file.
    //   -svg       --svg                        print block-diagram to a svg file.
    //   -sd        --simplify-diagrams          try to further simplify diagrams before drawing.
    //   -drf       --draw-route-frame           draw route frames instead of simple cables.
    //   -f <n>     --fold <n>                   threshold to activate folding mode during block-diagram generation (default 25 elements).
    //   -fc <n>    --fold-complexity <n>        complexity threshold to fold an expression in folding mode (default 2).
    //   -mns <n>   --max-name-size <n>          threshold during block-diagram generation (default 40 char).
    //   -sn        --simple-names               use simple names (without arguments) during block-diagram generation.
    //   -blur      --shadow-blur                add a shadow blur to SVG boxes.
    //   -sc        --scaled-svg                 automatic scalable SVG.

    // Math doc options:
    // ---------------------------------------
    //   -mdoc       --mathdoc                   print math documentation of the Faust program in LaTeX format in a -mdoc folder.
    //   -mdlang <l> --mathdoc-lang <l>          if translation file exists (<l> = en, fr, ...).
    //   -stripmdoc  --strip-mdoc-tags           strip mdoc tags when printing Faust -mdoc listings.

    // Debug options:
    // ---------------------------------------
    //   -d          --details                   print compilation details.
    //   -time       --compilation-time          display compilation phases timing information.
    //   -flist      --file-list                 print file list (including libraries) used to eval process.
    //   -tg         --task-graph                print the internal task graph in dot format.
    //   -sg         --signal-graph              print the internal signal graph in dot format.
    //   -norm       --normalized-form           print signals in normalized form and exit.
    //   -me         --math-exceptions           check / for 0 as denominator and remainder, fmod, sqrt, log10, log, acos, asin functions domain.
    //   -sts        --strict-select             generate strict code for 'selectX' even for stateless branches (both are computed).
    //   -wall       --warning-all               print all warnings.
    DebugWarnings,
    //   -t <sec>    --timeout <sec>             abort compilation after <sec> seconds (default 120).
    Timeout(String),
    // Information options:
    // ---------------------------------------
    //   -h          --help                      print this help message.
    //   -v          --version                   print version information and embedded backends list.
    //   -libdir     --libdir                    print directory containing the Faust libraries.
    //   -includedir --includedir                print directory containing the Faust headers.
    //   -archdir    --archdir                   print directory containing the Faust architectures.
    //   -dspdir     --dspdir                    print directory containing the Faust dsp libraries.
    //   -pathslist  --pathslist                 print the architectures and dsp library paths.

    // Example:
    // ---------------------------------------
    // faust -a jack-gtk.cpp -o myfx.cpp myfx.dsp
    DspPath(PathBuf),
}

impl FaustArg {
    #[must_use]
    pub fn default_lang() -> Self {
        Self::Lang("rust".to_owned())
    }

    #[must_use]
    pub fn default_timeout() -> Self {
        Self::Timeout("0".to_owned())
    }

    #[must_use]
    pub fn default_arch_file() -> Self {
        let default_template = NamedTempFile::new().expect("failed creating temporary file");
        let default_template_path = default_template.path().to_path_buf();
        let default_template_code = include_str!("../faust-template.rs");

        fs::write(&default_template_path, default_template_code)
            .expect("failed writing temporary architecture file");
        Self::ArchFile(default_template_path)
    }

    #[must_use]
    pub fn json_path(&self) -> PathBuf {
        let Self::DspPath(dsp_path) = self else {
            panic!("json_path can only be got from FaustArg::DspFile enum variant")
        };
        faust_utils::json_path_from_dsp_path(dsp_path)
    }

    #[must_use]
    pub fn xml_path(&self) -> PathBuf {
        let Self::DspPath(dsp_path) = self else {
            panic!("xml_path can only be got from FaustArg::DspFile enum variant")
        };
        faust_utils::xml_path_from_dsp_path(dsp_path)
    }
}

impl std::convert::From<String> for FaustArg {
    fn from(val: String) -> Self {
        Self::Arg(val)
    }
}

/// Trait to transform a `FaustArg` or a Vector o`FaustArgs`gs into a Vector`OsStr`sStr references.
///
/// `FaustArgs` cannot simply be tranlated into an &`OsStr` because one enum variant might produce two command arguments
pub trait FaustArgs2CommandArgs {
    fn to_command_args(&self) -> Vec<&OsStr>;
}

impl FaustArgs2CommandArgs for FaustArg {
    fn to_command_args(&self) -> Vec<&OsStr> {
        match self {
            Self::Arg(arg) => vec![arg.as_ref()],
            Self::ArchFile(path_buf) => vec!["-a".as_ref() as &OsStr, path_buf.as_ref()],
            Self::ImportDir(path_buf) => vec!["-I".as_ref() as &OsStr, path_buf.as_ref()],
            Self::OutPath(path_buf) => vec!["-o".as_ref() as &OsStr, path_buf.as_ref()],
            Self::Xml() => vec!["-xml".as_ref() as &OsStr],
            Self::Json() => vec!["-json".as_ref() as &OsStr],
            Self::ExtraOutputDir(path_buf) => vec!["-O".as_ref() as &OsStr, path_buf.as_ref()],
            Self::Lang(name) => vec!["-lang".as_ref() as &OsStr, name.as_ref()],
            Self::Single => vec!["-single".as_ref() as &OsStr],
            Self::Double => vec!["-double".as_ref() as &OsStr],
            Self::OneSample => vec!["-os".as_ref() as &OsStr],
            Self::ExternalControl => vec!["-ec".as_ref() as &OsStr],
            Self::ComputeMix => vec!["-cm".as_ref() as &OsStr],
            Self::StructName(name) => vec!["-cn".as_ref() as &OsStr, name.as_ref()],
            Self::ProcessName(name) => vec!["-pn".as_ref() as &OsStr, name.as_ref()],
            Self::InPlace => vec!["-inpl".as_ref() as &OsStr],
            Self::NoReprC => vec!["-noreprc".as_ref() as &OsStr],
            Self::DebugWarnings => vec!["-wall".as_ref() as &OsStr],
            Self::Timeout(t) => {
                vec!["-t".as_ref() as &OsStr, t.as_ref()]
            }
            Self::DspPath(path_buf) => vec![path_buf.as_ref()],
        }
    }
}

impl FaustArgs2CommandArgs for Vec<FaustArg> {
    fn to_command_args(&self) -> Vec<&OsStr> {
        self.iter().flat_map(|fa| fa.to_command_args()).collect()
    }
}
