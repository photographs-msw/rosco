use derive_builder::Builder;
use float_eq::{float_eq, float_ne};

use crate::constants;
use crate::note::{Note, NoteBuilder};

#[allow(dead_code)]
static INIT_START_TIME: f32 = 0.0;

/*
- [ ] Algo starts with Case 1 no active notes at current play time. In this case we look ahead by checking the notes at each track’s note index +1
- [ ] Find the next notes with the earliest start times, increment those track note indexes
- [ ] If there is a gap between current play time and next notes start time, output a rest note from current play time to next note start time and move play time to the next note start time
- [ ] If no gap, then Case 2, active notes at current play time. Action in this case is to check end time of all current notes and look ahead on all other tracks with no notes playing.
- [ ] If they have notes starting before any current notes are ending then that is next end time. In this case output all current notes, increment play time to next note start time, and increment note pointer for these tracks
- [ ] Else if no earlier notes are found on other tracks find the earliest end time of any current active notes. Output this window and move play time to this end time
- [ ] Repeat. On any iteration we either have active notes and determine next end time, or we do not and seek to next start time, issuing a rest for the gap
*/

#[derive(Builder, Clone, Debug)]
pub(crate) struct NoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<Note>>,

    #[builder(default = "0.0")]
    next_notes_time_ms: f32,

    // All positions in the grid before this have end times earlier than next_notes_time_ms
    // Allows O(1) access to scan for next notes window vs. always scanning from the beginning
    #[builder(default = "Vec::new()")]
    frontier_indexes: Vec<usize>,

    // TODO MOVE TO GRID BASED FACADE BEHIND MODULE IN THIS PARENT MODULE
    #[builder(default = "0")]
    index: usize,

    // TODO MOVE TO GRID BASED FACADE BEHIND MODULE IN THIS PARENT MODULE
    // #[builder(default = "0")]
    // next_notes_index: usize,
}

