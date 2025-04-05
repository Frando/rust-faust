#![allow(clippy::module_name_repetitions)]

use crate::{CodeOptionToCommandArgs, CodeOptionsToCommandArgsRef};
use std::{
    collections::{hash_map::IntoValues, HashMap, HashSet},
    ffi::OsStr,
    iter::FromIterator,
};
use strum::{EnumDiscriminants, EnumIs, EnumString, IntoDiscriminant};

#[derive(Debug, Clone, Default)]
pub struct CodeOptionMap(HashMap<CodeOptionDiscriminants, CodeOption>);
impl CodeOptionMap {
    pub fn insert(&mut self, value: CodeOption) -> Option<CodeOption> {
        self.0.insert(CodeOption::discriminant(&value), value)
    }

    #[must_use]
    pub fn get(&self, key: &CodeOptionDiscriminants) -> Option<&CodeOption> {
        self.0.get(key)
    }

    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub fn to_command_args_merge<'a>(&'a self, other_args: &'a Self) -> Vec<&'a OsStr> {
        let keys: HashSet<&CodeOptionDiscriminants> =
            self.0.keys().chain(other_args.0.keys()).collect();
        let values = keys
            .iter()
            .map(|key| {
                other_args
                    .get(key)
                    .unwrap_or_else(|| self.get(key).expect("cannot fail"))
            })
            .collect::<Vec<_>>();

        CodeOptionToCommandArgs::to_command_args(values)
    }
}

impl Extend<CodeOption> for CodeOptionMap {
    fn extend<T: IntoIterator<Item = CodeOption>>(&mut self, iter: T) {
        for i in iter {
            self.insert(i);
        }
    }
}

impl FromIterator<CodeOption> for CodeOptionMap {
    fn from_iter<T: IntoIterator<Item = CodeOption>>(iter: T) -> Self {
        let mut r = Self(HashMap::new());
        for i in iter {
            r.insert(i);
        }
        r
    }
}

impl IntoIterator for CodeOptionMap {
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_values()
    }
    type Item = CodeOption;

    type IntoIter = IntoValues<CodeOptionDiscriminants, CodeOption>;
}

impl<'a> CodeOptionToCommandArgs<'a> for &'a CodeOptionMap {
    fn to_command_args(self) -> Vec<&'a OsStr> {
        CodeOptionToCommandArgs::to_command_args(self.0.values())
    }
}

