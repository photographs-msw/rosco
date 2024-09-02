use derive_builder::Builder;

use crate::note::Note;
use crate::note_sequence_trait::{AppendNote, BuilderWrapper, NextNotes};

#[derive(Builder, Clone, Debug)]
pub(crate) struct GridNoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<Note>>,

    #[builder(default = "0")]
    index: usize,
}

impl AppendNote for GridNoteSequence {
    fn append_note(&mut self, note: Note) {
        self.sequence.push(vec![note]);
    }
}

impl NextNotes for GridNoteSequence {
    fn next_notes(&mut self) -> Vec<Note> {
        self.get_notes_at_and_advance(self.index)
    }
}

impl BuilderWrapper<GridNoteSequence> for GridNoteSequenceBuilder {
    fn new () -> GridNoteSequence {
        GridNoteSequenceBuilder::default().build().unwrap()
    }
}

#[allow(dead_code)]
impl GridNoteSequence {

    pub(crate) fn append_notes(&mut self, notes: &Vec<Note>) {
        if notes.is_empty() {
            panic!("Notes to add must not be empty");
        }
        self.sequence.push(notes.clone());
    }

    pub(crate) fn insert_notes(&mut self, notes: Vec<Note>) {
        if notes.is_empty() {
            panic!("Notes to add must not be empty");
        }
        if self.index >= self.sequence.len() {
            self.append_notes(&notes);
        }
        self.sequence.insert(self.index, notes);
    }

    pub(crate) fn insert_note(&mut self, note: Note) {
        if note.start_time_ms < 0.0 {
            panic!("Note start time must be >= 0.0");
        }
        self.sequence.insert(self.index, vec![note]);
    }

    pub(crate) fn insert_notes_at(&mut self, notes: Vec<Note>, index: usize) {
        if notes.is_empty() {
            panic!("Notes to add must not be empty");
        }
        if index >= self.sequence.len() {
            self.append_notes(&notes);
            return;
        }
        self.sequence.insert(index, notes);
    }

    pub(crate) fn insert_note_at(&mut self, note: Note, index: usize) {
        if index >= self.sequence.len() {
            self.append_note(note);
            return;
        }
        self.sequence.insert(index, vec![note]);
    }

    pub(crate) fn get_note(&self) -> Note {
        self.sequence[self.index][0].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_notes(&self) -> Vec<Note> {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].clone()
    }

    // deprecated because makes no sense once we store a vector at each position
    pub(crate) fn get_note_at(&self, index: usize) -> Note {
        if index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[index][0].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    // #VisibleForTesting
    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<Note> {
        if index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[index].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_note_at_and_advance(&mut self, index: usize) -> Note {
        if index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.index += 1;
        self.sequence[index][0].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_notes_at_and_advance(&mut self, index: usize) -> Vec<Note> {
        if index >= self.sequence.len() {
            return vec![];
            // panic!("Index out of bounds");
        }
        self.index += 1;
        self.sequence[index].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_iter_mut(&mut self) -> std::slice::IterMut<Note> {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].iter_mut()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_iter(&self) -> std::slice::Iter<Note> {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].iter()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_len(&self) -> usize {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].len()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_are_empty(&self) -> bool {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
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

impl<'a> Iterator for GridNoteSequence {
    type Item = Vec<Note>;

    fn next(&mut self) -> Option<Self::Item> {
        let notes = self.get_notes_at_and_advance(self.index);
        if notes.is_empty() {
            return None;
        }

        Some(notes)
    }
}

#[cfg(test)]
mod test_grid_note_sequence {
    use crate::note::NoteBuilder;
    use crate::float_utils::float_eq;
    use crate::grid_note_sequence::GridNoteSequenceBuilder;

    #[test]
    fn test_append_note() {
        let note = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();

        let sequence = GridNoteSequenceBuilder::default()
            .sequence(vec![vec![note.clone()]])
            .index(0)
            .build().unwrap();

        assert_eq!(sequence.get_notes()[0], note);
    }

    #[test]
    fn test_append_notes() {
        let note_1 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();
        let note_2 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();

        let sequence = GridNoteSequenceBuilder::default()
            .sequence(vec![vec![note_1.clone(), note_2.clone()]])
            .index(0)
            .build().unwrap();

        let actual = sequence.get_notes() ;
        assert_eq!(float_eq(actual[0].start_time_ms, note_1.start_time_ms), true);
        assert_eq!(actual[0], note_1);
        assert_eq!(actual[1], note_2);
    }

    #[test]
    fn test_insert_notes_get_notes_at() {
        let note_1 = setup_note()
            .start_time_ms(2.0)
            .end_time_ms()
            .build().unwrap();
        let note_2 = setup_note()
            .start_time_ms(2.0)
            .end_time_ms()
            .build().unwrap();
        let note_3 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();
        let note_4 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();
        let note_5 = setup_note()
            .start_time_ms(1.0)
            .end_time_ms()
            .build().unwrap();

        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();
        sequence.insert_notes(vec![note_1, note_2]);
        sequence.increment();
        sequence.insert_notes(vec![note_3, note_4]);
        sequence.insert_notes(vec![note_5]);

        assert_eq!(sequence.get_notes_at(0), vec![note_1, note_2]);
        assert_eq!(sequence.get_notes_at(1), vec![note_5]);
        assert_eq!(sequence.get_notes_at(2), vec![note_3, note_4]);
    }

    #[test]
    fn test_insert_note_increment_decrement_get_note_at() {
        let note_1 = setup_note()
            .start_time_ms(1.0)
            .end_time_ms()
            .build().unwrap();
        let note_2 = setup_note()
            .start_time_ms(2.0)
            .end_time_ms()
            .build().unwrap();
        let note_3 = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();

        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();
        sequence.insert_note(note_1);
        sequence.increment();
        sequence.insert_note_at(note_2, 1);
        sequence.decrement();
        sequence.insert_note(note_3);

        // inserted at 0 and pushed to 1 by inserting note_3 at 0
        assert_eq!(sequence.get_note_at(0), note_3);
        // inserted at 1 and pushed to 2 by insert of note_2 at 1 and note_3 at 0
        assert_eq!(sequence.get_note_at(1), note_1);
        // inserted at 1 after note_2
        assert_eq!(sequence.get_note_at(2), note_2);
    }

    #[test]
    #[should_panic(expected = "Notes to add must not be empty")]
    fn test_append_empty_notes() {
        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();

        sequence.append_notes(&vec![]);
    }

    #[test]
    #[should_panic(expected = "Notes to add must not be empty")]
    fn test_insert_empty_notes() {
        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();

        sequence.insert_notes(vec![]);
    }

    #[test]
    #[should_panic(expected = "Note start time must be >= 0.0")]
    fn test_insert_invalid_note() {
        let note_1 = setup_note()
            .start_time_ms(-1.0)
            .end_time_ms()
            .build().unwrap();

        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();

        sequence.insert_note(note_1);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .frequency(440.0)
            .duration_ms(1000.0)
            .volume(1.0)
            .default_envelope()
            .track_num(1)
            .clone()
    }
}