#[allow(dead_code)]
impl NoteSequence {
    
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
        if float_eq!(min_frontier_start_time_ms, notes[0].start_time_ms,
                     rmax <= constants::FLOAT_EPSILON) {
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

            if float_eq!(min_start_time_ms, notes_start_time_ms,
                         rmax <= constants::FLOAT_EPSILON) {
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

    pub(crate) fn get_next_notes_window(&mut self, notes_time_ms: f32) -> Vec<Note> {

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

        // TODO GET THIS WORKING, BUG NOW
        // self.remove_completed_frontier_indexes(notes_time_ms);

        let frontier_min_start_time_ms = self.get_frontier_min_start_time();
        // If the current note time is earlier than that, emit a rest note and increment
        // the current notes time to the frontier min start time + epsilon
        if notes_time_ms < frontier_min_start_time_ms {
            // emit a rest note
            window_notes.push(
                NoteBuilder::default()
                    .frequency(440.0)
                    .volume(0.0)
                    .start_time_ms(notes_time_ms)
                    .duration_ms(frontier_min_start_time_ms - notes_time_ms)
                    .end_time_ms()
                    .build().unwrap()
            );

            self.next_notes_time_ms = frontier_min_start_time_ms + constants::FLOAT_EPSILON;
            return window_notes;
        }

        let end_time_ms = self.get_frontier_next_min_end_time(notes_time_ms);
        // If the current note time is the same as the frontier min start time, emit all notes
        // in the frontier with the same start time and increment the current notes time to the
        // earliest end time in the frontier. This is the next window emit, note to end time.
        if NoteSequence::float_eq(notes_time_ms, frontier_min_start_time_ms) {
            let notes: Vec<Note> = self.get_frontier_notes()
                .iter()
                .flatten()
                .filter(|note| float_eq!(note.start_time_ms, notes_time_ms,
                    rmax <= constants::FLOAT_EPSILON))
                .map(|note| note_ref_into_note(note, notes_time_ms, end_time_ms))
                .collect();
            window_notes.append(&mut notes.clone());

            self.next_notes_time_ms = end_time_ms + constants::FLOAT_EPSILON;
        // if notes_time_ms is greater than the frontier min start time, get all notes in the
        // frontier that are playing at the current notes time and emit them up to end time
        // as the next window and increment the current notes time to the end time
        } else if notes_time_ms > frontier_min_start_time_ms {
            let notes: Vec<Note> = self.get_frontier_notes()
                .iter()
                .flatten()
                .filter(|note|
                        NoteSequence::float_leq(note.start_time_ms, notes_time_ms) &&
                            NoteSequence::float_geq(note.end_time_ms, notes_time_ms))
                .map(|note| note_ref_into_note(note, notes_time_ms, end_time_ms))
                .filter(|note| note.duration_ms > 0.0)
                .collect();
            window_notes.append(&mut notes.clone());

            self.next_notes_time_ms = end_time_ms + constants::FLOAT_EPSILON;
        } else {
            panic!("Invalid state for next notes window");
        }

        window_notes
    }

    // TODO UTILS
    fn float_leq(a: f32, b: f32) -> bool {
        if a < b || float_eq!(a, b, rmax <= constants::FLOAT_EPSILON) {
            return true;
        }
        false
    }

    fn float_geq(a: f32, b: f32) -> bool {
        if a > b || float_eq!(a, b, rmax <= constants::FLOAT_EPSILON) {
            return true;
        }
        false
    }

    fn float_eq(a: f32, b: f32) -> bool {
        float_eq!(a, b, rmax <= constants::FLOAT_EPSILON)
    }

    fn float_neq(a: f32, b: f32) -> bool {
        float_ne!(a, b, rmax <= constants::FLOAT_EPSILON)
    }

    fn get_frontier_notes(&self) ->  &[Vec<Note>] {
        let min_frontier_index = self.frontier_indexes[0];
        let max_frontier_index = self.frontier_indexes[self.frontier_indexes.len() - 1];
        &self.sequence[min_frontier_index..(max_frontier_index + 1)]
    }

    fn remove_completed_frontier_indexes(&mut self, note_time_ms: f32) {
        let mut frontier_indexes_to_remove = Vec::new();
        // Loop oever the notes in the position, if any have an end time later than current
        // note time, then the note hasn't been completed yet so the index is still active.
        // OTOH if all notes at an index have end times <= note_time_ms, that index is done
        for i in 0..self.frontier_indexes.len() {
            for note in self.sequence[self.frontier_indexes[i]].iter() {
                if note.end_time_ms > note_time_ms {
                    continue;
                }
            }
            frontier_indexes_to_remove.push(self.frontier_indexes[i]);
        }
        self.frontier_indexes
            .drain(frontier_indexes_to_remove[0]..frontier_indexes_to_remove[frontier_indexes_to_remove.len() - 1]);
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
            if (note.start_time_ms < note_time_ms ||
                float_eq!(note.start_time_ms, note_time_ms, rmax <= constants::FLOAT_EPSILON)) &&
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

    // fn notes_start_time(&self, index: usize) -> f32 {
    //     if index >= self.sequence.len() {
    //         panic!("Index out of bounds");
    //     }
    //     self.sequence[index][0].start_time_ms
    // }
    //
    //     fn next_notes_start_time(&self) -> f32 {
    //         self.notes_start_time(self.index)
    //     }
    //
    //     fn validate_notes_to_add(&self, notes: &Vec<Note>) {
    //         // validate notes to add
    //         if notes.is_empty() {
    //             panic!("Notes to add must not be empty");
    //         }
    //         let notes_first_start_time_ms = notes[0].start_time_ms;
    //         notes.iter().for_each(|note| {
    //             self.validate_note_to_add(note);
    //             if float_ne!(notes_first_start_time_ms, note.start_time_ms,
    //                      rmax <= constants::FLOAT_EQ_TOLERANCE) {
    //                 panic!("Notes added in one append operation must have the same start time");
    //             }
    //         });
    //     }

//     fn insert_note_helper(&mut self, note: &Note) {
//         let mut insert_position: usize = 0;
//         let notes_start_time_ms = note.start_time_ms;
//         let mut inserted = false;
//         while insert_position < self.sequence.len() {
//             if float_eq!(self.notes_start_time(insert_position), notes_start_time_ms,
//                          rmax <= constants::FLOAT_EQ_TOLERANCE) {
//                 self.sequence[insert_position].push(note.clone());
//                 inserted = true;
//                 break;
//             }
//             if self.notes_start_time(insert_position) > notes_start_time_ms {
//                 // Move all notes from this position until self.index one position forward
//                 self.sequence.insert(insert_position, vec![note.clone()]);
//                 self.index += 1;
//                 inserted = true;
//                 break;
//             }
//
//             insert_position += 1;
//         }
//         if !inserted {
//             self.sequence.push(vec![note.clone()]);
//             self.index += 1;
//         }
//     }
//
//     // ///////////////////////////////////////////
//     // TODO MOVE TO GRID BASED FACADE BEHIND MODULE IN THIS PARENT MODULE
//
//     // Only makes sense with an index and as an internal method
//     // Would be public in a grid- rather than time-based sequencer
//     fn max_start_time(&self) -> f32 {
//         if self.sequence.is_empty() {
//             return 0.0;
//         }
//         // Because notes are sorted by start time in append(), the last note has the max start time
//         self.sequence[self.index][0].start_time_ms
//     }
//
    // deprecated because makes no sense once we store a vector at each position
    pub(crate) fn get_note(&self) -> Note {
        self.sequence[self.index][0].clone()
    }

     // Only makes sense with an index and as an internal method
     // Would be public in a grid- rather than time-based sequencer
     pub(crate) fn get_notes(&self) -> Vec<Note> {
        self.sequence[self.index].clone()
    }

    // deprecated because makes no sense once we store a vector at each position
    pub(crate) fn get_note_at(&self, index: usize) -> Note {
        self.sequence[index][0].clone()
    }

     // Only makes sense with an index and as an internal method
     // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<Note> {
        self.sequence[index].clone()
    }

     // Only makes sense with an index and as an internal method
     // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_note_at_and_advance(&mut self, index: usize) -> Note {
        self.index += 1;
        self.sequence[index][0].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_notes_at_and_advance(&mut self, index: usize) -> Vec<Note> {
        self.index += 1;
        self.sequence[index].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_iter_mut(&mut self) -> std::slice::IterMut<Note> {
        self.sequence[self.index].iter_mut()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_iter(&self) -> std::slice::Iter<Note> {
        self.sequence[self.index].iter()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_len(&self) -> usize {
        self.sequence[self.index].len()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_are_empty(&self) -> bool {
        self.sequence[self.index].is_empty()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_index(&self) -> usize {
        self.index
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn increment(&mut self) {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.index += 1;
    }

     // Only makes sense with an index and as an internal method
     // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn decrement(&mut self) {
        self.index -= 1;
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn reset_index(&mut self) {
        self.index = 0;
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn at_end(&self) -> bool {
        self.index >= self.sequence.len()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_iter_mut(&mut self) -> std::slice::IterMut<Vec<Note>> {
        self.sequence.iter_mut()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_iter(&self) -> std::slice::Iter<Vec<Note>> {
        self.sequence.iter()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_len(&self) -> usize {
        self.sequence.len()
    }
}

#[cfg(test)]
mod test_note_sequence {
    use float_eq::assert_float_eq;
    use crate::constants;
    use crate::note::NoteBuilder;
    use crate::note_sequence::NoteSequenceBuilder;

//     #[test]
//     fn test_append_note() {
//         let note = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//
//         let sequence = NoteSequenceBuilder::default()
//             .sequence(vec![vec![note.clone()]])
//             .index(0)
//             .build().unwrap();
//
//         assert_eq!(sequence.get_notes()[0], note);
//     }
//
//     #[test]
//     fn test_append_notes() {
//         let note_1 = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_2 = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//
//         let sequence = NoteSequenceBuilder::default()
//             .sequence(vec![vec![note_1.clone(), note_2.clone()]])
//             .index(0)
//             .build().unwrap();
//
//         assert_eq!(sequence.get_notes(), vec![note_1, note_2]);
//     }
//
//     #[test]
//     fn test_insert_notes_get_notes_at() {
//         let note_1 = setup_note()
//             .start_time_ms(2.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_2 = setup_note()
//             .start_time_ms(2.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_3 = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_4 = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_5 = setup_note()
//             .start_time_ms(1.0)
//             .end_time_ms()
//             .build().unwrap();
//
//         let mut sequence = NoteSequenceBuilder::default()
//             .index(0)
//             .build().unwrap();
//         sequence.insert_notes(vec![note_1, note_2]);
//         sequence.insert_notes(vec![note_3, note_4]);
//         sequence.insert_notes(vec![note_5]);
//
//         assert_eq!(sequence.get_notes_at(0), vec![note_3, note_4]);
//         assert_eq!(sequence.get_notes_at(1), vec![note_5]);
//         assert_eq!(sequence.get_notes_at(2), vec![note_1, note_2]);
//     }
//
//     #[test]
//     fn test_insert_note_get_note_at() {
//         let note_1 = setup_note()
//             .start_time_ms(1.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_2 = setup_note()
//             .start_time_ms(2.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_3 = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//
//         let mut sequence = NoteSequenceBuilder::default()
//             .index(0)
//             .build().unwrap();
//         sequence.insert_note(note_1);
//         sequence.insert_note(note_2);
//         sequence.insert_note(note_3);
//
//         assert_eq!(sequence.get_note_at(0), note_3);
//         assert_eq!(sequence.get_note_at(1), note_1);
//         assert_eq!(sequence.get_note_at(2), note_2);
//     }
//
//     #[test]
//     fn test_insert_notes_multi_position_get_note_at() {
//         let note_1 = setup_note()
//             .start_time_ms(1.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_2 = setup_note()
//             .start_time_ms(2.0)
//             .end_time_ms()
//             .build().unwrap();
//         let note_3 = setup_note()
//             .start_time_ms(0.0)
//             .end_time_ms()
//             .build().unwrap();
//
//         let mut sequence = NoteSequenceBuilder::default()
//             .index(0)
//             .build().unwrap();
//         sequence.insert_notes_multi_position(vec![note_1, note_2, note_3]);
//
//         assert_eq!(sequence.get_note_at(0), note_3);
//         assert_eq!(sequence.get_note_at(1), note_1);
//         assert_eq!(sequence.get_note_at(2), note_2);
//     }

    // #[test]
    // #[should_panic(expected = "Notes to add must not be empty")]
    // fn test_append_empty_notes() {
    //     let mut sequence = NoteSequenceBuilder::default()
    //         .index(0)
    //         .build().unwrap();
    //
    //     sequence.append_notes(&vec![]);
    // }

    // #[test]
    // #[should_panic(expected = "Notes to add must not be empty")]
    // fn test_insert_empty_notes() {
    //     let mut sequence = NoteSequenceBuilder::default()
    //         .index(0)
    //         .build().unwrap();
    //
    //     sequence.insert_notes(vec![]);
    // }

    // #[test]
    // #[should_panic(expected = "Note start time must be >= 0.0")]
    // fn test_insert_invalid_note() {
    //     let note_1 = setup_note()
    //         .start_time_ms(-1.0)
    //         .end_time_ms()
    //         .build().unwrap();
    //
    //     let mut sequence = NoteSequenceBuilder::default()
    //         .index(0)
    //         .build().unwrap();
    //
    //     sequence.insert_note(note_1);
    // }

    // TODO TEST APPEND SECOND NOTE ON SAME TIME AS AN EXISTING NOTE

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
        let mut sequence = NoteSequenceBuilder::default().build().unwrap();

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

        let mut notes_window = sequence.get_next_notes_window(0.0);
        assert_eq!(notes_window.len(), 1);
        assert_float_eq!(notes_window[0].duration_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[0].start_time_ms, 0.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[0].end_time_ms, 1000.0, rmax <= constants::FLOAT_EPSILON);

        notes_window = sequence.get_next_notes_window(sequence.next_notes_time_ms);
        assert_eq!(notes_window.len(), 2);
        assert_float_eq!(notes_window[0].duration_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[0].start_time_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[0].end_time_ms, 1000.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[1].duration_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[1].start_time_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[1].end_time_ms, 1500.0, rmax <= constants::FLOAT_EPSILON);

        notes_window = sequence.get_next_notes_window(sequence.next_notes_time_ms);
        assert_eq!(notes_window.len(), 3);
        assert_float_eq!(notes_window[0].duration_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[0].start_time_ms, 1000.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[0].end_time_ms, 1500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[1].duration_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[1].start_time_ms, 1000.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[1].end_time_ms, 2000.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[2].duration_ms, 500.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[2].start_time_ms, 1000.0, rmax <= constants::FLOAT_EPSILON);
        assert_float_eq!(notes_window[2].end_time_ms, 2000.0, rmax <= constants::FLOAT_EPSILON);

        // TODO REMOVE COMPLETED FRONTIER INDEXES

        // TODO REST AND THEN SINGLE NOTE
        // notes_window = sequence.get_next_notes_window(sequence.next_notes_time_ms);
        // assert_eq!(notes_window.len(), 1);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .frequency(440.0)
            .duration_ms(1000.0)
            .volume(1.0)
            .clone()
    }
}
