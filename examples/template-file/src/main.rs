use std::{thread, time::Duration};

use faust_state::DspHandle;
use jack_utils::run_dsp_as_jack_client;

mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}



fn main() {
    let (dsp, mut state) = DspHandle::<faust::Dbmeter>::new();
    eprintln!("client name: {}", dsp.name());
    eprintln!("inputs: {}", dsp.num_inputs());
    eprintln!("outputs: {}", dsp.num_outputs());
    eprintln!("params: {:#?}", state.params());
    eprintln!("meta: {:#?}", state.meta());

    // Spawn a thread to do state changes.
    // This could be a GUI thread or API server.
    thread::spawn(move || loop {
        state.update();
        eprintln!("volume: {:?} dB", state.get_by_path("channel0").unwrap());
        thread::sleep(Duration::from_millis(200));
    });

    // Run the DSP as JACK client.
    run_dsp_as_jack_client(dsp);
}
