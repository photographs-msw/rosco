use crate::audio_gen::AudioGen;
use crate::instrument::Instrument;
use crate::note::Note;
use crate::sequence::Sequence;

pub(crate) struct Channel {
    sequence: Sequence,
    volume: f32
}

impl Channel {

   fn from(sequence: Sequence, volume: f32) -> Self {
        Channel {
            sequence,
            volume
        }
    }
}

#[allow(dead_code)]
pub(crate) struct MultiInstrument {
    num_channels: usize,
    channels: Vec<Channel>,
    audio_gen: AudioGen,
    // TODO THINK ABOUT MOVING NOTE INDEX TO SEQUENCE TO ADVANCE EACH INDEPENDENTLY. THIS WILL SUPPORT POLYPHONY
    //  WITH DIFFERENT DURATIONS
    //  ALSO SEMANTICALLY THIS IS WHERE IT SHOULD BE, JUST LIKE IT OWNS ITS VOLUME
    note_index: usize
}

#[allow(dead_code)]
impl MultiInstrument {

    pub fn from_instruments(instruments: Vec<Instrument>) -> Self {
        let mut channels = Vec::new();
        let mut channel_oscillators = Vec::new();
        let num_channels = instruments.len();
        for instrument in instruments {
            channels.push(Channel::from(Sequence::new(), 1.0 / num_channels as f32));
            let mut oscillators = Vec::new();
            for oscillator in &instrument.oscillators {
                oscillators.push(oscillator.clone());
            }
            channel_oscillators.push(oscillators);
        }
        MultiInstrument {
            num_channels,
            channels,
            audio_gen: AudioGen::from_channel_oscillators(channel_oscillators),
            note_index: 0
        }
    }

    pub fn play(&mut self) {
        self.audio_gen.gen_notes(&self.get_next_notes());
        // TODO FIGURE OUT ADVANCING SEQUENCE LOGIC HERE
        self.note_index += 1;
    }

    // TODO MAYBE HAVE WRAPPER METHODS TO ADVANCE SEQUENCES HERE

    pub fn add_note(&mut self, channel_num: usize, note: Note) {
        self.channels[channel_num].sequence.add_note(note);
    }

    pub fn add_note_to_channels(&mut self, note: Note) {
        for channel in &mut self.channels {
            channel.sequence.add_note(note);
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        for channel in &mut self.channels {
            channel.volume = volume;
        }
    }

    pub fn set_channel_volume(&mut self, channel_num: u8, volume: f32) {
        self.channels[channel_num as usize].volume = volume;
    }

    pub fn get_next_notes(&self) -> Vec<Note> {
        let mut notes = Vec::new();
        for channel in &self.channels {
            let mut note = channel.sequence.notes[self.note_index].clone();
            note.volume *= channel.volume;
            notes.push(note);
        }
        notes
    }
}
