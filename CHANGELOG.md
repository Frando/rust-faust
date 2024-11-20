
# Unreleased

## faust-build
- Add `FaustBuilder::build_xml()` to generate the description file. See `faust -xml` for details. @crop2000
- Add `FaustBuilder::set_arch_file()` to specify a custom architecture file @crop2000
- Add `FaustBuilder::set_faust_path()` to specify a custom path to the faust binary @crop2000
- Add `FaustBuilder::set_module_name` to specify the `mod $name` in the generated code. By default it's `dsp`. @obsoleszenz
- Add `FaustBuilder::set_struct_name` to specify the `struct $name` in the generated code. By default it's the CamelCased file name. @obsoleszenz
- Cleanups & refactorings to `FaustBuilder` @crop2000

## faust-macro
- New crate that implements a `proc_macro` to have faust code in your rust code.
  See `examples/example-jack-macro/src/main.rs` for details. @olafklingt

## faust-types
- Put import of `libm` and `jack` behind a feature. Use `default-features = false` to skip this. @amomentunfolding

## faust-state
- Evaluate assembler for flushing denormals at build-time. @plule
