use dsp::{UIActive, UIActiveValue, UIPassive, UIPassiveValue, Volume, DSP_UI};
use faust_types::{FaustDsp, UIGet, UISelfSet};
use std::{
    io,
    thread::{self, sleep},
    time::Duration,
};
use strum::{EnumCount, IntoEnumIterator, VariantNames};
use triple_buffer::triple_buffer;
mod dsp;

fn main() {
    let mut dsp = Volume::new();
    // Get number of inputs and ouputs
    let num_inputs = dsp.get_num_inputs() as usize;
    let num_outputs = dsp.get_num_inputs() as usize;

    eprintln!("inputs: {num_inputs}");
    eprintln!("outputs: {num_outputs}");
    eprintln!("active params: {:?}", UIActive::VARIANTS);
    eprintln!("passive params: {:?}", UIPassive::VARIANTS);
    eprintln!("UI: {:#?}", DSP_UI);

    sleep(Duration::from_secs(3));
    // wait-free buffers for control io
    let (mut send_active, mut recv_active) =
        triple_buffer(&[UIActiveValue::Volume(-70.0f32); UIActiveValue::COUNT]);
    let (mut send_passive, mut recv_passive) =
        triple_buffer(&[UIPassiveValue::Level(-70.0f32); UIPassiveValue::COUNT]);
    let enum_vol = DSP_UI.volume;

    // Spawn a thread to do state changes.
    // This could be a GUI thread or API server.
    let mut volume = -70.;
    thread::spawn(move || loop {
        // This loops the volume up and when on max sets it down to 0 again.
        // It also reports the current output level of the signal.
        volume += 10.;
        if volume > 4. {
            volume = -70.
        }

        let v = enum_vol.value(volume);
        send_active.write([v]);
        let l = recv_passive.read();
        eprintln!("[active]: {:?} dB [passive]:  {:?} dB", v, l);
        sleep(Duration::from_millis(200));
    });

    // Create JACK client
    let (client, in_ports, mut out_ports) =
        jack_utils::create_jack_client("jacktest", num_inputs, num_outputs);

    // Init DSP with a given sample rate
    let sample_rate = client.sample_rate();
    dsp.init(sample_rate as i32);

    // Create JACK process closure that runs for each buffer
    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let len = ps.n_frames() as usize;

        let inputs = in_ports.iter().map(|p| p.as_slice(ps)).collect::<Vec<_>>();
        let mut outputs = out_ports
            .iter_mut()
            .map(|p| p.as_mut_slice(ps))
            .collect::<Vec<_>>();

        // update all active controls
        let buffer = recv_active.output_buffer_mut();
        for active in buffer {
            active.set(&mut dsp);
        }
        send_passive.publish();

        // run dsp computation
        dsp.compute(len, &inputs, &mut outputs);

        // send all passive controls
        let buffer = send_passive.input_buffer_mut();
        for (i, passive) in UIPassive::iter().enumerate() {
            buffer[i] = passive.get_enum(&dsp);
        }
        send_passive.publish();

        jack::Control::Continue
    };
    // Init JACK process handler.
    let process = jack::contrib::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = jack::AsyncClient::new(client, (), process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    active_client.deactivate().unwrap();
}