#[derive(Debug, Clone, Eq, EnumDiscriminants, EnumIs, EnumString)]
#[strum_discriminants(derive(Hash))]
pub enum CodeOption {
    // Code generation options:
    // ---------------------------------------
    // ...
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
    // DebugWarnings,
    //   -t <sec>    --timeout <sec>             abort compilation after <sec> seconds (default 120).
    // Timeout(String),
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
}
impl CodeOption {
    pub fn arg_map_from_str_iter(
        iteratable: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> CodeOptionMap {
        let mut r = CodeOptionMap::new();
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
            // Code generation options:
            // ---------------------------------------
            //" => panic!("{}", impl_msg),//'lang' should be c, cpp (default), cmajor, codebox, csharp, dlang, fir, interp, java, jax, jsfx, julia, llvm, ocpp, rust, vhdl or wast/wasm.
            "-single" | "--single-precision-floats" => Self::Single, //use single precision floats for internal computations (default).

            "-double" | "--double-precision-floats" => Self::Double, //use double precision floats for internal computations.

            "-quad" | "--quad-precision-floats" => panic!("{}", impl_msg), //use quad precision floats for internal computations.
            "-fx" | "--fixed-point" => panic!("{}", impl_msg), //use fixed-point for internal computations.
            "-fx-size" | "--fixed-point-size" => panic!("{}", impl_msg), //fixed-point number total size in bits (-1 is used to generate a unique fixpoint_t type).
            "-es 1|0" | "--enable-semantics 1|0" => panic!("{}", impl_msg), //use enable semantics when 1 (default), and simple multiplication otherwise.
            "-lcc" | "--local-causality-check" => panic!("{}", impl_msg), //check causality also at local level.
            "-light" | "--light-mode" => panic!("{}", impl_msg), //do not generate the entire DSP API.
            "-clang" | "--clang" => panic!("{}", impl_msg), //when compiled with clang/clang++, adds specific #pragma for auto-vectorization.
            "-nvi" | "--no-virtual" => panic!("{}", impl_msg), //when compiled with the C++ backend, does not add the 'virtual' keyword.
            "-fp" | "--full-parentheses" => panic!("{}", impl_msg), //always add parentheses around binops.
            "-cir" | "--check-integer-range" => panic!("{}", impl_msg), //check float to integer range conversion.
            "-exp10" | "--generate-exp10" => panic!("{}", impl_msg), //pow(10,x) replaced by possibly faster exp10(x).
            "-os" | "--one-sample" => Self::OneSample, //generate one sample computation.

            "-ec" | "--external-control" => Self::ExternalControl, //separated 'control' and 'compute' functions.

            "-it" | "--inline-table" => panic!("{}", impl_msg), //inline rdtable/rwtable code in the main class.
            "-cm" | "--compute-mix" => Self::ComputeMix,        //mix in outputs buffers.

            "-ct" | "--check-table" => panic!("{}", impl_msg), //check rtable/rwtable index range and generate safe access code [0/1: 1 by default].
            "-cn" | "--class-name" => Self::StructName(
                str_iter
                    .next()
                    .unwrap_or_else(|| panic!("Missing Argument after {}", key))
                    .as_ref()
                    .into(),
            ), //specify the name of the dsp class to be used instead of mydsp.

            "-scn" | " --super-class-name" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //specify the name of the super class to be used instead of dsp.
            "-pn" | "--process-name" => Self::ProcessName(
                str_iter
                    .next()
                    .unwrap_or_else(|| panic!("Missing Argument after {}", key))
                    .as_ref()
                    .into(),
            ), //specify the name of the dsp entry-point instead of process.

            "-mcd" | "--max-copy-delay" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //use a copy delay up to max delay <n> and a dense delay above (ocpp only) or a ring buffer (defaut 16 samples).
            "-mdd" | "--max-dense-delay" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //use a dense delay up to max delay <n> (if enough density) and a ring buffer delay above (ocpp only, default 1024).
            "-mdy" | "--min-density" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //minimal density (100*number of delays/max delay) to use a dense delays (ocpp only, default 33).
            "-dlt" | "--delay-line-threshold" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //use a mask-based ring buffer delays up to max delay <n> and a select based ring buffers above (default INT_MAX samples).
            "-mem" | "--memory-manager" => panic!("{}", impl_msg), //allocations done using a custom memory manager.
            "-mem1" | "--memory-manager1" => panic!("{}", impl_msg), //allocations done using a custom memory manager, using the iControl/fControl and iZone/fZone model.
            "-mem2" | "--memory-manager2" => panic!("{}", impl_msg), //use iControl/fControl, iZone/fZone model and no explicit memory manager.
            "-mem3" | "--memory-manager3" => panic!("{}", impl_msg), //use iControl/fControl, iZone/fZone model and no explicit memory manager with access as function parameters.
            "-ftz" | "--flush-to-zero" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //code added to recursive signals [0:no (default), 1:fabs based, 2:mask based (fastest)].
            "-rui" | "--range-ui" => panic!("{}", impl_msg), //whether to generate code to constraint vslider/hslider/nentry values in [min..max] range.
            "-fui" | "--freeze-ui" => panic!("{}", impl_msg), //whether to freeze vslider/hslider/nentry to a given value (init value by default).
            "-inj" | "--inject" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //inject source file <f> into architecture file instead of compiling a dsp file.
            "-scal" | "--scalar" => panic!("{}", impl_msg), //generate non-vectorized code (default).
            "-inpl" | "--in-place" => Self::InPlace, //generates code working when input and output buffers are the same (scalar mode only).

            "-vec" | "--vectorize" => panic!("{}", impl_msg), //generate easier to vectorize code.
            "-vs" | "--vec-size" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //size of the vector (default 32 samples).
            "-lv" | "--loop-variant" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //[0:fastest, fixed vector size and a remaining loop (default), 1:simple, variable vector size, 2:fixed, fixed vector size].
            "-omp" | "--openmp" => panic!("{}", impl_msg), //generate OpenMP pragmas, activates --vectorize option.
            "-pl" | "--par-loop" => panic!("{}", impl_msg), //generate parallel loops in --openmp mode.
            "-sch" | "--scheduler" => panic!("{}", impl_msg), //generate tasks and use a Work Stealing scheduler, activates --vectorize option.
            "-ocl" | "--opencl" => panic!("{}", impl_msg), //generate tasks with OpenCL (experimental).
            "-cuda" | "--cuda" => panic!("{}", impl_msg), //generate tasks with CUDA (experimental).
            "-dfs" | "--deep-first-scheduling" => panic!("{}", impl_msg), //schedule vector loops in deep first order.
            "-g" | "--group-tasks" => panic!("{}", impl_msg), //group single-threaded sequential tasks together when -omp or -sch is used.
            "-fun" | "--fun-tasks" => panic!("{}", impl_msg), //separate tasks code as separated functions (in -vec, -sch, or -omp mode).
            "-fm" | "--fast-math" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //use optimized versions of mathematical functions implemented in <file>, use 'faust/dsp/fastmath.cpp' when file is 'def', assume functions are defined in the architecture file when file is 'arch'.
            "-mapp" | "--math-approximation" => panic!("{}", impl_msg), //simpler/faster versions of 'floor/ceil/fmod/remainder' functions.
            "-noreprc" | "--no-reprc" => Self::NoReprC, //(Rust only) Don't force dsp struct layout to follow C ABI.

            "-ns" | "--namespace" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //generate C++ or D code in a namespace <name>.
            "-vhdl-trace" | "--vhdl-trace" => panic!("{}", impl_msg), //activate trace.
            "-vhdl-float" | "--vhdl-float" => panic!("{}", impl_msg), //uses IEEE-754 format for samples instead of fixed point.
            "-vhdl-components <file> --vhdl-components" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //path to a file describing custom components for the VHDL backend.
            "-fpga-mem" | "--fpga-mem" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //FPGA block ram max size, used in -mem1/-mem2 mode.
            "-wi" | "--widening-iterations" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //number of iterations before widening in signal bounding.
            "-ni" | "--narrowing-iterations" =>
            /*(str_iter.next().unwrap_or_else(||panic!("Missing Argument after {}", key)).into())*/
            {
                panic!("{}", impl_msg)
            } //number of iterations before stopping narrowing in signal bounding.

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

            _ => panic!("{}", unknown_msg),
        };
        fa
    }
}

impl PartialEq for CodeOption {
    fn eq(&self, other: &Self) -> bool {
        let a: CodeOptionDiscriminants = self.into();
        let b: CodeOptionDiscriminants = other.into();
        a == b
    }
}

impl<'a> CodeOptionsToCommandArgsRef<'a> for CodeOption {
    fn to_command_args(&'a self) -> Vec<&'a OsStr> {
        match self {
            Self::Single => vec!["-single".as_ref() as &OsStr],
            Self::Double => vec!["-double".as_ref() as &OsStr],
            Self::OneSample => vec!["-os".as_ref() as &OsStr],
            Self::ExternalControl => vec!["-ec".as_ref() as &OsStr],
            Self::ComputeMix => vec!["-cm".as_ref() as &OsStr],
            Self::StructName(name) => vec!["-cn".as_ref() as &OsStr, name.as_ref()],
            Self::ProcessName(name) => {
                vec!["-pn".as_ref() as &OsStr, name.as_ref()]
            }
            Self::InPlace => vec!["-inpl".as_ref() as &OsStr],
            Self::NoReprC => vec!["-noreprc".as_ref() as &OsStr],
        }
    }
}
