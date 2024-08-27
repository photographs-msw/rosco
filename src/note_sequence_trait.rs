use crate::note::Note;

pub(crate) trait AppendNote {
    fn append_note(&mut self, note: Note);
}

pub(crate) trait BuilderWrapper<SequenceType> {
    fn new() -> SequenceType;
}

pub(crate) trait NextNotes {
    fn next_notes(&mut self) -> Vec<Note>;
}

pub(crate) trait CopySequenceNotes {
    fn copy_sequence_notes(&mut self) -> Vec<Vec<Note>>;
}
