use derive_builder::Builder;

use crate::constants::NO_TRACK;
use crate::playback_note;
use crate::playback_note::PlaybackNoteKind;

static DEFAULT_TRACK_VOLUME: f32 = 1.0;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
pub(crate) struct Track<SequenceType> {
    #[builder(default = "NO_TRACK")]
    pub(crate) num: i16,

    #[builder(default = "DEFAULT_TRACK_VOLUME")]
    pub(crate) volume: f32,

    pub(crate) sequence: SequenceType,

    #[builder(public, setter(custom))]
    pub(crate) playback_note_kind: PlaybackNoteKind,
}

impl<SequenceType> TrackBuilder<SequenceType> { 
    pub(crate) fn playback_note_kind(&mut self, playback_note_kind: PlaybackNoteKind) -> &mut Self { 
        self.playback_note_kind = Some(playback_note_kind);
        self
    }
    
    pub(crate) fn default_playback_note_kind(&mut self) -> &mut Self { 
        self.playback_note_kind =
            Some(PlaybackNoteKind::Base(playback_note::default_playback_note()));
        self
    }
}

impl<SequenceType> Track<SequenceType> {}
