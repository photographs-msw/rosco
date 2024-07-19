use crate::audio_gen;
use crate::note::Note;
use crate::oscillator;
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
    channel_oscillators: Vec<Vec<oscillator::OscType>>,
    channels: Vec<Channel>,
}

#[allow(dead_code)]
impl MultiInstrument {

    pub fn from_channel_oscillators(channel_oscillators: Vec<Vec<oscillator::OscType>>) -> Self {
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

    pub(crate) fn play_channel_notes(&self) {
        audio_gen::gen_notes(self.get_next_notes(), self.channel_oscillators.clone());
    }

    pub(crate) fn play_channel_notes_and_advance(&mut self) {
        let notes = self.get_next_notes();
        audio_gen::gen_notes(notes, self.channel_oscillators.clone());
        for channel in self.channels.iter_mut() {
            channel.sequence.advance();
        }
    }

    pub(crate) fn reset_all_channels(&mut self) {
        for channel in &mut self.channels {
            channel.sequence.reset_index();
        }
    }

    pub(crate) fn loop_once(&mut self) {
        self.reset_all_channels();
        while !self.channels.iter().all(|channel| channel.sequence.at_end()) {
            self.play_channel_notes_and_advance();
        }
    }

    pub(crate) fn loop_n(&mut self, n: u8) {
        self.reset_all_channels();
        for _ in 0..n {
            self.loop_once();
        }
    }

    pub(crate) fn add_note_to_channel(&mut self, channel_num: usize, note: Note) {
        self.validate_channel_num(channel_num);

        self.channels[channel_num].sequence.add_note(note);
    }

    pub(crate) fn add_note_to_channels(&mut self, note: Note) {
        self.validate_has_channels();

        for channel in &mut self.channels {
            channel.sequence.add_note(note);
        }
    }

    pub(crate) fn add_chord_to_channels(&mut self, channel_nums: Vec<usize>, chord: Vec<Note>) {
        for channel_num in &channel_nums {
            self.validate_channel_num(*channel_num);
        }
        if channel_nums.len() != chord.len() {
            panic!("Number of channels must match number of notes in chord");
        }
        let first_index: usize = self.channels[channel_nums[0]].sequence.get_index();
        for channel_num in &channel_nums[1..] {
            if self.channels[*channel_num].sequence.get_index() != first_index {
                panic!("Channels must all be at the same index to add chord notes across \
                        channel sequences");
            }
        }

        for (channel_num, note) in channel_nums.iter().zip(chord) {
            self.channels[*channel_num].sequence.add_note(note);
        }
    }

    pub(crate) fn set_volume_for_channels(&mut self, volume: f32) {
        self.validate_has_channels();

        for channel in &mut self.channels {
            channel.volume = volume;
        }
    }

    pub(crate) fn set_volume_for_channel(&mut self, channel_num: usize, volume: f32) {
        self.validate_channel_num(channel_num);

        self.channels[channel_num].volume = volume;
    }

    pub(crate) fn play_notes_direct(&self, notes: Vec<Note>) {
        audio_gen::gen_notes(notes, self.channel_oscillators.clone());
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

    fn validate_channel_num(&self, channel_num: usize) {
        if channel_num >= self.channels.len() {
            panic!("Invalid channel number");
        }
    }

    fn validate_has_channels(&self) {
        if self.channels.len() == 0 {
            panic!("No channels available");
        }
    }
}
