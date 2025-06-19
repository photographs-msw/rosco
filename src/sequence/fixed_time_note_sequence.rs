use derive_builder::Builder;
use crate::meter::durations::DurationType;
use crate::note::playback_note::PlaybackNote;
use crate::sequence::note_sequence_trait::{AppendNote, AppendNotes, BuilderWrapper, IterMutWrapper, NextNotes, SetCurPosition};
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};

pub(crate) static DEFAULT_DURATION_TYPE: DurationType = DurationType::Quarter;
pub(crate) static DEFAULT_TEMPO: u8 = 120;
pub(crate) static DEFAULT_NUM_STEPS: usize = 16;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
pub(crate) struct FixedTimeNoteSequence {
    #[builder(default = "TimeNoteSequenceBuilder::default().build().unwrap()")]
    inner_sequence: TimeNoteSequence,

    #[builder(default = "DEFAULT_DURATION_TYPE")]
    duration_type: DurationType,

    #[builder(default = "DEFAULT_TEMPO")]
    tempo: u8,

    #[builder(default = "DEFAULT_NUM_STEPS")]
    num_steps: usize,

    #[builder(default = "0")]
    current_step: usize,

    // This is a computed field, so we don't want a setter for it
    #[builder(setter(custom))]
    step_duration_ms: f32,
}

// #[derive(Default)]
// pub(crate) struct FixedTimeNoteSequenceBuilder {
//     inner_sequence: Option<TimeNoteSequence>,
//     duration_type: Option<DurationType>,
//     tempo: Option<u8>,
//     num_steps: Option<usize>,
//     current_step: Option<usize>,
// }

#[allow(dead_code)]
impl FixedTimeNoteSequenceBuilder {
    pub(crate) fn step_duration_ms(&mut self) -> &mut Self  {
        let duration_type = self.duration_type.unwrap_or(DEFAULT_DURATION_TYPE);
        let tempo = self.tempo.unwrap_or(DEFAULT_TEMPO);

        // Calculate step_duration_ms directly here
        let quarter_note_duration_ms = 60000.0 / tempo as f32;
        self.step_duration_ms = Some(
            quarter_note_duration_ms *
                (duration_type.to_factor() / DEFAULT_DURATION_TYPE.to_factor()) 
        );
        
        self
    }
}

// Helper function to set note end time based on step duration
fn set_note_end_time_for_step_duration(note: &mut PlaybackNote, step_duration_ms: f32) {
    let start_time = note.note_start_time_ms();
    note.set_note_end_time_ms(start_time + step_duration_ms);
}

// Trait implementations for FixedTimeNoteSequence
impl AppendNote for FixedTimeNoteSequence {
    fn append_note(&mut self, mut note: PlaybackNote) {
        set_note_end_time_for_step_duration(&mut note, self.step_duration_ms);
        self.inner_sequence.append_note(note);
    }
}

impl AppendNotes for FixedTimeNoteSequence {
    fn append_notes(&mut self, notes: &Vec<PlaybackNote>) {
        let mut modified_notes = notes.clone();
        for note in modified_notes.iter_mut() {
            set_note_end_time_for_step_duration(note, self.step_duration_ms);
        }
        self.inner_sequence.append_notes(&modified_notes);
    }
}

impl NextNotes for FixedTimeNoteSequence {
    fn next_notes(&mut self) -> Vec<PlaybackNote> {
        self.inner_sequence.next_notes()
    }
}

impl SetCurPosition for FixedTimeNoteSequence {
    fn set_cur_position(&mut self, position: f32) {
        self.inner_sequence.set_cur_position(position);
    }
}

impl IterMutWrapper for FixedTimeNoteSequence {
    fn iter_mut(&mut self) -> std::slice::IterMut<Vec<PlaybackNote>> {
        self.inner_sequence.iter_mut()
    }
}

impl BuilderWrapper<FixedTimeNoteSequence> for FixedTimeNoteSequenceBuilder {
    fn new() -> FixedTimeNoteSequence {
        FixedTimeNoteSequenceBuilder::default().build().unwrap()
    }
}

#[allow(dead_code)]
impl FixedTimeNoteSequence {
    // Wrapper methods that delegate to inner_sequence with step duration adjustment

    pub(crate) fn append_notes(&mut self, notes: &Vec<PlaybackNote>) {
        let mut modified_notes = notes.clone();
        for note in modified_notes.iter_mut() {
            set_note_end_time_for_step_duration(note, self.step_duration_ms);
        }
        self.inner_sequence.append_notes(&modified_notes);
    }

    pub(crate) fn append_note(&mut self, mut note: PlaybackNote) {
        set_note_end_time_for_step_duration(&mut note, self.step_duration_ms);
        self.inner_sequence.append_note(note);
    }

    pub(crate) fn insert_notes(&mut self, notes: Vec<PlaybackNote>) {
        let mut modified_notes = notes;
        for note in modified_notes.iter_mut() {
            set_note_end_time_for_step_duration(note, self.step_duration_ms);
        }
        self.inner_sequence.insert_notes(modified_notes);
    }

    pub(crate) fn insert_note(&mut self, mut note: PlaybackNote) {
        set_note_end_time_for_step_duration(&mut note, self.step_duration_ms);
        self.inner_sequence.insert_note(note);
    }

    pub(crate) fn insert_notes_multi_position(&mut self, notes: Vec<PlaybackNote>) {
        let mut modified_notes = notes;
        for note in modified_notes.iter_mut() {
            set_note_end_time_for_step_duration(note, self.step_duration_ms);
        }
        self.inner_sequence.insert_notes_multi_position(modified_notes);
    }

    // Delegate methods that don't take PlaybackNote arguments (no modification needed)
    pub(crate) fn get_next_notes_window(&mut self) -> Vec<PlaybackNote> {
        self.inner_sequence.get_next_notes_window()
    }

    pub(crate) fn set_cur_position(&mut self, position: f32) {
        self.inner_sequence.set_cur_position(position);
    }

    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<PlaybackNote> {
        self.inner_sequence.get_notes_at(index)
    }

    // Additional utility methods specific to FixedTimeNoteSequence
    pub(crate) fn get_step_duration_ms(&self) -> f32 {
        self.step_duration_ms
    }

    pub(crate) fn get_current_step(&self) -> usize {
        self.current_step
    }

    pub(crate) fn increment_step(&mut self) {
        if self.current_step < self.num_steps - 1 {
            self.current_step += 1;
        }
    }

    pub(crate) fn decrement_step(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
    }

    pub(crate) fn reset_step(&mut self) {
        self.current_step = 0;
    }

    pub(crate) fn set_step(&mut self, step: usize) {
        if step < self.num_steps {
            self.current_step = step;
        }
    }
}

// Iterator implementation
impl Iterator for FixedTimeNoteSequence {
    type Item = Vec<PlaybackNote>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_sequence.next()
    }
}

