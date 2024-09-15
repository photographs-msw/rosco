use derive_builder::Builder;

use crate::envelope;
use crate::envelope::Envelope;
use crate::oscillator::Waveform;
use crate::note;
use crate::note::Note;
use crate::playback_note_trait::{NoteEnvelope, NoteOscillator};

#[derive(Builder, Clone, Debug)]
pub(crate) struct PlaybackNote {
    #[builder(default = "note::default_note()")]
    pub(crate) note: Note,

    #[builder(default = "note::INIT_START_TIME")]
    pub(crate) playback_start_time_ms: f32,

    #[builder(default = "note::INIT_END_TIME")]
    pub (crate) playback_end_time_ms: f32,

    #[builder(default = "false")]
    pub(crate) has_envelope: bool,
    #[builder(default = "None", setter(custom))]
    pub(crate) envelope: Option<Envelope>,

    #[builder(default = "false")]
    pub(crate) has_waveforms: bool,
    #[builder(default = "None", setter(custom))]
    pub(crate) waveforms: Option<Vec<Waveform>>,
}

impl PlaybackNoteBuilder {
    pub(crate) fn envelope(&mut self, envelope: Envelope) -> &mut Self {
        self.has_envelope = Option::from(true);
        self.envelope = Some(Some(envelope));
        self
    }

    pub(crate) fn waveforms(&mut self, waveforms: Vec<Waveform>) -> &mut Self {
        self.has_waveforms = Option::from(true);
        self.waveforms = Some(Some(waveforms));
        self
    }
}

impl PlaybackNote {
    pub(crate) fn playback_duration_ms(&self) -> f32 {
        self.playback_end_time_ms - self.playback_start_time_ms
    }
    
    pub(crate) fn set_waveforms(&mut self, waveforms: Vec<Waveform>) {
        self.waveforms = Some(waveforms);
        self.has_waveforms = true;
    }
    
    pub(crate) fn set_envelope(&mut self, envelope: Envelope) {
        self.envelope = Some(envelope);
        self.has_envelope = true;
    }
}

#[allow(dead_code)]
pub(crate) fn default_playback_note() -> PlaybackNote {
    PlaybackNoteBuilder::default().build().unwrap()
}

#[cfg(test)]
mod test_playback_note {
    use crate::playback_note::PlaybackNoteBuilder;

    #[test]
    fn test_default_playback_note() {
        let playback_note = PlaybackNoteBuilder::default().build().unwrap();
        assert_eq!(playback_note.note, crate::note::default_note());
        assert_eq!(playback_note.playback_start_time_ms, crate::note::INIT_START_TIME);
        assert_eq!(playback_note.playback_end_time_ms, crate::note::INIT_END_TIME);
        assert_eq!(playback_note.playback_duration_ms(), crate::note::DEFAULT_DURATION);
        assert_eq!(playback_note.has_envelope, false);
        assert_eq!(playback_note.has_waveforms, false);
        assert_eq!(playback_note.envelope.is_none(), true);
        assert_eq!(playback_note.waveforms.is_none(), true);
    }

    #[test]
    fn test_playback_note_with_envelope() {
        let playback_note = PlaybackNoteBuilder::default()
            .envelope(crate::envelope::default_envelope())
            .build().unwrap();
        assert_eq!(playback_note.has_envelope, true);
        assert_eq!(playback_note.envelope.is_some(), true);
        assert_eq!(playback_note.has_waveforms, false);
        assert_eq!(playback_note.waveforms.is_none(), true);
    }
    
    #[test]
    fn test_playback_note_with_waveforms() {
        let playback_note = PlaybackNoteBuilder::default()
            .waveforms(vec![crate::oscillator::Waveform::Sine])
            .build().unwrap();
        assert_eq!(playback_note.has_envelope, false);
        assert_eq!(playback_note.envelope.is_none(), true);
        assert_eq!(playback_note.has_waveforms, true);
        assert_eq!(playback_note.waveforms.is_some(), true);
    }
}

