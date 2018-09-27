extern crate hsl;
extern crate midir;

use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::net::UdpSocket;
use hsl::HSL;
use midir::{MidiInput, Ignore};

fn main() {
    let mut midi = MidiInput::new("LED Strip").expect("Couldn't open MIDI device.");
    println!("Ports: {}", midi.port_count());
    midi.connect(12, "LED Strip", move |stamp, message, _| {
        println!("Message: {}", stamp);
    }, {}).expect("Couldn't connect");
}
