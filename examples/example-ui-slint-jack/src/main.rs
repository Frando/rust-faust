use std::{thread, time::Duration};

use faust_state::DspHandle;
use slint;
slint::include_modules!();

mod jack;
mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}

fn main() {
    let (dsp, mut state) = DspHandle::<faust::Volume>::new();
    eprintln!("client name: {}", dsp.name());
    eprintln!("inputs: {}", dsp.num_inputs());
    eprintln!("outputs: {}", dsp.num_outputs());
    eprintln!("params: {:#?}", state.params());
    eprintln!("meta: {:#?}", state.meta());

    let ui = HelloWorld::new();

    // Spawn a thread to do state changes.
    // This could be a GUI thread or API server.
    thread::spawn(move ||  {
        // Run the DSP as JACK client.
        jack::run_dsp(dsp);
    });

    let ui_weak = ui.as_weak();
    ui.global::<Logic>().on_roundoff(move |number, decimals| {
        return format!("{:.1$}", number, decimals as usize).into();
    });
    ui.on_ui_update(move |key, value| {
        state.set_by_path(&key, value);
        state.update();
    });

    thread::spawn(move || {
        loop {
            
            thread::sleep(Duration::from_millis(200));
        }
    });

    ui.run();
}
