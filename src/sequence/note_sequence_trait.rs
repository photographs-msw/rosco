use crate::note::playback_note::PlaybackNote;

pub(crate) trait AppendNote {
    fn append_note(&mut self, note: PlaybackNote);
}

pub(crate) trait BuilderWrapper<SequenceType> {
    fn new() -> SequenceType;
}

pub(crate) trait NextNotes {
    fn next_notes(&mut self) -> Vec<PlaybackNote>;
}
