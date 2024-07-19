use crate::audio_gen;
use crate::note::Note;
use crate::oscillator;
use crate::sequence::Sequence;

pub(crate) struct Instrument<> {
    oscillators: Vec<oscillator::OscType>,
    sequence: Sequence,
}

#[allow(dead_code)]
impl Instrument {

    pub(crate) fn from_oscillators(oscillators: Vec<oscillator::OscType>) -> Self {
        Instrument {
            oscillators,
            sequence: Sequence::new()
        }
    }

    pub(crate) fn add_note(&mut self, note: Note) {
        self.sequence.add_note(note);
    }

    pub(crate) fn play_note(&self) {
        audio_gen::gen_note(&self.sequence.get_note(), self.oscillators.clone());
    }

    pub(crate) fn play_note_and_advance(&mut self) {
        audio_gen::gen_note(&self.sequence.get_note_and_advance(), self.oscillators.clone());
    }

    pub(crate) fn reset(&mut self) {
        self.sequence.reset_index();
    }

    pub(crate) fn loop_once(&self) {
        for note in self.sequence.iter() {
            audio_gen::gen_note(note, self.oscillators.clone());
        }
    }

    pub(crate) fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.sequence.iter() {
                audio_gen::gen_note(note, self.oscillators.clone());
            }
        }
    }

    pub(crate) fn play_note_direct(&self, note: &Note) {
        audio_gen::gen_note(note, self.oscillators.clone());
    }
}