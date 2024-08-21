use derive_builder::Builder;
use float_eq::{float_eq, float_ne};

use crate::constants;
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

#[derive(Builder, Clone, Debug)]
pub(crate) struct NoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<Note>>,
    
    #[builder(default = "0")]
    index: usize
}

impl NoteSequence {
    
    // Manage Notes
    pub(crate) fn append_notes(&mut self, notes: Vec<Note>) {
        self.validate_notes_to_add(&notes);
        
        // Maintain the invariant that all notes with the same start_time are grouped in one
        // note sequence at one index, so if these notes have the same start_time as current
        // notes in last position, add these to that position
        if float_eq!(self.get_notes_start_time(self.index), notes[0].start_time_ms,
                     rmax <= constants::FLOAT_EQ_TOLERANCE) {
            self.sequence[self.index].append(&mut notes.clone());
        } else {
            // Maintain invariant that notes are sorted by start time, so if these notes don't
            // have same start time it must be greater
            if self.max_start_time() > notes[0].start_time_ms { 
                panic!("Notes must be appended sorted by start time");
            }
            self.sequence.push(notes);
            self.index += 1;
        }
    }
    
    pub(crate) fn insert_notes(&mut self, notes: Vec<Note>) {
        self.validate_notes_to_add(&notes);
       
        // Find insert position where existing notes at position have same start time as notes
        // to insert, or notes at existing position have greater start time as notes to insert
        let insert_position: usize = self.index;
        let notes_start_time_ms = notes[0].start_time_ms;
        let mut inserted = false;
        while insert_position < self.sequence.len() {
            if float_eq!(self.get_notes_start_time(insert_position), notes_start_time_ms,
                         rmax <= constants::FLOAT_EQ_TOLERANCE) {
                self.sequence[insert_position].append(&mut notes.clone()); 
                inserted = true;
                break;
            }
            if self.get_notes_start_time(insert_position) > notes_start_time_ms {
                // Move all notes from this position until self.index one position forward
                self.sequence.insert(insert_position, notes.clone());
                self.index += 1;
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.sequence.push(notes.clone());
            self.index += 1;
        }
    }
    
    pub(crate) fn insert_notes_multi_position(&mut self, notes: Vec<Note>) {
        self.validate_notes_to_add(&notes);
        notes.iter().for_each(|note| {self.insert_note_helper(note);})
    }
    
    pub(crate) fn append_note(&mut self, note: Note) {
        self.validate_note_to_add(&note);
        self.append_notes(vec![note]);
    }

    pub(crate) fn insert_note(&mut self, note: Note) {
        self.validate_note_to_add(&note);
        self.insert_note_helper(&note);
    }

    pub(crate) fn max_start_time(&self) -> f32 {
        if self.sequence.is_empty() {
            return 0.0;
        }
        // Because notes are sorted by start time in append(), the last note has the max start time
        self.sequence[self.sequence.len() - 1][0].start_time_ms
    }

    // deprecated because makes no sense once we store a vector at each position
    pub(crate) fn get_note(&self) -> Note {
        self.sequence[self.index][0].clone()
    }
    
    pub(crate) fn get_notes(&self) -> Vec<Note> {
        self.sequence[self.index].clone()
    }

    // deprecated because makes no sense once we store a vector at each position
    pub(crate) fn get_note_at(&self, index: usize) -> Note {
        self.sequence[index][0].clone()
    }

    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<Note> {
        self.sequence[index].clone()
    }

    pub(crate) fn get_note_at_and_advance(&mut self, index: usize) -> Note {
        self.index += 1;
        self.sequence[index][0].clone()
    }

    pub(crate) fn get_notes_at_and_advance(&mut self, index: usize) -> Vec<Note> {
        self.index += 1;
        self.sequence[index].clone()
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
        self.sequence[index][0].start_time_ms
    }

    fn validate_notes_to_add(&self, notes: &Vec<Note>) {
        // validate notes to add
        if notes.is_empty() {
            panic!("Notes to add must not be empty");
        }
        let notes_first_start_time_ms = notes[0].start_time_ms;
        notes.iter().for_each(|note| {
            self.validate_note_to_add(note);
            if float_ne!(notes_first_start_time_ms, note.start_time_ms,
                     rmax <= constants::FLOAT_EQ_TOLERANCE) {
                panic!("Notes added in one append operation must have the same start time");
            }
        });
    }
    
    fn validate_note_to_add(&self, note: &Note) {
        if note.start_time_ms < 0.0 {
            panic!("Note start time must be >= 0.0");
        }
    }

    fn insert_note_helper(&mut self, note: &Note) {
        let insert_position: usize = self.index;
        let notes_start_time_ms = note.start_time_ms;
        let mut inserted = false;
        while insert_position < self.sequence.len() {
            if float_eq!(self.get_notes_start_time(insert_position), notes_start_time_ms,
                         rmax <= constants::FLOAT_EQ_TOLERANCE) {
                self.sequence[insert_position].push(note.clone());
                inserted = true;
                break;
            }
            if self.get_notes_start_time(insert_position) > notes_start_time_ms {
                // Move all notes from this position until self.index one position forward
                self.sequence.insert(insert_position, vec![note.clone()]);
                self.index += 1;
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.sequence.push(vec![note.clone()]);
            self.index += 1;
        }
    }
}
