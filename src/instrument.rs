use crate::audio_gen::AudioGen;
use crate::note::Note;
use crate::oscillator::OscType;
use crate::sequence::Sequence;

#[allow(dead_code)]
pub(crate) struct Instrument<> {
    pub(crate) audio_gen: AudioGen,
    pub(crate) oscillators: Vec<OscType>,
    sequence: Sequence,
}

#[allow(dead_code)]
impl Instrument {

    pub fn from_oscillators(oscillators: Vec<OscType>) -> Self {
        Instrument {
            audio_gen: AudioGen::from_oscillators(oscillators.clone()),
            oscillators,
            sequence: Sequence::new()
        }
    }

    pub fn add_note(&mut self, note: Note) {
        self.sequence.add_note(note);
    }

    pub fn play_note(&self) {
        self.audio_gen.gen_note(&self.sequence.get_note());
    }

    pub fn play_note_and_advance(&mut self) {
        self.audio_gen.gen_note(&self.sequence.get_note_and_advance());
    }

    pub fn loop_once(&self) {
        for note in self.sequence.iter() {
            self.audio_gen.gen_note(&note);
        }
    }

    pub fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.sequence.iter() {
                self.audio_gen.gen_note(&note);
            }
        }
    }

    pub fn play_note_direct(&self, note: &Note) {
        self.audio_gen.gen_note(&note);
    }
}