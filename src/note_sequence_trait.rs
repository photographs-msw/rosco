use crate::note::Note;

pub(crate) trait AppendNote {
    fn append_note(&mut self, note: Note);
}

pub(crate) trait BuilderWrapper<SequenceType> {
    fn new() -> SequenceType; 
}
