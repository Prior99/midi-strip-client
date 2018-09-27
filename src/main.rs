extern crate hsl;
extern crate midir;
#[macro_use]
extern crate clap;

use std::io::stdin;
use std::net::UdpSocket;
use hsl::HSL;
use midir::{MidiInput, MidiInputConnection, Ignore};

struct Client {
    socket: UdpSocket,
    port: u16,
    host: String,
}

impl Client {
    fn midi_message(&mut self, timestamp: u64, payload: &[u8]) {
        if (payload.len() != 3) {
            println!("Encountered unknown MIDI message with {} bytes of length.", payload.len());
        }
        match payload {
            [144, note, velocity] => {
                let hue = (*note as f64) * 3_f64;
                let saturation = 1_f64;
                let lightness = (*velocity as f64) / 127_f64 * 0.7_f64;
                let color = HSL { h: hue, s: saturation, l: lightness };
                let rgb = color.to_rgb();
                let buffer: [u8;3] = [rgb.0, rgb.1, rgb.2];
                self.socket.send_to(&buffer, format!("{}:{}", self.host, self.port));
            },
            [128, note, velocity] => {
                self.socket.send_to(&[0, 0, 0], format!("{}:{}", self.host, self.port));
            },
            _ => {}
        }
        println!("Message: {}: {:?}", payload.len(), payload);
    }
}

fn main() {
    use clap::App;
    let yml = load_yaml!("commandline.yml");
    let matches = App::from_yaml(yml).get_matches();
    let mut midi = MidiInput::new("LED Strip").expect("Couldn't open MIDI device.");
    midi.ignore(Ignore::None);
    match matches.subcommand() {
        ("list-ports", Some(list_port_matches)) => {
            for i in 0..midi.port_count() {
                println!("MIDI port {}: {}", i, midi.port_name(i).unwrap_or("Unkown".to_string()));
            }
        },
        ("start", Some(start_matches)) => {
            // Process arguments.
            let midi_port = value_t!(start_matches, "midi-port", usize).expect("Not a valid MIDI port.");
            let local_port = value_t!(start_matches, "local-port", u16).expect("Not a valid port.");
            let port = value_t!(start_matches, "port", u16).expect("Not a valid port.");
            let host = start_matches.value_of("host").expect("No host specified.");

            // Bind to UDP socket.
            let socket = UdpSocket::bind(format!("0.0.0.0:{}", local_port)).expect("Couldn't bind to UDP socket.");
            println!("UDP socket bound to 0.0.0.0:{} sending to {}:{}", local_port, host, port);

            // Confirm used MIDI port.
            let port_name = midi.port_name(midi_port).unwrap_or("Unkown".to_string());
            println!("Connected to MIDI port {} ({}).", midi_port, port_name);

            // Create client.
            let mut client = Client { socket, port, host: String::from(host) };

            // Listen for MIDI events and send them to the client.
            let connection = midi.connect(midi_port, "LED Strip", move |timestamp, message, _| {
                client.midi_message(timestamp, message)
            }, {}).expect("Couldn't connect to MIDI port.");

            // Exit once the user has pressed the enter key.
            let mut input = String::new();
            println!("Press [Enter] to exit.");
            stdin().read_line(&mut input).expect("Unable to open stdin.");
        },
        ("", None) => println!("Unkown command"),
        _ => unreachable!(),
    }
}
