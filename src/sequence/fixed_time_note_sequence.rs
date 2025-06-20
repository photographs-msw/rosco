use crate::meter::durations::DurationType;
use crate::note::playback_note::PlaybackNote;
use crate::sequence::note_sequence_trait::{AppendNote, AppendNotes, BuilderWrapper, IterMutWrapper, NextNotes, SetCurPosition};
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};

pub(crate) static DEFAULT_DURATION_TYPE: DurationType = DurationType::Quarter;
pub(crate) static DEFAULT_TEMPO: u8 = 120;
pub(crate) static DEFAULT_NUM_STEPS: usize = 16;

// Private helper function to calculate step duration for a duration_type
fn calculate_step_duration_ms(tempo: u8, duration_type: DurationType) -> f32 {
    let quarter_note_duration_ms = 60000.0 / tempo as f32;
    quarter_note_duration_ms * (duration_type.to_factor() / 0.25)
}

/*
A sequence of notes that are played back in fixed time steps. Notes are grouped by step. All notes
added are adjusted to have an end time that matches the step duration. Wraps a TimeNoteSequence
and delegates to it but adds the fixed time step behavior.

NOTE: Doesn't use the derive_builder macros because for whatever reason they didn't play nice with
the computed step_duration_ms field, even with the same pattern followed as Flanger.
 */
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct FixedTimeNoteSequence {
    pub(crate) inner_sequence: TimeNoteSequence,
    pub(crate) duration_type: DurationType,
    pub(crate) tempo: u8,
    pub(crate) num_steps: usize,
    pub(crate) current_step: usize,
    // Computed field based on tempo and duration_type
    pub(crate) step_duration_ms: f32,
}

#[derive(Default)]
pub(crate) struct FixedTimeNoteSequenceBuilder {
    inner_sequence: Option<TimeNoteSequence>,
    duration_type: Option<DurationType>,
    tempo: Option<u8>,
    num_steps: Option<usize>,
    current_step: Option<usize>,
}



#[allow(dead_code)]
impl FixedTimeNoteSequenceBuilder {
    pub(crate) fn inner_sequence(&mut self, inner_sequence: TimeNoteSequence) -> &mut Self {
        self.inner_sequence = Some(inner_sequence);
        self
    }

    pub(crate) fn duration_type(&mut self, duration_type: DurationType) -> &mut Self {
        self.duration_type = Some(duration_type);
        self
    }

    pub(crate) fn tempo(&mut self, tempo: u8) -> &mut Self {
        self.tempo = Some(tempo);
        self
    }

    pub(crate) fn num_steps(&mut self, num_steps: usize) -> &mut Self {
        self.num_steps = Some(num_steps);
        self
    }

    pub(crate) fn current_step(&mut self, current_step: usize) -> &mut Self {
        self.current_step = Some(current_step);
        self
    }