// #[allow(dead_code)]
// #[derive(Clone, Debug)]
// pub(crate) enum PlaybackNoteKind {
//     Base(PlaybackNote),
//     WithOscillator(PlaybackNote, Vec<Waveform>),
//     WithEnvelope(PlaybackNote, Envelope),
//     WithOscillatorAndEnvelope(PlaybackNote, Vec<Waveform>, Envelope),
// }
// 
// #[allow(dead_code)]
// impl PlaybackNoteKind {
//     pub(crate) fn default_base() -> PlaybackNoteKind {
//         PlaybackNoteKind::Base(default_playback_note())
//     }
// 
//     pub(crate) fn default_with_oscillator() -> PlaybackNoteKind {
//         PlaybackNoteKind::WithOscillator(default_playback_note(), Vec::new())
//     }
// 
//     pub(crate) fn default_with_envelope() -> PlaybackNoteKind {
//         PlaybackNoteKind::WithEnvelope(default_playback_note(), envelope::default_envelope())
//     }
// 
//     pub(crate) fn default_with_oscillator_and_envelope() -> PlaybackNoteKind {
//         PlaybackNoteKind::WithOscillatorAndEnvelope(default_playback_note(), Vec::new(),
//                                                     envelope::default_envelope())
//     }
// }
// 
// #[allow(dead_code)]
// impl PlaybackNoteKind {
//     pub(crate) fn get_playback_start_time_ms(&self) -> f32 {
//         match self {
//             PlaybackNoteKind::Base(playback_note) =>
//                 playback_note.playback_start_time_ms,
//             PlaybackNoteKind::WithOscillator(playback_note, _) =>
//                 playback_note.playback_start_time_ms,
//             PlaybackNoteKind::WithEnvelope(playback_note, _) =>
//                 playback_note.playback_start_time_ms,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
//                 playback_note.playback_start_time_ms,
//         }
//     }
// 
//     pub(crate) fn get_note(&self) -> Note {
//         match self {
//             PlaybackNoteKind::Base(playback_note) =>
//                 playback_note.note,
//             PlaybackNoteKind::WithOscillator(playback_note, _) =>
//                 playback_note.note,
//             PlaybackNoteKind::WithEnvelope(playback_note, _) =>
//                 playback_note.note,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
//                 playback_note.note,
//         }
//     }
// 
//     pub(crate) fn set_playback_start_time_ms(&mut self, start_time_ms: f32) -> f32 {
//         match self {
//             PlaybackNoteKind::Base(playback_note) => {
//                 playback_note.playback_start_time_ms = start_time_ms;
//                 playback_note.playback_start_time_ms
//             },
//             PlaybackNoteKind::WithOscillator(playback_note, _) => {
//                 playback_note.playback_start_time_ms = start_time_ms;
//                 playback_note.playback_start_time_ms
//             },
//             PlaybackNoteKind::WithEnvelope(playback_note, _) => {
//                 playback_note.playback_start_time_ms = start_time_ms;
//                 playback_note.playback_start_time_ms
//             },
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) => {
//                 playback_note.playback_start_time_ms = start_time_ms;
//                 playback_note.playback_start_time_ms
//             },
//         }
//     }
// 
//     pub(crate) fn get_playback_end_time_ms(&self) -> f32 {
//         match self {
//             PlaybackNoteKind::Base(playback_note) =>
//                 playback_note.playback_end_time_ms,
//             PlaybackNoteKind::WithOscillator(playback_note, _) =>
//                 playback_note.playback_end_time_ms,
//             PlaybackNoteKind::WithEnvelope(playback_note, _) =>
//                 playback_note.playback_end_time_ms,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
//                 playback_note.playback_end_time_ms,
//         }
//     }
// 
//     pub(crate) fn set_playback_end_time_ms(&mut self, end_time_ms: f32) -> f32 {
//         match self {
//             PlaybackNoteKind::Base(playback_note) => {
//                 playback_note.playback_end_time_ms = end_time_ms;
//                 playback_note.playback_end_time_ms
//             },
//             PlaybackNoteKind::WithOscillator(playback_note, _) => {
//                 playback_note.playback_end_time_ms = end_time_ms;
//                 playback_note.playback_end_time_ms
//             },
//             PlaybackNoteKind::WithEnvelope(playback_note, _) => {
//                 playback_note.playback_end_time_ms = end_time_ms;
//                 playback_note.playback_end_time_ms
//             },
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) => {
//                 playback_note.playback_end_time_ms = end_time_ms;
//                 playback_note.playback_end_time_ms
//             },
//         }
//     }
// 
//     pub(crate) fn get_window_duration_ms(&self) -> f32 {
//         match self {
//             PlaybackNoteKind::Base(playback_note) =>
//                 playback_note.playback_duration_ms,
//             PlaybackNoteKind::WithOscillator(playback_note, _) =>
//                 playback_note.playback_duration_ms,
//             PlaybackNoteKind::WithEnvelope(playback_note, _) =>
//                 playback_note.playback_duration_ms,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) =>
//                 playback_note.playback_duration_ms,
//         }
//     }
//     
//     pub(crate) fn set_playback_duration_ms(&mut self) -> f32 {
//         match self {
//             PlaybackNoteKind::Base(playback_note) => {
//                 playback_note.playback_duration_ms =
//                     playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
//                 playback_note.playback_duration_ms
//             },
//             PlaybackNoteKind::WithOscillator(playback_note, _) => {
//                 playback_note.playback_duration_ms =
//                     playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
//                 playback_note.playback_duration_ms
//             }
//             PlaybackNoteKind::WithEnvelope(playback_note, _) => {
//                 playback_note.playback_duration_ms =
//                     playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
//                 playback_note.playback_duration_ms
//             } 
//             PlaybackNoteKind::WithOscillatorAndEnvelope(playback_note, _, _) => {
//                 playback_note.playback_duration_ms =
//                     playback_note.playback_end_time_ms - playback_note.playback_start_time_ms;
//                 playback_note.playback_duration_ms
//                 
//             },
//         }
//     }
// }
// 
// impl NoteEnvelope for PlaybackNoteKind {
//     fn envelope(&self) -> Option<Envelope> {
//         match self {
//             PlaybackNoteKind::Base(_) => None,
//             PlaybackNoteKind::WithOscillator(_, _) => None,
//             PlaybackNoteKind::WithEnvelope(_, envelope) => Some(*envelope),
//             PlaybackNoteKind::WithOscillatorAndEnvelope(_, _, envelope) => Some(*envelope),
//         }
//     }
// 
//     fn has_envelope(&self) -> bool {
//         match self {
//             PlaybackNoteKind::Base(_) => false,
//             PlaybackNoteKind::WithOscillator(_, _) => false,
//             PlaybackNoteKind::WithEnvelope(_, _) => true,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(_, _, _) => true,
//         }
//     }
// }
// 
// impl NoteOscillator for PlaybackNoteKind {
//     fn waveforms(&self) -> Option<Vec<Waveform>> {
//         match self {
//             PlaybackNoteKind::Base(_) => None,
//             PlaybackNoteKind::WithOscillator(_, waveforms) => Some(waveforms.clone()),
//             PlaybackNoteKind::WithEnvelope(_, _) => None,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(_, waveforms, _) => Some(waveforms.clone()),
//         }
//     }
// 
//     fn has_waveforms(&self) -> bool {
//         match self {
//             PlaybackNoteKind::Base(_) => false,
//             PlaybackNoteKind::WithOscillator(_, _) => true,
//             PlaybackNoteKind::WithEnvelope(_, _) => false,
//             PlaybackNoteKind::WithOscillatorAndEnvelope(_, _, _) => true,
//         }
//     }
// }
