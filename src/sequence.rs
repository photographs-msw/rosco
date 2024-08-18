use derive_builder::Builder;

use crate::note::Note;

static INIT_INDEX: i64 = 0;

// TODO MAKE THIS GENERIC
#[derive(Builder, Clone, Debug)]
pub struct Sequence {
    #[builder(default = "Vec::new()")]
    pub(crate) notes: Vec<Note>,

    #[builder(default = "INIT_INDEX")]
    index: i64
}

#[allow(dead_code)]
impl Sequence {

    pub(crate) fn get_index(&self) -> i64 {
        self.index
    }

    pub(crate) fn advance(&mut self) {
        self.index += 1;
    }

    pub(crate) fn reset_index(&mut self) {
        self.index = 0;
    }

    pub(crate) fn at_end(&self) -> bool {
        self.index >= 0 && self.index as usize >= self.notes.len()
    }

    pub(crate) fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub(crate) fn get_note(&self) -> Note {
        if self.index < 0 || self.index as usize >= self.notes.len() {
            panic!("Index out of bounds");
        }
        self.notes[self.index as usize].clone()
    }

    pub(crate) fn get_note_and_advance(&mut self) -> Note {
        if self.index < 0 || self.index as usize >= self.notes.len() {
            panic!("Index out of bounds");
        }
        let note = self.notes[self.index as usize].clone();
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
