use std::{thread, time::Duration};
use faust_state::{DspHandle, StateHandle};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::{SystemTime};
use rtrb::{Consumer, Producer, RingBuffer};

use slint;
use slint::VecModel;
use slint::Weak;

slint::include_modules!();

mod jack;
mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}

#[derive(Debug)]
pub struct Event {
    path: String,
    value: f32
}

type Volumes = [f32; 4];
type Gains = [f32; 4];
type CueToggles = [f32; 4];

struct UILoop {
    ui: Weak<UI>,
    state: StateHandle,
    event_receiver: Receiver<Event>, 
    midi_in_receiver: Consumer<jack::Midi>,
    midi_out_send: Producer<jack::Midi>
}

impl UILoop {
    fn new(ui: Weak<UI>, state: StateHandle, event_receiver: Receiver<Event>, midi_in_receiver: Consumer<jack::Midi>, midi_out_send: Producer<jack::Midi>) -> Self {
        UILoop {
            ui,
            state,
            event_receiver,
            midi_in_receiver,
            midi_out_send
        }
    }
    fn run(&mut self) {
        // Init values
        let volumes = self.get_volumes();
        let gains = self.get_gains();
        let cue_toggles = self.get_cue_toggles();
        self.update_ui(volumes, gains, cue_toggles);
        let mut last_ui_update = SystemTime::UNIX_EPOCH;
        loop {
            for event in self.event_receiver.try_iter() {
                println!("[UILoop] Received event {:?}", event);
                self.state.set_by_path(&event.path, event.value).unwrap();
            }

            self.process_midi_in().unwrap();

            self.state.send();
            self.state.update();
            let current_time = SystemTime::now(); 
            if last_ui_update.elapsed().unwrap() >= Duration::from_millis(33) {
                last_ui_update = current_time;
                let volumes = self.get_volumes();

                let gains = self.get_gains();
                let cue_toggles = self.get_cue_toggles();
                self.update_ui(volumes, gains, cue_toggles);

                /*for (key,value) in self.state.params().iter() {
                    println!("{}: {}", value.path(), self.state.get_param(key.clone()).unwrap());
                }*/
            }

                    
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn process_midi_in(&mut self) -> Result<(), &str> {
        while !self.midi_in_receiver.is_empty() {
            let midi_in = self.midi_in_receiver.pop().unwrap();
            println!("[UILoop] [MidiIn] {:?}", midi_in);


            let status = midi_in.status();
            let channel = midi_in.channel();
            let note = midi_in.note();
            let value = midi_in.value();

            if status != 0xB {
                println!("[UILoop] [MidiIn] Unknown Midi command?");
                continue;
            }

            if note == 4 {
                let key = format!("volume{}", channel);
                let value = value as f32 / 127.0;
                println!("{}: {}", key, value);
                self.state.set_by_path(&key, value).unwrap(); 
            } else if note == 3 {
                let key = format!("gain{}", channel);
                let value = value as f32 / 127.0;
                println!("{}: {}", key, value);
                self.state.set_by_path(&key, value).unwrap(); 
            } else if note == 7 && value == 127 {
                let key = format!("cue_toggle{}", channel);
                let value = if self.state.get_by_path(&key).unwrap() == &1.0 { 0.0 } else { 1.0 };
                println!("{}: {}", key, value);
                self.state.set_by_path(&key, value).unwrap(); 
            }
        }

        Ok(())
    }


    fn get_gains(&self) -> Gains {
        [
            self.state.get_by_path("gain0").unwrap_or(&0.0).clone(),
            self.state.get_by_path("gain1").unwrap_or(&0.0).clone(),
            self.state.get_by_path("gain2").unwrap_or(&0.0).clone(),
            self.state.get_by_path("gain3").unwrap_or(&0.0).clone(),
        ]
    }

    fn get_volumes(&self) -> Volumes {
        [
            self.state.get_by_path("volume0").unwrap_or(&0.0).clone(),
            self.state.get_by_path("volume1").unwrap_or(&0.0).clone(),
            self.state.get_by_path("volume2").unwrap_or(&0.0).clone(),
            self.state.get_by_path("volume3").unwrap_or(&0.0).clone(),
        ]
    }

    fn get_cue_toggles(&self) -> CueToggles {
        [
            self.state.get_by_path("cue_toggle0").unwrap_or(&0.0).clone(),
            self.state.get_by_path("cue_toggle1").unwrap_or(&0.0).clone(),
            self.state.get_by_path("cue_toggle2").unwrap_or(&0.0).clone(),
            self.state.get_by_path("cue_toggle3").unwrap_or(&0.0).clone(),
        ]

    }
    fn update_ui(&self, volumes: Volumes, gains: Gains, cue_toggles: CueToggles) {
        self.ui.upgrade_in_event_loop(move |handle| {
            let volumes = VecModel::<f32>::from_slice(volumes.as_slice());
            handle.global::<Backend>().set_volumes(volumes.into());
            let gains = VecModel::<f32>::from_slice(gains.as_slice());
            handle.global::<Backend>().set_gains(gains.into());
            let cue_toggles = VecModel::<f32>::from_slice(cue_toggles.as_slice());
            handle.global::<Backend>().set_cue_toggles(cue_toggles.into());

        });
    }
}

fn main() {
    let (dsp, mut state) = DspHandle::<faust::Mixer>::new();

    println!("{:?}", state.params());

    let ui = UI::new();
    let (event_sender, event_receiver) = channel::<Event>();
    let (mut midi_in_send, mut midi_in_receiver) = RingBuffer::<jack::Midi>::new(100).split(); 
    let (mut midi_out_send, mut midi_out_receiver) = RingBuffer::<jack::Midi>::new(100).split(); 

    // Audio thread taking care of JACK & computing dsp in faust
    thread::spawn(move ||  {
        // Run the DSP as JACK client.
        jack::run_dsp(dsp, midi_in_send, midi_out_receiver);
    });

    ui.global::<Logic>().on_roundoff(move |number, decimals| {
        return format!("{:.1$}", number, decimals as usize).into();
    });

    ui.global::<Logic>().on_dbconvert(move |db| {
        if db < -43.0 {
            0
        } else if db < -27.0 {
            1
        } else if db < -16.0 {
            2
        } else if db < -10.0 {
            3
        } else if db < -6.0 {
            4
        } else if db < -3.0 {
            5
        } else if db < -2.0 {
            6
        } else if db < -1.0 {
            7
        } else if db < 0.0 {
            8
        } else {
            9
        }
    });

    ui.global::<Backend>().on_send(move |path, value| {
        event_sender.clone().send(Event {path: path.into(), value}).unwrap(); 
    });


    let mut ui_loop = UILoop::new(
        ui.as_weak(),
        state,
        event_receiver,
        midi_in_receiver,
        midi_out_send
    );
    thread::spawn(move || {
        ui_loop.run();
    });

    ui.run();
}
