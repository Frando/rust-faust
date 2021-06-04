use std::{thread, time::Duration};
use std::sync::mpsc;

use faust_state::DspHandle;

mod jack;
mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}


fn int_ranged_float(min: f32, max: f32, number: u8) -> f32 {
    let distance = max - min;
    let step = distance / 128.0;
    println!("{} {}", distance, step);
    return number as f32 * step;

}

fn main() {
    let (dsp, mut state) = DspHandle::<faust::Volume>::new();
    eprintln!("client name: {}", dsp.name());
    eprintln!("inputs: {}", dsp.num_inputs());
    eprintln!("outputs: {}", dsp.num_outputs());
    eprintln!("params: {:#?}", state.params());
    eprintln!("meta: {:#?}", state.meta());


    let (tx, rx) = mpsc::channel::<u8>();

    // Spawn a thread to do state changes.
    // This could be a GUI thread or API server.
    thread::spawn(move || loop {
        let volume = int_ranged_float(0.0, 1.0, rx.recv().unwrap());
        println!("Hallo! {}", volume);
        state.set_by_path("volume", volume);
        state.send();
    });

    // Run the DSP as JACK client.
    jack::run_dsp(dsp, move |num_channel, midi_message| {
        //println!("Hallo! {} {:?}", num_channel, midi_message.bytes);

        tx.send(midi_message.bytes[2].clone().into()).unwrap();
    });
}
