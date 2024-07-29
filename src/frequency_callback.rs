use crate::note::Note;
use crate::oscillator;

pub(crate) trait NoteFrequencyCallback {
    fn get_note_frequency(&self, frequency: f32, sample_clock: f32) -> f32;
}

pub(crate) trait NotesFrequencyCallback {
    fn get_notes_frequency(&self, notes: &Vec<Note>, sample_clock: f32) -> f32;
}

pub(crate) struct InstrumentGetFreqCallback<'a> {
    pub(crate) waveforms: &'a Vec<oscillator::Waveform>,
}

impl NoteFrequencyCallback for InstrumentGetFreqCallback {
    fn get_note_frequency(&self, frequency: f32, sample_clock: f32) -> f32 {
        oscillator::get_note_freq(self.waveforms, frequency, sample_clock)
    }
}

pub(crate) struct MultiInstrumentGetFreqCallback<'a> {
    pub(crate) track_waveforms: &'a Vec<Vec<oscillator::Waveform>>,
}

impl NotesFrequencyCallback for MultiInstrumentGetFreqCallback {
    fn get_notes_frequency(&self, notes: &Vec<Note>, sample_clock: f32) -> f32 {
        oscillator::get_notes_freq(notes, self.track_waveforms, sample_clock)
    }
}
