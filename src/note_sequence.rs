use derive_builder::Builder;

use crate::note::Note;

static INIT_INDEX: usize = 0;

/*
- [ ] Start at index 0 for all tracks
- [ ] All tracks have a dummy first note at 0.0
- [ ] Sequences are a Vev<Vec<Note>> ordered by start time
- [ ] Each position is all notes at that start time, eg a chord
- [ ] Algo starts with Case 1 no active notes at current play time. In this case we look ahead by checking the notes at each track’s note index +1
- [ ] Find the next notes with the earliest start times, increment those track note indexes
- [ ] If there is a gap between current play time and next notes start time, output a rest note from current play time to next note start time and move play time to the next note start time
- [ ] If no gap, then Case 2, active notes at current play time. Action in this case is to check end time of all current notes and look ahead on all other tracks with no notes playing.
- [ ] If they have notes starting before any current notes are ending then that is next end time. In this case output all current notes, increment play time to next note start time, and increment note pointer for these tracks
- [ ] Else if no earlier notes are found on other tracks find the earliest end time of any current active notes. Output this window and move play time to this end time
- [ ] Repeat. On any iteration we either have active notes and determine next end time, or we do not and seek to next start time, issuing a rest for the gap
*/

#[derive(Builder, Clone, Debug)]
pub struct NoteSequence {
    #[builder(default = "Vec::new()")]
    pub(crate) notes: Vec<Note>,

    #[builder(default = "INIT_INDEX")]
    index: usize
}

#[allow(dead_code)]
impl NoteSequence {

    pub(crate) fn get_index(&self) -> usize {
        self.index
    }

    pub(crate) fn advance(&mut self) {
        self.index += 1;
    }

    pub(crate) fn reset_index(&mut self) {
        self.index = 0;
    }

    pub(crate) fn at_end(&self) -> bool {
        self.index >= self.notes.len()
    }

    pub(crate) fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub(crate) fn get_note(&self) -> Note {
        if self.index >= self.notes.len() {
            panic!("Index out of bounds");
        }
        self.notes[self.index].clone()
    }

    pub(crate) fn get_note_and_advance(&mut self) -> Note {
        if self.index >= self.notes.len() {
            panic!("Index out of bounds");
        }
        let note = self.notes[self.index].clone();
        self.advance();
        note
    }

    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<Note> {
        self.notes.iter_mut()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<Note> {
        self.notes.iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.notes.len()
    }
}
