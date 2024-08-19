use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use derive_builder::Builder;

use crate::note::Note;

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

#[derive(Builder, Debug)]
pub(crate) struct NoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<Note>>,
    
    #[builder(default = "0")]
    index: usize
}

impl NoteSequence {
    pub(crate) fn new() -> NoteSequence {
        NoteSequence {
            sequence: Vec::new(),
            index: 0
        }
    }

    // Manage Notes
    pub(crate) fn append(&mut self, notes: Vec<Note>) {
        notes.iter().for_each(|note| {
            if note.start_time_ms < 0.0 {
                panic!("Note start time must be >= 0.0");
            }
        });
        self.sequence.push(notes);
        // Maintain invariant that notes are sorted by start time
        self.sequence.sort_by(
            |a, b| a[0].start_time_ms.partial_cmp(&b[0].start_time_ms)
                .unwrap());
    }
    
    pub(crate) fn max_start_time(&self) -> f32 {
        if self.sequence.is_empty() {
            panic!("No notes in sequence");
        }
        // Because notes are sorted by start time in append(), the last note has the max start time
        self.sequence[self.sequence.len() - 1][0].start_time_ms
    }

    pub(crate) fn get_notes(&self) -> Vec<Note> {
        self.sequence[self.index].clone()
    }

    pub(crate) fn notes_iter_mut(&mut self) -> std::slice::IterMut<Note> {
        self.sequence[self.index].iter_mut()
    }

    pub(crate) fn notes_iter(&self) -> std::slice::Iter<Note> {
        self.sequence[self.index].iter()
    }

    pub(crate) fn notes_len(&self) -> usize {
        self.sequence[self.index].len()
    }

    pub (crate) fn notes_are_empty(&self) -> bool {
        self.sequence[self.index].is_empty()
    }

    // Manage Sequence
    pub(crate) fn get_index(&self) -> usize {
        self.index
    }

    pub(crate) fn increment(&mut self) {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.index += 1;
    }

    pub(crate) fn decrement(&mut self) {
        if self.index - 1 < 0 {
            panic!("Index out of bounds");
        }
        self.index -= 1;
    }

    pub(crate) fn reset_index(&mut self) {
        self.index = 0;
    }

    pub(crate) fn at_end(&self) -> bool {
        self.index >= self.sequence.len()
    }

    pub (crate) fn sequence_is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    pub(crate) fn sequence_iter_mut(&mut self) -> std::slice::IterMut<Vec<Note>> {
        self.sequence.iter_mut()
    }

    pub(crate) fn sequence_iter(&self) -> std::slice::Iter<Vec<Note>> {
        self.sequence.iter()
    }

    pub(crate) fn sequence_len(&self) -> usize {
        self.sequence.len()
    }
    
    fn get_notes_start_time(&self, index: usize) -> f32 {
        if index < 0 || index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        if self.sequence[index].is_empty() {
            panic!("No notes at index");
        }
        self.sequence[index][0].start_time_ms
    }
}