    pub(crate) fn build(&mut self) -> Result<FixedTimeNoteSequence, String> {
        let inner_sequence = self.inner_sequence.take().unwrap_or_else(|| {
            TimeNoteSequenceBuilder::default().build().unwrap()
        });

        let duration_type = self.duration_type.unwrap_or(DEFAULT_DURATION_TYPE);
        let tempo = self.tempo.unwrap_or(DEFAULT_TEMPO);
        let num_steps = self.num_steps.unwrap_or(DEFAULT_NUM_STEPS);
        let current_step = self.current_step.unwrap_or(0);

        // Calculate step_duration_ms using the helper function
        let step_duration_ms = calculate_step_duration_ms(tempo, duration_type);

        Ok(FixedTimeNoteSequence {
            inner_sequence,
            duration_type,
            tempo,
            num_steps,
            current_step,
            step_duration_ms,
        })
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

#[cfg(test)]
mod test_fixed_time_note_sequence {
    use super::*;
    use crate::note::note::NoteBuilder;
    use crate::note::playback_note::{PlaybackNoteBuilder, NoteType};
    use crate::sequence::note_sequence_trait::{AppendNote, AppendNotes, SetCurPosition};

    fn setup_test_note(start_time_ms: f32) -> PlaybackNote {
        PlaybackNoteBuilder::default()
            .note_type(NoteType::Oscillator)
            .note(
                NoteBuilder::default()
                    .start_time_ms(start_time_ms)
                    .end_time_ms(start_time_ms + 100.0) // Default end time, will be overridden
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap()
    }

    #[test]
    fn test_builder_with_defaults() {
        let sequence = FixedTimeNoteSequenceBuilder::default()
            .build()
            .unwrap();

        assert_eq!(sequence.duration_type, DEFAULT_DURATION_TYPE);
        assert_eq!(sequence.tempo, DEFAULT_TEMPO);
        assert_eq!(sequence.num_steps, DEFAULT_NUM_STEPS);
        assert_eq!(sequence.current_step, 0);

        // Check calculated step duration for default quarter note at 120 BPM
        let expected_step_duration = 60000.0 / 120.0; // 500ms for quarter note
        assert_eq!(sequence.step_duration_ms, expected_step_duration);
    }

    #[test]
    fn test_builder_with_custom_values() {
        let sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(140)
            .duration_type(DurationType::Eighth)
            .num_steps(32)
            .current_step(5)
            .build()
            .unwrap();

        assert_eq!(sequence.tempo, 140);
        assert_eq!(sequence.duration_type, DurationType::Eighth);
        assert_eq!(sequence.num_steps, 32);
        assert_eq!(sequence.current_step, 5);

        // Check calculated step duration for eighth note at 140 BPM
        let quarter_note_duration = 60000.0 / 140.0;
        let expected_step_duration = quarter_note_duration * (DurationType::Eighth.to_factor() / DurationType::Quarter.to_factor());
        assert_eq!(sequence.step_duration_ms, expected_step_duration);
    }

    #[test]
    fn test_step_duration_calculation() {
        // Test different tempo and duration combinations
        let sequence_120_quarter = FixedTimeNoteSequenceBuilder::default()
            .tempo(120)
            .duration_type(DurationType::Quarter)
            .build()
            .unwrap();
        assert_eq!(sequence_120_quarter.step_duration_ms, 500.0); // 60000/120 = 500ms

        let sequence_120_eighth = FixedTimeNoteSequenceBuilder::default()
            .tempo(120)
            .duration_type(DurationType::Eighth)
            .build()
            .unwrap();
        assert_eq!(sequence_120_eighth.step_duration_ms, 250.0); // 500 * 0.5 = 250ms

        let sequence_60_quarter = FixedTimeNoteSequenceBuilder::default()
            .tempo(60)
            .duration_type(DurationType::Quarter)
            .build()
            .unwrap();
        assert_eq!(sequence_60_quarter.step_duration_ms, 1000.0); // 60000/60 = 1000ms
    }

    #[test]
    fn test_append_note_sets_end_time() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(120) // 500ms per quarter note
            .build()
            .unwrap();

        let note = setup_test_note(100.0);
        let original_end_time = note.note_end_time_ms();

        sequence.append_note(note.clone());

        // Verify the note in the sequence has the correct end time
        let notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note_start_time_ms(), 100.0);
        assert_eq!(notes[0].note_end_time_ms(), 100.0 + 500.0); // start + step_duration
        assert_ne!(notes[0].note_end_time_ms(), original_end_time);
    }

    #[test]
    fn test_append_notes_sets_end_times() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(120) // 500ms per quarter note
            .build()
            .unwrap();

        let notes = vec![
            setup_test_note(0.0),
            setup_test_note(0.0), // Same start time
        ];

        sequence.append_notes(&notes);

        let stored_notes = sequence.get_notes_at(0);
        assert_eq!(stored_notes.len(), 2);

        for note in stored_notes {
            assert_eq!(note.note_start_time_ms(), 0.0);
            assert_eq!(note.note_end_time_ms(), 500.0); // 0 + 500ms step duration
        }
    }

    #[test]
    fn test_insert_note_sets_end_time() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(60) // 1000ms per quarter note
            .build()
            .unwrap();

        let note = setup_test_note(250.0);
        sequence.insert_note(note);

