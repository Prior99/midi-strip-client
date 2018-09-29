#![feature(drain_filter)]

extern crate hsl;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate midir;
extern crate ws;
extern crate serde;
extern crate serde_json;

mod color;
mod color_info;
mod client;
mod midi_message;
mod midi_client;

use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::cell::RefCell;
use midi_client::{create_midi, MidiClient};
use client::Client;
use ws::{connect};
use std::thread::{spawn};

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
            let midi_device_id = value_t!(start_matches, "midi", usize).expect("Not a valid MIDI device.");
            let url = start_matches.value_of("url").expect("No url specified.");
            let mut midi_client: Arc<RefCell<Option<MidiClient>>> = Arc::new(RefCell::new(None));

            if let Err(error) = connect(url.clone(), move |socket| {
                let (tx, rx) = sync_channel(3);
                midi_client.replace(Some(MidiClient::new(midi_device_id, tx)));
                spawn(move || Client::new(socket).poll(rx));
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
