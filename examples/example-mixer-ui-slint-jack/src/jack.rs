use faust_state::DspHandle;
use faust_types::FaustDsp;
use jack::AudioIn;
use jack::*;
use std::{io, slice};
use rtrb::{Consumer, Producer, RingBuffer};
pub use jack::RawMidi;

const MAX_MIDI: usize = 3;

//a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
pub struct Midi {
    pub len: usize,

    pub data: [u8; MAX_MIDI],
    pub time: jack::Frames,
}

impl From<jack::RawMidi<'_>> for Midi {
    fn from(midi: jack::RawMidi<'_>) -> Self {
        let len = std::cmp::min(MAX_MIDI, midi.bytes.len());
        let mut data = [0; MAX_MIDI];
        data[..len].copy_from_slice(&midi.bytes[..len]);
        Midi {
            len,
            data,
            time: midi.time,
        }
    }
}


impl std::fmt::Debug for Midi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Midi")
         .field("data", &self.data)
         .field("data (as hex)", &format!("{:02X?}", &self.data))
         .field("status", &format!("{:02x?}", self.status()))
         .field("channel", &format!("{:02x?}", self.channel()))
         .finish()
    }
}

impl Midi {
    pub fn status(&self) -> u8 {
        self.data[0] >> 4
    }
    pub fn channel(&self) -> u8 {
        self.data[0] & 0x0F
    }
    pub fn note(&self) -> u8 {
        self.data[1]
    }
    pub fn value(&self) -> u8 {
        self.data[2]
    }
}

pub fn run_dsp<T>(mut dsp: DspHandle<T>, mut midi_in_send: Producer<Midi>, mut midi_out_receiver: Consumer<Midi>)
where
    T: FaustDsp<T = f32> + 'static + Send,
{
    // Get number of inputs and ouputs
    let num_inputs = dsp.num_inputs();
    let num_outputs = dsp.num_outputs();

    // Create JACK client
    let (client, in_ports, mut out_ports, midi_in_ports, mut midi_out_ports) =
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
        if len as usize > buffer_size {
            panic!("JACK wants {} samples but our buffer can only hold {}", len, buffer_size);
        }

        // Copy audio input for all ports from jack to the faust input buffer
        for index_port in 0..num_inputs {
            let port = in_ports[index_port].as_slice(ps);
            inputs[index_port][0..len as usize].copy_from_slice(&port);
        }

        let midi_in_port = midi_in_ports.first().unwrap();
        for midi in midi_in_port.iter(ps) {
            midi_in_send.push(midi.into());
            
        }

        // Call the update_and_compute handler on the Faust DSP. This first processes param changes
        // from the State handler and then computes the outputs from the inputs and params.
        dsp.update_and_compute(len as i32, &buffer_input[..], &mut buffer_output[..]);

        // Copy audio output for all ports from faust to the jack output
        for index_port in 0..num_outputs {
            let port = out_ports[index_port].as_mut_slice(ps);
            port.copy_from_slice(&outputs[index_port][0..len as usize]);
        }

        let mut midi_out_port = midi_out_ports.get_mut(0).unwrap();
        let mut midi_out_writer = midi_out_port.writer(ps);
        while let Ok(midi) = midi_out_receiver.pop() {
            midi_out_writer.write(&RawMidi {
                time: midi.time,
                bytes: &midi.data
            }).unwrap();
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
) -> (jack::Client, Vec<Port<AudioIn>>, Vec<Port<AudioOut>>, Vec<Port<MidiIn>>, Vec<Port<MidiOut>>) {
    let (client, _status) = jack::Client::new(name, jack::ClientOptions::NO_START_SERVER).unwrap();
    let mut in_ports: Vec<Port<AudioIn>> = Vec::new();
    let mut out_ports: Vec<Port<AudioOut>> = Vec::new();
    let mut midi_in_ports: Vec<Port<MidiIn>> = Vec::new();
    let mut midi_out_ports: Vec<Port<MidiOut>> = Vec::new();

    for i in 0..num_inputs {
        let left_or_right = if i % 2 == 0 { "l" } else { "r" };
        let name = if i < 8 {
            let n = i % 8 / 2 + 1;
            format!("channel{}_{}", n, left_or_right)
        } else if i < 12 {
            let n = i % 4 / 2 + 1;
            format!("return_fx{}_{}", n, left_or_right)
        } else {
            format!("in_{}", i)
        };

        let port = client
            .register_port(&name, jack::AudioIn::default())
            .unwrap();
        in_ports.push(port);
    }
    for i in 0..num_outputs {
        let left_or_right = if i % 2 == 0 { "l" } else { "r" };
        
        let name = if i < 2 {
            format!("main_{}", left_or_right)
        } else if i < 4 {
            let n = i % 4 / 2 + 1;
            format!("booth_{}", left_or_right)
        } else if i < 6 {
            format!("cue_{}", left_or_right)
        } else if i < 10 {
            let n = i % 4 / 2 + 1;
            format!("send_fx{}_{}", n, left_or_right)
        } else {
            format!("output_{}", i)

        };
        let port = client
            .register_port(&name, jack::AudioOut::default())
            .unwrap();
        out_ports.push(port);
    }
    midi_in_ports.push(client.register_port("midi_in0", jack::MidiIn::default()).unwrap());
    midi_out_ports.push(client.register_port("midi_out0", jack::MidiOut::default()).unwrap());
    (client, in_ports, out_ports, midi_in_ports, midi_out_ports)
}
