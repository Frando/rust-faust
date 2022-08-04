use faust_build::build_dsp;

fn main() {
    println!("cargo:rerun-if-changed=dsp");
    build_dsp("dsp/mixer.dsp");

    #[cfg(all(
        not(any(windows, target_vendor = "apple")),
        not(any(feature = "x11", feature = "wayland"))
    ))]
    compile_error!(
        "At least one of \"x11\" or \"wayland\" features must be enabled (both may be enabled)."
    );
    slint_build::compile("ui/main.slint").unwrap();
}
