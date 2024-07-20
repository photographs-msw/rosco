use crate::audio_gen;
use crate::channel::Channel;
use crate::note::Note;
use crate::oscillator;
use crate::sequence::Sequence;

static DEFAULT_INSTR_VOLUME: f32 = 1.0;

pub(crate) struct Instrument<> {
    oscillators: Vec<oscillator::OscType>,
    channel: Channel,
}

#[allow(dead_code)]
impl Instrument {

    pub(crate) fn from(oscillators: Vec<oscillator::OscType>, sequence: Sequence,
                       volume: f32) -> Self {
        Instrument {
            oscillators,
            channel: Channel::from(sequence, volume)
        }
    }

    pub(crate) fn from_oscillators(oscillators: Vec<oscillator::OscType>) -> Self {
        Instrument {
            oscillators,
            channel: Channel::from(Sequence::new(), DEFAULT_INSTR_VOLUME)
        }
    }

    pub(crate) fn add_note(&mut self, note: Note) {
        self.channel.sequence.add_note(note);
    }

    pub(crate) fn play_note(&self) {
        audio_gen::gen_note(&self.channel.sequence.get_note(), self.oscillators.clone());
    }

    pub(crate) fn play_note_and_advance(&mut self) {
        audio_gen::gen_note(&self.channel.sequence.get_note_and_advance(), self.oscillators.clone());
    }

    pub(crate) fn reset(&mut self) {
        self.channel.sequence.reset_index();
    }

    pub(crate) fn loop_once(&self) {
        for note in self.channel.sequence.iter() {
            audio_gen::gen_note(note, self.oscillators.clone());
        }
    }

    pub(crate) fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.channel.sequence.iter() {
                audio_gen::gen_note(note, self.oscillators.clone());
            }
        }
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.channel.volume = volume;
    }

    pub(crate) fn play_note_direct(&self, note: &Note) {
        audio_gen::gen_note(note, self.oscillators.clone());
    }
}