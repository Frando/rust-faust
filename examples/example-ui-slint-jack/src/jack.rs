use faust_state::DspHandle;
use faust_types::FaustDsp;
use jack::AudioIn;
use jack::*;
use std::{io, slice};

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
    let sample_rate = client.sample_rate();
    dsp.init(sample_rate as i32);

    // Init input and output buffers
    let buffer_size = (client.buffer_size() * 2) as usize;
    let mut inputs: Vec<Vec<f32>> = vec![vec![0_f32; buffer_size]; num_inputs];
    let mut outputs: Vec<Vec<f32>> = vec![vec![0_f32; buffer_size]; num_outputs];

    // Map our Vec<Vec<f32>> to a Vec<&f[32]> to create a buffer for the faust lib
    let buffer_input: Vec<&[f32]> = inputs
        .iter()
        .map(|input| unsafe { slice::from_raw_parts(input.as_ptr(), buffer_size) })
        .collect();

    // Map our Vec<Vec<f32>> to a Vec<&f[32]> to create a buffer for the faust lib
    let mut buffer_output: Vec<&mut [f32]> = outputs
        .iter_mut()
        .map(|output| unsafe { slice::from_raw_parts_mut(output.as_mut_ptr(), buffer_size) })
        .collect();

    // Create JACK process closure that runs for each buffer
    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let len = ps.n_frames();
        if (len as usize > buffer_size) {
            panic!("JACK wants {} samples but our buffer can only hold {}", len, buffer_size);
        }

        // Copy audio input for all ports from jack to the faust input buffer
        for index_port in 0..num_inputs {
            let port = in_ports[index_port].as_slice(ps);
            inputs[index_port][0..len as usize].copy_from_slice(&port);
        }

        // Call the update_and_compute handler on the Faust DSP. This first processes param changes
        // from the State handler and then computes the outputs from the inputs and params.
        dsp.update_and_compute(len as i32, &buffer_input[..], &mut buffer_output[..]);

        // Copy audio output for all ports from faust to the jack output
        for index_port in 0..num_outputs {
            let port = out_ports[index_port].as_mut_slice(ps);
            port.copy_from_slice(&outputs[index_port][0..len as usize]);
        }

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
