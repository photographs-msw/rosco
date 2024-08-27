use derive_builder::Builder;

use crate::constants;
use crate::float_utils::{float_eq, float_geq, float_leq};
use crate::note::{Note, NoteBuilder};
use crate::note_sequence_trait::{AppendNote, BuilderWrapper, CopySequenceNotes, NextNotes};

#[allow(dead_code)]
static INIT_START_TIME: f32 = 0.0;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TimeNoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<Note>>,

    #[builder(default = "0.0")]
    next_notes_time_ms: f32,

    // All positions in the grid before this have end times earlier than next_notes_time_ms
    // Allows O(1) access to scan for next notes window vs. always scanning from the beginning
    #[builder(default = "Vec::new()")]
    frontier_indexes: Vec<usize>,
}

impl AppendNote for TimeNoteSequence {
    fn append_note(&mut self, note: Note) {
        self.append_notes(&vec![note]);
    }
}

impl NextNotes for TimeNoteSequence {
    fn next_notes(&mut self) -> Vec<Note> {
        self.get_next_notes_window()
    }
}

impl CopySequenceNotes for TimeNoteSequence {
    fn copy_sequence_notes(&mut self) -> Vec<Vec<Note>> {
        self.sequence.clone()
    }
}

impl BuilderWrapper<TimeNoteSequence> for TimeNoteSequenceBuilder {
    fn new () -> TimeNoteSequence {
        TimeNoteSequenceBuilder::default().build().unwrap()
    }
}

#[allow(dead_code)]
impl TimeNoteSequence {

    pub(crate) fn build_new() -> TimeNoteSequenceBuilder {
        TimeNoteSequenceBuilder::default()
    }

    // Manage Notes
    pub(crate) fn append_notes(&mut self, notes: &Vec<Note>) {
        self.validate_notes_to_add(&notes);

        if self.frontier_indexes.is_empty() {
            self.sequence.push(notes.clone());
            // Went from no indexes with notes to the 0th index now has notes, start of frontier
            self.frontier_indexes.push(0);
            return;
        }

        // Maintain the invariant that all notes with the same start_time are grouped in one
        // note sequence at one index, so if these notes have the same start_time as current
        // notes in last position, add these to that position
        // For the max index in the frontier, check for same start time -- this is the supported
        // semantics for append, which means "add to the end"
        let max_frontier_index = self.frontier_indexes[self.frontier_indexes.len() - 1];
        let min_frontier_start_time_ms = self.get_frontier_min_start_time();
        if float_eq(min_frontier_start_time_ms, notes[0].start_time_ms) {
            self.sequence[max_frontier_index].append(&mut notes.clone());
        } else {
            if min_frontier_start_time_ms > notes[0].start_time_ms {
                panic!("Notes must be appended sorted by start time");
            }
            self.sequence.push(notes.clone());
            self.frontier_indexes.push(max_frontier_index + 1);
        }
    }

    pub(crate) fn append_note(&mut self, note: Note) {
        self.append_notes(&vec![note]);
    }

    pub(crate) fn insert_notes(&mut self, notes: Vec<Note>) {
        self.validate_notes_to_add(&notes);

        // Find insert position where existing notes at position have same start time as notes
        // to insert, or notes at existing position have greater start time as notes to insert
        let mut insert_position: usize = 0;
        let notes_start_time_ms = notes[0].start_time_ms;
        let mut inserted = false;
        while insert_position < self.sequence.len() {
            let min_start_time_ms = self.get_min_start_time(insert_position);

            if float_eq(min_start_time_ms, notes_start_time_ms) {
                self.sequence[insert_position].append(&mut notes.clone());
                inserted = true;
                break;
            }
            if min_start_time_ms > notes_start_time_ms {
                // Move all notes from this position until self.index one position forward
                self.sequence.insert(insert_position, notes.clone());
                inserted = true;
                break;
            }

            insert_position += 1;
        }
        if !inserted {
            self.sequence.push(notes.clone());
            self.frontier_indexes.push(insert_position);
        }
    }

    pub(crate) fn insert_note(&mut self, note: Note) {
        self.insert_notes(vec![note]);
    }

    pub(crate) fn insert_notes_multi_position(&mut self, notes: Vec<Note>) {
        notes.iter().for_each(|note| {self.insert_note(*note);})
    }

