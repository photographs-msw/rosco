use derive_builder::Builder;

use crate::envelope;
use crate::envelope::Envelope;
use crate::oscillator::{LFO, Waveform};
use crate::note;
use crate::note::Note;

#[derive(Builder, Clone, Debug)]
pub(crate) struct PlaybackNote {
    #[builder(default = "note::default_note()")]
    pub(crate) note: Note,

    #[builder(default = "note::INIT_START_TIME")]
    pub(crate) playback_start_time_ms: f32,

    #[builder(default = "note::INIT_END_TIME")]
    pub (crate) playback_end_time_ms: f32,

    #[builder(default = "vec![Waveform::Sine]", setter(custom))]
    pub(crate) waveforms: Vec<Waveform>,

    #[builder(default = "envelope::default_envelope()", setter(custom))]
    pub(crate) envelope: Envelope,

    #[builder(default = "vec![lfo::default_lfo()]", setter(custom))]
    pub(crate) lfos: Vec<LFO>,
}

#[allow(dead_code)]
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
    
    pub(crate) fn lfos(&mut self, lfos: Vec<LFO>) -> &mut Self {
        self.has_lfos = Option::from(true);
        self.lfos = Some(Some(lfos));
        self
    }
}

#[allow(dead_code)]
impl PlaybackNote {
    pub(crate) fn playback_duration_ms(&self) -> f32 {
        self.playback_end_time_ms - self.playback_start_time_ms
    }

    pub(crate) fn set_envelope(&mut self, envelope: Envelope) {
        self.envelope = Some(envelope);
        self.has_envelope = true;
    }

    pub(crate) fn set_waveforms(&mut self, waveforms: Vec<Waveform>) {
        self.waveforms = Some(waveforms);
        self.has_waveforms = true;
    }
    
    pub(crate) fn set_lfos(&mut self, lfos: Vec<LFO>) {
        self.lfos = Some(lfos);
        self.has_lfos = true;
    }
}

#[allow(dead_code)]
pub(crate) fn default_playback_note() -> PlaybackNote {
    PlaybackNoteBuilder::default().build().unwrap()
}

#[cfg(test)]
mod test_playback_note {
    use crate::oscillator::LFOBuilder;
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
        assert_eq!(playback_note.has_lfos, false);
        assert_eq!(playback_note.envelope.is_none(), true);
        assert_eq!(playback_note.waveforms.is_none(), true);
        assert_eq!(playback_note.lfos.is_none(), true);
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
    
    #[test]
    fn test_playback_note_with_lfos() {
        let playback_note = PlaybackNoteBuilder::default()
            .lfos(vec![LFOBuilder::default().build().unwrap()])
            .build().unwrap();
        assert_eq!(playback_note.has_envelope, false);
        assert_eq!(playback_note.envelope.is_none(), true);
        assert_eq!(playback_note.has_waveforms, false);
        assert_eq!(playback_note.waveforms.is_none(), true);
        assert_eq!(playback_note.has_lfos, true);
        assert_eq!(playback_note.lfos.is_some(), true);
    }
}
