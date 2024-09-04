use derive_builder::Builder;

use crate::envelope;
use crate::envelope::Envelope;
use crate::oscillator::Waveform;
use crate::note;
use crate::note::Note;
use crate::playback_note_trait::{NoteEnvelope, NoteOscillator};

#[derive(Builder, Clone, Copy, Debug)]
pub(crate) struct PlaybackNote {
    #[builder(default = "note::default_note()")]
    pub(crate) note: Note,

    #[builder(default = "note::INIT_START_TIME")]
    pub(crate) playback_start_time_ms: f32,

    #[builder(default = "note::INIT_END_TIME")]
    pub (crate) playback_end_time_ms: f32,

    #[builder(default = "note::INIT_END_TIME - note::INIT_START_TIME")]
    pub(crate) playback_duration_ms: f32,
}

pub(crate) fn default_playback_note() -> PlaybackNote {
    PlaybackNoteBuilder::default().build().unwrap()
}

#[derive(Clone, Debug)]
pub(crate) enum PlaybackNoteKind {
    Base(PlaybackNote),
    WithOscillator(PlaybackNote, Vec<Waveform>),
    WithEnvelope(PlaybackNote, Envelope),
    WithOscillatorAndEnvelope(PlaybackNote, Vec<Waveform>, Envelope),
}

impl PlaybackNoteKind {
    pub(crate) fn default_base() -> PlaybackNoteKind {
        PlaybackNoteKind::Base(default_playback_note())
    }

    pub(crate) fn default_with_oscillator() -> PlaybackNoteKind {
        PlaybackNoteKind::WithOscillator(default_playback_note(), Vec::new())
    }

    pub(crate) fn default_with_envelope() -> PlaybackNoteKind {
        PlaybackNoteKind::WithEnvelope(default_playback_note(), envelope::default_envelope())
    }

    pub(crate) fn default_with_oscillator_and_envelope() -> PlaybackNoteKind {
        PlaybackNoteKind::WithOscillatorAndEnvelope(default_playback_note(), Vec::new(),
                                                    envelope::default_envelope())
    }
}

impl PlaybackNoteKind {
    pub(crate) fn get_playback_start_time_ms(&self) -> f32 {
        match self {
            PlaybackNoteKind::Base(playback_note) =>
                playback_note.playback_start_time_ms,
            PlaybackNoteKind::WithOscillator(playback_note, _) =>
                playback_note.playback_start_time_ms,
            PlaybackNoteKind::WithEnvelope(playback_note, _) =>
                playback_note.playback_start_time_ms,
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
                playback_note.playback_start_time_ms,
        }
    }

    pub(crate) fn get_note(&self) -> Note {
        match self {
            PlaybackNoteKind::Base(playback_note) =>
                playback_note.note,
            PlaybackNoteKind::WithOscillator(playback_note, _) =>
                playback_note.note,
            PlaybackNoteKind::WithEnvelope(playback_note, _) =>
                playback_note.note,
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
                playback_note.note,
        }
    }