    pub(crate) fn get_next_notes_window(&mut self) -> Vec<Note> {

        fn note_ref_into_note( note: &Note, notes_time_ms: f32, end_time_ms: f32) -> Note {
            let mut new_note = note.clone();
            // not start time is current notes_time_ms
            new_note.start_time_ms = notes_time_ms;
            // note end time is the minimum of the end time for all notes and its end time
            let mut note_end_time_ms = note.end_time_ms;
            if notes_time_ms < note_end_time_ms {
                note_end_time_ms = end_time_ms;
            }
            new_note.duration_ms = note_end_time_ms - notes_time_ms;
            new_note
        }
        let mut window_notes = Vec::new();

        self.remove_completed_frontier_indexes(self.next_notes_time_ms);

        let frontier_min_start_time_ms = self.get_frontier_min_start_time();
        // If the current note time is earlier than that, emit a rest note and increment
        // the current notes time to the frontier min start time + epsilon
        if self.next_notes_time_ms < frontier_min_start_time_ms {
            // emit a rest note
            window_notes.push(
                NoteBuilder::default()
                    .frequency(440.0)
                    .volume(0.0)
                    .start_time_ms(self.next_notes_time_ms)
                    .duration_ms(frontier_min_start_time_ms - self.next_notes_time_ms)
                    .end_time_ms()
                    .build().unwrap()
            );

            self.next_notes_time_ms = frontier_min_start_time_ms + constants::FLOAT_EPSILON;
            return window_notes;
        }

        let end_time_ms = self.get_frontier_next_min_end_time(self.next_notes_time_ms);
        // If the current note time is the same as the frontier min start time, emit all notes
        // in the frontier with the same start time and increment the current notes time to the
        // earliest end time in the frontier. This is the next window emit, note to end time.
        if float_eq(self.next_notes_time_ms, frontier_min_start_time_ms) {
            let notes: Vec<Note> = self.get_frontier_notes()
                .iter()
                .flatten()
                .filter(|note| float_eq(note.start_time_ms, self.next_notes_time_ms))
                .map(|note| note_ref_into_note(note, self.next_notes_time_ms, end_time_ms))
                .collect();
            window_notes.append(&mut notes.clone());

            self.next_notes_time_ms = end_time_ms + constants::FLOAT_EPSILON;
        // if notes_time_ms is greater than the frontier min start time, get all notes in the
        // frontier that are playing at the current notes time and emit them up to end time
        // as the next window and increment the current notes time to the end time
        } else if self.next_notes_time_ms > frontier_min_start_time_ms {
            let notes: Vec<Note> = self.get_frontier_notes()
                .iter()
                .flatten()
                .filter(|note|
                        float_leq(note.start_time_ms, self.next_notes_time_ms) &&
                            float_geq(note.end_time_ms, self.next_notes_time_ms))
                .map(|note| note_ref_into_note(note, self.next_notes_time_ms, end_time_ms))
                .filter(|note| note.duration_ms > 0.0)
                .collect();
            window_notes.append(&mut notes.clone());

            self.next_notes_time_ms = end_time_ms + constants::FLOAT_EPSILON;
        } else {
            panic!("Invalid state for next notes window");
        }

        window_notes
    }

    fn get_frontier_notes(&self) ->  &[Vec<Note>] {
        let min_frontier_index = self.frontier_indexes[0];
        let max_frontier_index = self.frontier_indexes[self.frontier_indexes.len() - 1];
        &self.sequence[min_frontier_index..(max_frontier_index + 1)]
    }

    fn remove_completed_frontier_indexes(&mut self, note_time_ms: f32) {
        let mut frontier_indexes_to_remove = Vec::new();
        // Loop over the notes in the position, if any have an end time later than current
        // note time, then the note hasn't been completed yet so the index is still active.
        // OTOH if all notes at an index have end times <= note_time_ms, that index is done
        for i in 0..self.frontier_indexes.len() {
            if self.sequence[self.frontier_indexes[i]].iter().all(
                    |note|
                    float_leq(note.end_time_ms, note_time_ms)) {
                frontier_indexes_to_remove.push(i);
            }
        }
        frontier_indexes_to_remove.iter().for_each(|index| {
            self.frontier_indexes.remove(*index);
        });
    }

    fn get_frontier_min_start_time(&self) -> f32 {
        // Get the earliest start time of all notes in the frontier
        let mut start_time_ms = f32::MAX;
        for note in self.get_frontier_notes().iter().flatten() {
            if note.start_time_ms < start_time_ms {
                start_time_ms = note.start_time_ms;
            }
        }
        start_time_ms
    }

    fn get_min_start_time(&self, index: usize) -> f32 {
        // Get the earliest start time of all notes in the frontier
        let mut min_start_time_ms = f32::MAX;
        for note in self.sequence[index].iter() {
            if note.start_time_ms < min_start_time_ms {
                min_start_time_ms = note.start_time_ms;
            }
        }
        min_start_time_ms
    }

