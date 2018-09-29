use ws::{Sender};
use color_info::ColorInfo;
use color:: Color;
use midi_message::MidiMessage;
use std::time::{Duration, Instant};
use palette::rgb::LinSrgba;
use palette::Blend;

pub struct Client {
    socket: Sender,
    stack: Vec<ColorInfo>,
    release: Duration,
}

impl Client {
    pub fn new(socket: Sender, release: Duration) -> Client {
        Client { socket, release, stack: vec!() }
    }

    fn mix(&self, now: &Instant) -> LinSrgba {
        let mut result = LinSrgba::new(0_f32, 0_f32, 0_f32, 1_f32);
        let iterator = self.stack.iter()
            .map(|color_info| LinSrgba::from(color_info.to_hsla(&now)));
        for color in iterator {
            result = color.over(result);
        }
        result
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let (r, g, b, _) = self.mix(&now).into_components();
        debug!("Sending RGB ({}, {}, {})", r, g, b);
        let json = serde_json::to_string(&Color {
            r: (r * 255_f32) as u8,
            g: (g * 255_f32) as u8,
            b: (b * 255_f32) as u8,
        }).unwrap();
        if let Err(error) = self.socket.send(json) {
            error!("Error sending message to server: {:?}", error);
        }
        self.stack.drain_filter(|info| info.is_gone(&now));
    }

    fn handle_key_release(&mut self, note: u8) {
        match self.stack.iter_mut().find(|info| info.note == note) {
            Some(color_info) => color_info.deleted = Some(Instant::now()),
            None => {},
        };
    }

    fn handle_key_press(&mut self, note: u8, velocity: u8) {
        self.stack.push(ColorInfo::new(note, velocity, self.release));
    }

    pub fn handle_message(&mut self, message: MidiMessage) {
        match message {
            MidiMessage::KeyPress { note, velocity } => self.handle_key_press(note, velocity),
            MidiMessage::KeyRelease { note } => self.handle_key_release(note),
            _ => {},
        };
    }
}
