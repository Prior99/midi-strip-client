use std::sync::mpsc::{SyncSender};
use midir::{MidiInput, MidiInputConnection, Ignore};
use midi_message::MidiMessage;

pub struct MidiClient {
    _connection: MidiInputConnection<()>,
}

impl MidiClient {
    pub fn new(midi_device_id: usize, tx: SyncSender<MidiMessage>) -> MidiClient {
        let midi = create_midi();
        let port_name = midi.port_name(midi_device_id).unwrap_or("Unkown".to_string());
        info!("Connecting to MIDI device {} ({}).", midi_device_id, port_name);
        let connection = midi
            .connect(midi_device_id, "LED Strip", move |_, message, _| {
                debug!("MIDI message received {:?}", message);
                let mut array: [u8; 3] = Default::default();
                array.copy_from_slice(message);
                if let Err(error) = tx.send(MidiMessage::new(array)) {
                    println!("Error communicating with client thread: {:?}", error);
                }
            }, {})
            .expect("Couldn't connect to MIDI port.");
        MidiClient { _connection: connection }
    }
}

pub fn create_midi() -> MidiInput {
    let mut midi = MidiInput::new("LED Strip").expect("Couldn't open MIDI device.");
    midi.ignore(Ignore::None);
    midi
}
