use crate::audio_gen::gen_note;
use crate::note::Note;
use crate::oscillator::OscType;
use crate::sequence::Sequence;

pub(crate) struct Instrument<> {
    oscillators: Vec<OscType>,
    sequence: Sequence,
}

#[allow(dead_code)]
impl Instrument {

    pub fn from_oscillators(oscillators: Vec<OscType>) -> Self {
        Instrument {
            oscillators,
            sequence: Sequence::new()
        }
    }

    pub fn add_note(&mut self, note: Note) {
        self.sequence.add_note(note);
    }

    pub fn play_note(&self) {
        gen_note(&self.sequence.get_note(), self.oscillators.clone());
    }

    pub fn play_note_and_advance(&mut self) {
        gen_note(&self.sequence.get_note_and_advance(), self.oscillators.clone());
    }

    pub fn reset(&mut self) {
        self.sequence.reset_index();
    }

    pub fn loop_once(&self) {
        for note in self.sequence.iter() {
            gen_note(note, self.oscillators.clone());
        }
    }

    pub fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.sequence.iter() {
                gen_note(note, self.oscillators.clone());
            }
        }
    }

    pub fn play_note_direct(&self, note: &Note) {
        gen_note(note, self.oscillators.clone());
    }
}