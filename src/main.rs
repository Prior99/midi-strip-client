#![feature(drain_filter)]
extern crate hsl;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate midir;
extern crate ws;
extern crate serde;
extern crate serde_json;

mod color;
mod color_info;
mod client;
mod midi_message;
mod midi_client;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{sync_channel, Receiver};
use std::cell::RefCell;
use std::thread;
use midi_client::{create_midi, MidiClient};
use client::Client;
use ws::{connect};
use std::thread::{spawn, JoinHandle};
use midi_message::MidiMessage;
use std::time::Duration;

fn thread_update(client: Arc<Mutex<Client>>) -> JoinHandle<()> {
    spawn(move || {
        loop {
            client.lock().unwrap().update();
            thread::sleep(Duration::from_millis(10));
        }
    })
}

fn thread_messages(client: Arc<Mutex<Client>>, rx: Receiver<MidiMessage>) -> JoinHandle<()> {
    spawn(move || {
        for message in rx.iter() {
            let mut client = client.lock().unwrap();
            client.handle_message(message);
        }
    })
}

fn main() {
    use clap::App;
    let yml = load_yaml!("commandline.yml");
    let matches = App::from_yaml(yml).get_matches();
    let log_level = if matches.is_present("verbose") {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };
    if let Err(log_error) = simplelog::TermLogger::init(log_level, simplelog::Config::default()) {
        println!("Couldn't setup logging: {}", log_error);
    }
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
                let client_arc = Arc::new(Mutex::new(Client::new(socket)));
                thread_update(client_arc.clone());
                thread_messages(client_arc.clone(), rx);
                info!("Connected to {}.", url);
                |_| Ok(())
            }) {
                error!("Error with websocket connection: {:?}", error);
            }
        },
        ("", None) => println!("Unkown command"),
        _ => unreachable!(),
    }
}
