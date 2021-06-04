use faust_state::DspHandle;
use faust_types::FaustDsp;
use jack::AudioIn;
use jack::*;
use smallvec::SmallVec;
use std::io;

pub fn run_dsp<T>(mut dsp: DspHandle<T>)
where
    T: FaustDsp<T = f32> + 'static + Send,
{
    // Get number of inputs and ouputs
    let num_inputs = dsp.num_inputs();
    let num_outputs = dsp.num_inputs();

    // Create JACK client
    let (client, in_ports, mut out_ports) =
        create_jack_client(dsp.name(), num_inputs as usize, num_outputs as usize);

    // Init DSP with a given sample rate
    dsp.init(client.sample_rate() as i32);

    // Create JACK process closure that runs for each buffer
    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        // TODO: Make sure that this doesn't allocate.
        let mut inputs = SmallVec::<[&[f32]; 64]>::with_capacity(num_inputs as usize);
        let mut outputs = SmallVec::<[&mut [f32]; 64]>::with_capacity(num_outputs as usize);
        let len = ps.n_frames();
        for port in in_ports.iter() {
            inputs.push(port.as_slice(ps));
        }
        for port in out_ports.iter_mut() {
            outputs.push(port.as_mut_slice(ps));
        }

        // Call the update_and_compute handler on the Faust DSP. This first processes param changes
        // from the State handler and then computes the outputs from the inputs and params.
        dsp.update_and_compute(len as i32, &inputs[..], &mut outputs[..]);
        jack::Control::Continue
    };
    // Init JACK process handler.
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = jack::AsyncClient::new(client, (), process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    active_client.deactivate().unwrap();
}

fn create_jack_client(
    name: &str,
    num_inputs: usize,
    num_outputs: usize,
) -> (jack::Client, Vec<Port<AudioIn>>, Vec<Port<AudioOut>>) {
    let (client, _status) = jack::Client::new(name, jack::ClientOptions::NO_START_SERVER).unwrap();
    let mut in_ports: Vec<Port<AudioIn>> = Vec::new();
    let mut out_ports: Vec<Port<AudioOut>> = Vec::new();

    for i in 0..num_inputs {
        let port = client
            .register_port(&format!("in{}", i), jack::AudioIn::default())
            .unwrap();
        in_ports.push(port);
    }
    for i in 0..num_outputs {
        let port = client
            .register_port(&format!("out{}", i), jack::AudioOut::default())
            .unwrap();
        out_ports.push(port);
    }
    (client, in_ports, out_ports)
}
