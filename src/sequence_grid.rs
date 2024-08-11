use crate::note::Note;
use crate::oscillator;
use crate::sequence::Sequence;

pub(crate) struct SequenceGrid {
    pub(crate) sequences: Vec<Sequence>,
    sample_clock_index: f32,
}

#[allow(dead_code)]
impl SequenceGrid {
    pub(crate) fn get_sample_rate_index(&self) -> f32 {
        self.sample_clock_index % oscillator::SAMPLE_RATE
    }

    pub(crate) fn get_current_time_ms(&self) -> f32 {
        self.sample_clock_index / oscillator::SAMPLE_RATE
    }

    pub(crate) fn active_notes(&mut self) -> Vec<Note> {
        let cur_time_ms = self.get_current_time_ms();
        let mut active_notes = Vec::new();
        for sequence in &mut self.sequences.iter_mut() {
            for note in sequence.iter_mut()  {
                if note.is_playing(cur_time_ms) {
                    note.cur_playing_time_ms(cur_time_ms);
                    active_notes.push(note.clone());
                }
            }
        }
        active_notes
    }
}