        let notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note_start_time_ms(), 250.0);
        assert_eq!(notes[0].note_end_time_ms(), 1250.0); // 250 + 1000ms step duration
    }

    #[test]
    fn test_insert_notes_sets_end_times() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(240) // 250ms per quarter note
            .build()
            .unwrap();

        let notes = vec![
            setup_test_note(100.0),
            setup_test_note(100.0),
            setup_test_note(100.0),
        ];

        sequence.insert_notes(notes);

        let stored_notes = sequence.get_notes_at(0);
        assert_eq!(stored_notes.len(), 3);

        for note in stored_notes {
            assert_eq!(note.note_start_time_ms(), 100.0);
            assert_eq!(note.note_end_time_ms(), 350.0); // 100 + 250ms step duration
        }
    }

    #[test]
    fn test_insert_notes_multi_position_sets_end_times() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(120) // 500ms per quarter note
            .build()
            .unwrap();

        let notes = vec![
            setup_test_note(0.0),
            setup_test_note(500.0),
            setup_test_note(1000.0),
        ];

        sequence.insert_notes_multi_position(notes);

        // Should have 3 separate positions since they have different start times
        let notes_at_0 = sequence.get_notes_at(0);
        let notes_at_1 = sequence.get_notes_at(1);
        let notes_at_2 = sequence.get_notes_at(2);

        assert_eq!(notes_at_0.len(), 1);
        assert_eq!(notes_at_0[0].note_start_time_ms(), 0.0);
        assert_eq!(notes_at_0[0].note_end_time_ms(), 500.0);

        assert_eq!(notes_at_1.len(), 1);
        assert_eq!(notes_at_1[0].note_start_time_ms(), 500.0);
        assert_eq!(notes_at_1[0].note_end_time_ms(), 1000.0);

        assert_eq!(notes_at_2.len(), 1);
        assert_eq!(notes_at_2[0].note_start_time_ms(), 1000.0);
        assert_eq!(notes_at_2[0].note_end_time_ms(), 1500.0);
    }

    #[test]
    fn test_trait_implementations() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(120)
            .build()
            .unwrap();

        // Test AppendNote trait
        let note1 = setup_test_note(0.0);
        AppendNote::append_note(&mut sequence, note1);

        // Test AppendNotes trait
        let notes = vec![setup_test_note(0.0)];
        AppendNotes::append_notes(&mut sequence, &notes);

        // Test SetCurPosition trait
        SetCurPosition::set_cur_position(&mut sequence, 100.0);

        // Verify notes were added with correct end times
        let stored_notes = sequence.get_notes_at(0);
        assert_eq!(stored_notes.len(), 2);
        for note in stored_notes {
            assert_eq!(note.note_end_time_ms(), 500.0); // 0 + 500ms step duration
        }
    }

    #[test]
    fn test_step_management_methods() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .num_steps(10)
            .current_step(5)
            .build()
            .unwrap();

        // Test getters
        assert_eq!(sequence.get_current_step(), 5);
        assert_eq!(sequence.get_step_duration_ms(), 500.0); // Default 120 BPM quarter note

        // Test increment
        sequence.increment_step();
        assert_eq!(sequence.get_current_step(), 6);

        // Test decrement
        sequence.decrement_step();
        assert_eq!(sequence.get_current_step(), 5);

        // Test set_step
        sequence.set_step(8);
        assert_eq!(sequence.get_current_step(), 8);

        // Test reset
        sequence.reset_step();
        assert_eq!(sequence.get_current_step(), 0);
    }

    #[test]
    fn test_step_boundaries() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .num_steps(5)
            .current_step(0)
            .build()
            .unwrap();

        // Test decrement at boundary (should not go below 0)
        sequence.decrement_step();
        assert_eq!(sequence.get_current_step(), 0);

        // Move to last step
        sequence.set_step(4);
        assert_eq!(sequence.get_current_step(), 4);

        // Test increment at boundary (should not exceed num_steps - 1)
        sequence.increment_step();
        assert_eq!(sequence.get_current_step(), 4);

        // Test set_step with invalid value (should not change)
        sequence.set_step(10);
        assert_eq!(sequence.get_current_step(), 4);
    }

    #[test]
    fn test_delegation_methods() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .build()
            .unwrap();

        // Add some notes to test delegation
        let note1 = setup_test_note(0.0);
        let note2 = setup_test_note(500.0);
        sequence.append_note(note1);
        sequence.append_note(note2);

        // Test set_cur_position delegation
        sequence.set_cur_position(250.0);

        // Test get_next_notes_window delegation
        let notes_window = sequence.get_next_notes_window();
        assert!(!notes_window.is_empty());

        // Test get_notes_at delegation
        let notes_at_0 = sequence.get_notes_at(0);
        assert_eq!(notes_at_0.len(), 1);
        assert_eq!(notes_at_0[0].note_start_time_ms(), 0.0);
    }

    #[test]
    fn test_iterator_implementation() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .build()
            .unwrap();

        // Add notes at different times
        sequence.append_note(setup_test_note(0.0));
        sequence.append_note(setup_test_note(500.0));
        sequence.append_note(setup_test_note(1000.0));

        // Test iterator
        let mut count = 0;
        for notes_batch in sequence {
            assert!(!notes_batch.is_empty());
            count += 1;
            if count > 10 { // Safety break to avoid infinite loop in tests
                break;
            }
        }
        assert!(count > 0);
    }

    #[test]
    fn test_builder_wrapper_trait() {
        let sequence = FixedTimeNoteSequenceBuilder::new();

        // Should create with default values
        assert_eq!(sequence.duration_type, DEFAULT_DURATION_TYPE);
        assert_eq!(sequence.tempo, DEFAULT_TEMPO);
        assert_eq!(sequence.num_steps, DEFAULT_NUM_STEPS);
        assert_eq!(sequence.current_step, 0);
    }

    #[test]
    fn test_different_duration_types() {
        // Test with different duration types to ensure step duration calculation is correct
        let sequence_whole = FixedTimeNoteSequenceBuilder::default()
            .tempo(60) // 1 second per quarter note
            .duration_type(DurationType::Whole)
            .build()
            .unwrap();
        assert_eq!(sequence_whole.step_duration_ms, 4000.0); // 4 * 1000ms

        let sequence_half = FixedTimeNoteSequenceBuilder::default()
            .tempo(60)
            .duration_type(DurationType::Half)
            .build()
            .unwrap();
        assert_eq!(sequence_half.step_duration_ms, 2000.0); // 2 * 1000ms

        let sequence_sixteenth = FixedTimeNoteSequenceBuilder::default()
            .tempo(60)
            .duration_type(DurationType::Sixteenth)
            .build()
            .unwrap();
        assert_eq!(sequence_sixteenth.step_duration_ms, 250.0); // 0.25 * 1000ms
    }

    #[test]
    fn test_note_end_time_precision() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(133) // Non-round number to test precision
            .build()
            .unwrap();

        let note = setup_test_note(123.456);
        sequence.append_note(note);

        let stored_notes = sequence.get_notes_at(0);
        let expected_step_duration = 60000.0 / 133.0; // ~451.128ms
        let expected_end_time = 123.456 + expected_step_duration;

        assert_eq!(stored_notes[0].note_start_time_ms(), 123.456);
        assert!((stored_notes[0].note_end_time_ms() - expected_end_time).abs() < 0.001);
    }

    #[test]
    fn test_empty_sequence_operations() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .build()
            .unwrap();

        // Test operations on empty sequence
        sequence.set_cur_position(100.0);
        let notes_window = sequence.get_next_notes_window();
        assert!(notes_window.is_empty());

        // Test step operations on empty sequence
        sequence.increment_step();
        sequence.decrement_step();
        sequence.reset_step();
        assert_eq!(sequence.get_current_step(), 0);
    }

    #[test]
    fn test_clone_and_debug() {
        let sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(140)
            .duration_type(DurationType::Eighth)
            .build()
            .unwrap();

        // Test Clone trait
        let cloned_sequence = sequence.clone();
        assert_eq!(cloned_sequence.tempo, sequence.tempo);
        assert_eq!(cloned_sequence.duration_type, sequence.duration_type);
        assert_eq!(cloned_sequence.step_duration_ms, sequence.step_duration_ms);

        // Test Debug trait (should not panic)
        let debug_string = format!("{:?}", sequence);
        assert!(debug_string.contains("FixedTimeNoteSequence"));
    }

    #[test]
    fn test_multiple_notes_same_start_time() {
        let mut sequence = FixedTimeNoteSequenceBuilder::default()
            .tempo(120)
            .build()
            .unwrap();

        // Add multiple notes with the same start time
        let notes = vec![
            setup_test_note(100.0),
            setup_test_note(100.0),
            setup_test_note(100.0),
        ];

        sequence.append_notes(&notes);

        let stored_notes = sequence.get_notes_at(0);
        assert_eq!(stored_notes.len(), 3);

        // All notes should have the same corrected end time
        for note in stored_notes {
            assert_eq!(note.note_start_time_ms(), 100.0);
            assert_eq!(note.note_end_time_ms(), 600.0); // 100 + 500ms step duration
        }
    }
}



