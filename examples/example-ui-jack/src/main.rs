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
        // This loops the volume up and when on max sets it down to 0 again.
        // It also reports the current output level of the signal.
        eprintln!("volume: {} dB", state.get_by_path("volume").unwrap());
        eprintln!("level:  {} dB", state.get_by_path("level").unwrap());
        volume += 10.;
        if volume > 4. {
            volume = -70.
        }
        state.set_by_path("volume", volume);
        state.send();
        thread::sleep(Duration::from_millis(200));
    });

    // Run the DSP as JACK client.
    jack::run_dsp(dsp);
}
