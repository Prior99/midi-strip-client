use hsl::HSL;
use ws::{Sender};
use midi_message::MidiMessage;
use std::sync::mpsc::{Receiver};
use color_info::ColorInfo;
use color:: Color;

pub struct Client {
    socket: Sender,
    stack: Vec<ColorInfo>,
}

impl Client {
    pub fn new(socket: Sender) -> Client {
        Client { socket, stack: vec!() }
    }

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

    fn handle_key_release(&mut self, note: u8) {
        self.stack.drain_filter(|info| info.note == note);
    }

    fn handle_key_press(&mut self, note: u8, velocity: u8) {
        self.stack.push(ColorInfo { note, velocity });
    }

    pub fn poll(&mut self, rx: Receiver<MidiMessage>) {
        for message in rx.iter() {
            match message {
                MidiMessage::KeyPress { note, velocity } => self.handle_key_press(note, velocity),
                MidiMessage::KeyRelease { note } => self.handle_key_release(note),
                _ => {},
            };
            self.update();
        }
    }
}