    fn get_frontier_next_min_end_time(&self, note_time_ms: f32) -> f32 {
        let mut end_time_ms = f32::MAX;

        // First pass, is what is the earliest end time in the future, after note_time_ms
        // for a note that starts on or before note_time_ms and ends after it
        for note in self.get_frontier_notes().iter().flatten() {
            if (note.start_time_ms < note_time_ms || float_eq(note.start_time_ms, note_time_ms)) &&
                    note.end_time_ms > note_time_ms &&
                    note.end_time_ms < end_time_ms {
                end_time_ms = note.end_time_ms;
            }
        }

        // Second pass, is there a note that starts after note_time_ms earlier than the
        // earliest end time. Because if there is then that is the end time of this window
        for note in self.get_frontier_notes().iter().flatten() {
            if note.start_time_ms > note_time_ms && note.start_time_ms < end_time_ms {
                end_time_ms = note.start_time_ms;
            }
        }

        end_time_ms
    }

    fn validate_notes_to_add(&self, notes: &Vec<Note>) {
        for note in notes {
            if note.start_time_ms < 0.0 {
                panic!("Note start time must be >= 0.0");
            }
        }
    }

    // #VisibleForTesting
    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<Note> {
        self.sequence[index].clone()
    }
}

// Custom iterator for TrackGrid over the note_windows in the grid
impl<'a> Iterator for TimeNoteSequence {
    type Item = Vec<Note>;

    fn next(&mut self) -> Option<Self::Item> {
        let notes_window = self.get_next_notes_window();
        if notes_window.is_empty() {
            return None;
        }

        Some(notes_window)
    }
}


#[cfg(test)]
mod test_time_note_sequence {
    use crate::float_utils::assert_float_eq;
    use crate::note::NoteBuilder;
    use crate::time_note_sequence::TimeNoteSequenceBuilder;

    #[test]
    fn test_get_next_notes_window() {
        let note_1 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();
        let note_2 = setup_note()
            .start_time_ms(500.0)
            .end_time_ms()
            .build().unwrap();
        let note_3 = setup_note()
            .start_time_ms(1000.0)
            .end_time_ms()
            .build().unwrap();
        let note_4 = setup_note()
            .start_time_ms(1000.0)
            .end_time_ms()
            .build().unwrap();
        let note_5 = setup_note()
            .start_time_ms(2500.0)
            .end_time_ms()
            .build().unwrap();
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();

        sequence.append_note(note_1.clone());
        sequence.append_note(note_2.clone());
        sequence.append_notes(&vec![note_3.clone(), note_4.clone()]);
        sequence.append_note(note_5.clone());

        assert_eq!(sequence.frontier_indexes.len(), 4);
        assert_eq!(sequence.sequence.len(), 4);
        assert_eq!(sequence.sequence[0].len(), 1);
        assert_eq!(sequence.sequence[0][0], note_1);
        assert_eq!(sequence.sequence[1].len(), 1);
        assert_eq!(sequence.sequence[1][0], note_2);
        assert_eq!(sequence.sequence[2].len(), 2);
        assert_eq!(sequence.sequence[2][0], note_3);
        assert_eq!(sequence.sequence[2][1], note_4);
        assert_eq!(sequence.sequence[2].len(), 2);
        assert_eq!(sequence.sequence[3].len(), 1);
        assert_eq!(sequence.sequence[3][0], note_5);

        // 1 start 0 - 500
        let mut notes_window = sequence.get_next_notes_window();
        assert_eq!(notes_window.len(), 1);
        assert_float_eq(notes_window[0].duration_ms, 500.0);
        assert_float_eq(notes_window[0].start_time_ms, 0.0);
        assert_float_eq(notes_window[0].end_time_ms, 1000.0);

        // 1 500 - 1000
        // 2 start 500 - 1000
        notes_window = sequence.get_next_notes_window();
        assert_eq!(notes_window.len(), 2);
        assert_float_eq(notes_window[0].duration_ms, 500.0);
        assert_float_eq(notes_window[0].start_time_ms, 500.0);
        assert_float_eq(notes_window[0].end_time_ms, 1000.0);
        assert_float_eq(notes_window[1].duration_ms, 500.0);
        assert_float_eq(notes_window[1].start_time_ms, 500.0);
        assert_float_eq(notes_window[1].end_time_ms, 1500.0);

        // 2 1000 - 1500
        // 3 start 1000 - 1500
        // 4 start 1000 - 1500
        notes_window = sequence.get_next_notes_window();
        assert_eq!(notes_window.len(), 3);
        assert_float_eq(notes_window[0].duration_ms, 500.0);
        assert_float_eq(notes_window[0].start_time_ms, 1000.0);
        assert_float_eq(notes_window[0].end_time_ms, 1500.0);
        assert_float_eq(notes_window[1].duration_ms, 500.0);
        assert_float_eq(notes_window[1].start_time_ms, 1000.0);
        assert_float_eq(notes_window[1].end_time_ms, 2000.0);
        assert_float_eq(notes_window[2].duration_ms, 500.0);
        assert_float_eq(notes_window[2].start_time_ms, 1000.0);
        assert_float_eq(notes_window[2].end_time_ms, 2000.0);

        // 3 1500 - 2000
        // 4 1500 - 2000
        notes_window = sequence.get_next_notes_window();
        assert_eq!(notes_window.len(), 2);
        assert_float_eq(notes_window[0].duration_ms, 500.0);
        assert_float_eq(notes_window[0].start_time_ms, 1500.0);
        assert_float_eq(notes_window[0].end_time_ms, 2000.0);
        assert_float_eq(notes_window[1].duration_ms, 500.0);
        assert_float_eq(notes_window[1].start_time_ms, 1500.0);
        assert_float_eq(notes_window[1].end_time_ms, 2000.0);
        
        // Rest 2000 - 2500
        notes_window = sequence.get_next_notes_window();
        assert_eq!(notes_window.len(), 1);
        assert_float_eq(notes_window[0].duration_ms, 500.0);
        assert_float_eq(notes_window[0].start_time_ms, 2000.0);
        assert_float_eq(notes_window[0].end_time_ms, 2500.0);
        // 0 volume because it is a rest note
        assert_float_eq(notes_window[0].volume, 0.0);
        
        // 5 start 2500 - 3500
        notes_window = sequence.get_next_notes_window();
        assert_eq!(notes_window.len(), 1);
        assert_float_eq(notes_window[0].duration_ms, 1000.0);
        assert_float_eq(notes_window[0].start_time_ms, 2500.0);
        assert_float_eq(notes_window[0].end_time_ms, 3500.0);
    }

