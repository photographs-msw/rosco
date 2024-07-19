use crate::audio_gen::gen_notes;
use crate::note::Note;
use crate::oscillator::OscType;
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
    channel_oscillators: Vec<Vec<OscType>>,
    channels: Vec<Channel>,
}

#[allow(dead_code)]
impl MultiInstrument {

    pub fn from_channel_oscillators(channel_oscillators: Vec<Vec<OscType>>) -> Self {
        let mut channels = Vec::new();
        let num_channels = channel_oscillators.len();
        for _ in 0..num_channels {
            channels.push(Channel::from(Sequence::new(), 1.0 / num_channels as f32));
        }
        MultiInstrument {
            channel_oscillators,
            channels,
        }
    }

    pub fn play_channel_notes(&self) {
        gen_notes(self.get_next_notes(), self.channel_oscillators.clone());
    }

    pub fn play_channel_notes_and_advance(&mut self) {
        let notes = self.get_next_notes();
        gen_notes(notes, self.channel_oscillators.clone());
        for channel in self.channels.iter_mut() {
            channel.sequence.advance();
        }
    }

    pub fn reset_all_channels(&mut self) {
        for channel in &mut self.channels {
            channel.sequence.reset_index();
        }
    }

    pub fn loop_once(&mut self) {
        self.reset_all_channels();
        while !self.channels.iter().all(|channel| channel.sequence.at_end()) {
            self.play_channel_notes_and_advance();
        }
    }

    pub fn loop_n(&mut self, n: u8) {
        self.reset_all_channels();
        for _ in 0..n {
            self.loop_once();
        }
    }

    pub fn add_note_to_channel(&mut self, channel_num: usize, note: Note) {
        self.channels[channel_num].sequence.add_note(note);
    }

    pub fn add_note_to_channels(&mut self, note: Note) {
        for channel in &mut self.channels {
            channel.sequence.add_note(note);
        }
    }

    pub fn set_volume_for_channels(&mut self, volume: f32) {
        for channel in &mut self.channels {
            channel.volume = volume;
        }
    }

    pub fn set_volume_for_channel(&mut self, channel_num: usize, volume: f32) {
        self.channels[channel_num].volume = volume;
    }

    pub fn play_notes_direct(&self, notes: Vec<Note>) {
        gen_notes(notes, self.channel_oscillators.clone());
    }

    fn get_next_notes(&self) -> Vec<Note> {
        let mut notes = Vec::new();
        for channel in &self.channels {
            if channel.sequence.at_end() {
                continue;
            }
            let mut note = channel.sequence.get_note();
            note.volume *= channel.volume;
            notes.push(note);
        }
        notes
    }
}
