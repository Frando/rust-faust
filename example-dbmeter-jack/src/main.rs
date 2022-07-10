use std::{thread, time::Duration};

use faust_state::DspHandle;

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

    let mut volume = -70.;

    // Spawn a thread to do state changes.
    // This could be a GUI thread or API server.
    thread::spawn(move || loop {
        eprintln!("volume: {:?} dB", state.get_by_path("vu1/test1").unwrap());
        thread::sleep(Duration::from_millis(200));
    });

    // Run the DSP as JACK client.
    jack::run_dsp(dsp);
}