    #[test]
    fn test_insert() {
        let note_1 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();
        let note_2 = setup_note()
            .start_time_ms(500.0)
            .end_time_ms()
            .build().unwrap();
        let note_3 = setup_note()
            .start_time_ms(1000.0)
            .end_time_ms()
            .build().unwrap();
        let note_4 = setup_note()
            .start_time_ms(1000.0)
            .end_time_ms()
            .build().unwrap();
        let note_5 = setup_note()
            .start_time_ms(2500.0)
            .end_time_ms()
            .build().unwrap();
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();
        
        sequence.insert_note(note_5.clone());
        let mut notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_5);

        sequence.insert_note(note_2.clone());
        notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_2);
        notes = sequence.get_notes_at(1);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_5);

        sequence.insert_note(note_3.clone());
        sequence.insert_note(note_4.clone());
        notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_2);
        notes = sequence.get_notes_at(1);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0], note_3);
        assert_eq!(notes[0], note_4);
        
        sequence.insert_note(note_1.clone());
        notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_1);
    }

    #[test]
    fn test_insert_multi_position() {
        let note_1 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();
        let note_2 = setup_note()
            .start_time_ms(500.0)
            .end_time_ms()
            .build().unwrap();
        let note_3 = setup_note()
            .start_time_ms(1000.0)
            .end_time_ms()
            .build().unwrap();
        let note_4 = setup_note()
            .start_time_ms(1000.0)
            .end_time_ms()
            .build().unwrap();
        let note_5 = setup_note()
            .start_time_ms(2500.0)
            .end_time_ms()
            .build().unwrap();
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();

        sequence.insert_notes_multi_position(vec![note_5.clone(), note_2.clone(),
                                                  note_3.clone(), note_4.clone(), note_1.clone()]);
        assert_eq!(sequence.sequence.len(), 4);
        assert_eq!(sequence.sequence[0].len(), 1);
        assert_eq!(sequence.sequence[1].len(), 1);
        assert_eq!(sequence.sequence[2].len(), 2);
        assert_eq!(sequence.sequence[3].len(), 1);

        let mut notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_1);

        notes = sequence.get_notes_at(1);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_2);

        notes = sequence.get_notes_at(2);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0], note_3);
        assert_eq!(notes[1], note_4);

        notes = sequence.get_notes_at(3);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_5);
    }
    
    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .frequency(440.0)
            .duration_ms(1000.0)
            .volume(1.0)
            .clone()
    }
}
