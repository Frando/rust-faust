# rust-faust

A better integration of [FAUST](https://faust.grame.fr/) for [Rust](https://www.rust-lang.org/).

WIP but very open to contributions! Please just open issues or come the [RustAudio Discord](https://rust-audio.discourse.group/) and ping @Frando.

* `faust-build`: Build FAUST dsp files into Rust modules as part of the normal crate compilation
* `faust-types`: Types and traits needed by Rust modules built from FAUST dsp files.
* `faust-state`: Abstractions and data structures to make it easier to work with the trait implementations in Faust modules
* `faust-macro`: A macro to write dsp files within rust files utilizes faust-build internally. Uses the faust declaration of the dsp name for the naming of the struct and module name.

For now, see [`example-jack`](examples/example-jack) for how this can be used with a simple Faust DSP file and [rust-jack](https://github.com/RustAudio/rust-jack).
