#![feature(drain_filter)]
extern crate hsl;
extern crate midir;
#[macro_use]
extern crate clap;
extern crate ws;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use hsl::HSL;
use midir::{MidiInput, MidiInputConnection, Ignore};
use ws::{connect, Sender};
use std::sync::Arc;
use std::cell::RefCell;

#[derive(Serialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

struct ColorInfo {
    note: u8,
    velocity: u8,
}

struct Client {
    socket: Sender,
    stack: Vec<ColorInfo>,
}

impl std::fmt::Debug for ColorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ColorInfo {{ note: {}, velocity: {} }}", self.note, self.velocity)
    }
}

impl Client {
    fn update(&mut self) {
        let (hue, saturation, lightness) = match self.stack.last() {
            Some(ColorInfo { note, velocity }) => {
                let hue = (*note as f64) * 3_f64;
                let saturation = 1_f64;
                let lightness = (*velocity as f64) / 127_f64 * 0.7_f64;
                (hue, saturation, lightness)
            },
            None => {
                (0_f64, 0_f64, 0_f64)
            }
        };
        let color = HSL { h: hue, s: saturation, l: lightness };
        let rgb = color.to_rgb();
        let json = serde_json::to_string(&Color { r: rgb.0, g: rgb.1, b: rgb.2 }).unwrap();
        println!("Sending RGB {}", json);
        if let Err(error) = self.socket.send(json) {
            println!("Error sending message to server: {:?}", error);
        }
    }
    fn midi_message(&mut self, payload: &[u8]) {
        match payload {
            [144, note, velocity] => {
                self.stack.push(ColorInfo { note: *note, velocity: *velocity });
            },
            [128, note, velocity] => {
                self.stack.drain_filter(|info| info.note == *note && info.velocity == *velocity);
            },
            _ => {}
        }
        self.update();
    }
}

fn create_midi() -> MidiInput {
    let mut midi = MidiInput::new("LED Strip").expect("Couldn't open MIDI device.");
    midi.ignore(Ignore::None);
    midi
}

fn main() {
    use clap::App;
    let yml = load_yaml!("commandline.yml");
    let matches = App::from_yaml(yml).get_matches();
    match matches.subcommand() {
        ("midi-devices", Some(_)) => {
            let mut midi = create_midi();
            for i in 0..midi.port_count() {
                println!("MIDI device #{}: {}", i, midi.port_name(i).unwrap_or("Unkown".to_string()));
            }
        },
        ("start", Some(start_matches)) => {
            // Process arguments.
            let midi_device = value_t!(start_matches, "midi", usize).expect("Not a valid MIDI device.");
            let url = start_matches.value_of("url").expect("No url specified.");
            let mut connection_ref_cell: Arc<RefCell<Option<MidiInputConnection<()>>>> = Arc::new(RefCell::new(None));

            if let Err(error) = connect(url.clone(), move |socket| {
                let midi = create_midi();
                let port_name = midi.port_name(midi_device).unwrap_or("Unkown".to_string());
                println!("Connected to MIDI device {} ({}).", midi_device, port_name);

                // Create client.
                let mut client = Client { socket, stack: vec!() };
                // Listen for MIDI events and send them to the client.
                let connection = midi.connect(midi_device, "LED Strip", move |_, message, _| {
                    client.midi_message(message)
                }, {}).expect("Couldn't connect to MIDI port.");
                connection_ref_cell.replace(Some(connection));

                println!("Connected to {}.", url);
                |_| Ok(())
            }) {
                println!("Error with websocket connection: {:?}", error);
            }
        },
        ("", None) => println!("Unkown command"),
        _ => unreachable!(),
    }
}