    pub(crate) fn set_playback_start_time_ms(&mut self, start_time_ms: f32) -> f32 {
        match self {
            PlaybackNoteKind::Base(playback_note) => {
                playback_note.playback_start_time_ms = start_time_ms;
                playback_note.playback_start_time_ms
            },
            PlaybackNoteKind::WithOscillator(playback_note, _) => {
                playback_note.playback_start_time_ms = start_time_ms;
                playback_note.playback_start_time_ms
            },
            PlaybackNoteKind::WithEnvelope(playback_note, _) => {
                playback_note.playback_start_time_ms = start_time_ms;
                playback_note.playback_start_time_ms
            },
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) => {
                playback_note.playback_start_time_ms = start_time_ms;
                playback_note.playback_start_time_ms
            },
        }
    }

    pub(crate) fn get_playback_end_time_ms(&self) -> f32 {
        match self {
            PlaybackNoteKind::Base(playback_note) =>
                playback_note.playback_end_time_ms,
            PlaybackNoteKind::WithOscillator(playback_note, _) =>
                playback_note.playback_end_time_ms,
            PlaybackNoteKind::WithEnvelope(playback_note, _) =>
                playback_note.playback_end_time_ms,
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
                playback_note.playback_end_time_ms,
        }
    }

    pub(crate) fn set_playback_end_time_ms(&mut self, end_time_ms: f32) -> f32 {
        match self {
            PlaybackNoteKind::Base(playback_note) => {
                playback_note.playback_end_time_ms = end_time_ms;
                playback_note.playback_end_time_ms
            },
            PlaybackNoteKind::WithOscillator(playback_note, _) => {
                playback_note.playback_end_time_ms = end_time_ms;
                playback_note.playback_end_time_ms
            },
            PlaybackNoteKind::WithEnvelope(playback_note, _) => {
                playback_note.playback_end_time_ms = end_time_ms;
                playback_note.playback_end_time_ms
            },
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) => {
                playback_note.playback_end_time_ms = end_time_ms;
                playback_note.playback_end_time_ms
            },
        }
    }

    pub(crate) fn get_window_duration_ms(&self) -> f32 {
        match self {
            PlaybackNoteKind::Base(playback_note) =>
                playback_note.playback_duration_ms,
            PlaybackNoteKind::WithOscillator(playback_note, _) =>
                playback_note.playback_duration_ms,
            PlaybackNoteKind::WithEnvelope(playback_note, _) =>
                playback_note.playback_duration_ms,
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
                playback_note.playback_duration_ms,
        }
    }
    
    pub(crate) fn set_playback_duration_ms(&mut self) -> f32 {
        match self {
            PlaybackNoteKind::Base(playback_note) => {
                playback_note.playback_duration_ms =
                    playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
                playback_note.playback_duration_ms
            },
            PlaybackNoteKind::WithOscillator(playback_note, _) => {
                playback_note.playback_duration_ms =
                    playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
                playback_note.playback_duration_ms
            }
            PlaybackNoteKind::WithEnvelope(playback_note, _) => {
                playback_note.playback_duration_ms =
                    playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
                playback_note.playback_duration_ms
            } 
            PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) => {
                playback_note.playback_duration_ms =
                    playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
                playback_note.playback_duration_ms
                
            },
        }
    }
}

impl NoteEnvelope for PlaybackNoteKind {
    fn envelope(&self) -> Option<Envelope> {
        match self {
            PlaybackNoteKind::Base(_) => None,
            PlaybackNoteKind::WithOscillator(_, _) => None,
            PlaybackNoteKind::WithEnvelope(_, envelope) => Some(*envelope),
            PlaybackNoteKind::WithOscillatorAndEnvelope(_, _, envelope) => Some(*envelope),
        }
    }

    fn has_envelope(&self) -> bool {
        match self {
            PlaybackNoteKind::Base(_) => false,
            PlaybackNoteKind::WithOscillator(_, _) => false,
            PlaybackNoteKind::WithEnvelope(_, _) => true,
            PlaybackNoteKind::WithOscillatorAndEnvelope(_, _, _) => true,
        }
    }
}

impl NoteOscillator for PlaybackNoteKind {
    fn waveforms(&self) -> Option<Vec<Waveform>> {
        match self {
            PlaybackNoteKind::Base(_) => None,
            PlaybackNoteKind::WithOscillator(_, waveforms) => Some(waveforms.clone()),
            PlaybackNoteKind::WithEnvelope(_, _) => None,
            PlaybackNoteKind::WithOscillatorAndEnvelope(_, waveforms, _) => Some(waveforms.clone()),
        }
    }

    fn has_waveforms(&self) -> bool {
        match self {
            PlaybackNoteKind::Base(_) => false,
            PlaybackNoteKind::WithOscillator(_, _) => true,
            PlaybackNoteKind::WithEnvelope(_, _) => false,
            PlaybackNoteKind::WithOscillatorAndEnvelope(_, _, _) => true,
        }
    }
}
