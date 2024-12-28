use faust_state::DspHandle;
use jack_utils::run_dsp_as_jack_client;
use std::{thread, time::Duration};

faust_macro::faust!(
    declare flags       "-single"; // example for possible flags declaration use
    declare name        "VolumeControl"; //necessary declaration to have a valid name
    declare version     "1.0";
    declare author      "Franz Heinzmann";
    declare license     "BSD";
    declare options     "[osc:on]";

    import("stdfaust.lib");

    stereo(func) = _,_ : func(_),func(_) : _,_;

    volumeM = *(vslider("volume", 0, -70, +4, 0.1) : ba.db2linear : si.smoo);
    volume = stereo(volumeM);

    envelop = abs : max ~ -(1.0/ma.SR) : max(ba.db2linear(-70)) : ba.linear2db;
    vumeterM(x) = envelop(x) : vbargraph("level[2][unit:dB][style:dB]", -60, +5);
    vumeterS(a,b) = a,b <: _,_,_,_ :
    (a, b, attach(0,vumeterM((a+b)/2)), 0) :>
    _,_;
    vumeter = _,_ : vumeterS(_,_);

    faderchannel = _,_ : volume : vumeter : _,_;
    process = faderchannel;
);

fn main() {
    let (dsp, mut state) = DspHandle::<VolumeControl>::new();
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
            volume = -70.;
        }
        let _ = state.set_by_path("volume", volume);
        state.send();
        thread::sleep(Duration::from_millis(200));
    });

    // Run the DSP as JACK client.
    run_dsp_as_jack_client(dsp);
}

