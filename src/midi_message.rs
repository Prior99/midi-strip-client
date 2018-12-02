pub enum MidiMessage {
    KeyPress { note: u8, velocity: u8 },
    KeyRelease { note: u8 },
    Unknown,
}

impl MidiMessage {
    pub fn new(message: [u8; 3]) -> MidiMessage {
        match message {
            [144, note, velocity] => MidiMessage::KeyPress { note, velocity },
            [128, note, _] => MidiMessage::KeyRelease { note },
            _ => MidiMessage::Unknown,
        }
    }
}
