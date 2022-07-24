use std::{thread, time::Duration};
use faust_state::DspHandle;
use std::sync::mpsc::channel;

use slint;
slint::include_modules!();

mod jack;
mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}

pub struct Event {
    path: String,
    value: f32
}

fn main() {
    let (dsp, mut state) = DspHandle::<faust::Volume>::new();

    let ui = HelloWorld::new();
    let (event_sender, event_receiver) = channel::<Event>();

    // Audio thread taking care of JACK & computing dsp in faust
    thread::spawn(move ||  {
        // Run the DSP as JACK client.
        jack::run_dsp(dsp);
    });

    ui.global::<Logic>().on_roundoff(move |number, decimals| {
        return format!("{:.1$}", number, decimals as usize).into();
    });

    ui.on_ui_update(move |path, value| {
        event_sender.clone().send(Event {path: path.into(), value}).unwrap(); 
    });

    let ui_weak = ui.as_weak();
    thread::spawn(move || {
        let ui = ui_weak;
        let event_receiver = event_receiver;
        // Init
        let volume_value = state.get_by_path("volume").unwrap().clone();
        ui.upgrade_in_event_loop(move |handle| {
            handle.set_volume(volume_value);
        });

        loop {
            state.update();
            let vumeter_value = state.get_by_path("vumeter").unwrap().clone();
            ui.upgrade_in_event_loop(move |handle| {
                handle.set_vumeter(vumeter_value);
            });
            if let Ok(event) = event_receiver.try_recv() {
                state.set_by_path(&event.path, event.value).unwrap(); 
                state.send();
            }
                    
            thread::sleep(Duration::from_millis(33));
        }
    });

    ui.run();
}
