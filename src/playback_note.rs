use derive_builder::Builder;

use crate::oscillator::Waveform;
use crate::note;
use crate::note::Note;
use crate::sample_effect_trait::{ApplyEffect, BuilderWrapper};

#[derive(Builder, Clone, Debug)]
pub(crate) struct PlaybackNote<EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType>,
        LFOType: ApplyEffect + BuilderWrapper<LFOType>> {
    #[builder(default = "note::default_note()")]
    pub(crate) note: Note,

    #[builder(default = "note::INIT_START_TIME")]
    pub(crate) playback_start_time_ms: f32,

    #[builder(default = "note::INIT_END_TIME")]
    pub (crate) playback_end_time_ms: f32,

    #[builder(default = "None", setter(custom))]
    pub(crate) waveforms: Option<Vec<Waveform>>,

    // So if not set explicitly, envelope is populated with the default of either the
    // NoOpEnvelope or the default Envelope which slightly fades in and out
    #[builder(default = "EnvelopeType::new()")]
    pub(crate) envelope: EnvelopeType,

    // So if not set explicitly, lfos is populated with the either hte NoOpLFO or the default LFO
    // which is a sine wave with a frequency of 1/10th of a second
    #[builder(default = "vec![LFOType::new()]")]
    pub(crate) lfos: Vec<LFOType>,
}

#[allow(dead_code)]
impl<
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType>,
    LFOType: ApplyEffect + BuilderWrapper<LFOType>>
PlaybackNoteBuilder<EnvelopeType, LFOType> {
    
    pub(crate) fn waveforms(&mut self, waveforms: Vec<Waveform>) -> &mut Self {
        self.waveforms = Some(Some(waveforms));
        self
    }
}

#[allow(dead_code)]
impl<
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType>,
    LFOType: ApplyEffect + BuilderWrapper<LFOType>>
PlaybackNote<EnvelopeType, LFOType> {
    pub(crate) fn playback_duration_ms(&self) -> f32 {
        self.playback_end_time_ms - self.playback_start_time_ms
    }

    pub(crate) fn set_waveforms(&mut self, waveforms: Vec<Waveform>) {
        self.waveforms = Some(waveforms);
    }

    pub(crate) fn has_waveforms(&self) -> bool {
        self.waveforms.is_some()
    }

    pub(crate) fn apply_effects(&mut self, sample: f32, freq: f32, sample_clock: f32) -> f32 {
        let mut out_sample = sample;
        
        // Apply each effect in sequence and return the modified sample
        out_sample = self.envelope.apply_effect(out_sample, freq, sample_clock);
        
        self.lfos.iter().for_each(|lfo| {
            out_sample = lfo.apply_effect(out_sample, freq, sample_clock)
        });
        
        out_sample 
    }
}

#[cfg(test)]
mod test_playback_note {
    use crate::lfo::{LFO, LFOBuilder};
    use crate::envelope::{Envelope, EnvelopeBuilder};
    use crate::playback_note::{PlaybackNote, PlaybackNoteBuilder};
    
    #[test]
    fn test_default_playback_note() {
        let playback_note: PlaybackNote<Envelope, LFO> =
            PlaybackNoteBuilder::default().build().unwrap();
        assert_eq!(playback_note.note, crate::note::default_note());
        assert_eq!(playback_note.playback_start_time_ms, crate::note::INIT_START_TIME);
        assert_eq!(playback_note.playback_end_time_ms, crate::note::INIT_END_TIME);
        assert_eq!(playback_note.playback_duration_ms(), crate::note::DEFAULT_DURATION);
        assert_eq!(playback_note.has_waveforms(), false);
        assert_eq!(playback_note.waveforms.is_none(), true);
        assert_eq!(playback_note.envelope, crate::envelope::default_envelope());
        assert_eq!(playback_note.lfos.is_empty(), false);
        assert_eq!(playback_note.lfos.len(), 1);
    }
    
    // #[test]
    // fn test_playback_note_with_envelope() {
    //     let playback_note = PlaybackNoteBuilder::default()
    //         .envelope(crate::envelope::default_envelope())
    //         .build().unwrap();
    //     // assert_eq!(playback_note.has_envelope, true);
    //     assert_eq!(playback_note.envelope.is_some(), true);
    //     assert_eq!(playback_note.has_waveforms, false);
    //     assert_eq!(playback_note.waveforms.is_none(), true);
    // }
    // 
    // #[test]
    // fn test_playback_note_with_waveforms() {
    //     let playback_note = PlaybackNoteBuilder::default()
    //         .waveforms(vec![crate::oscillator::Waveform::Sine])
    //         .build().unwrap();
    //     // assert_eq!(playback_note.has_envelope, false);
    //     assert_eq!(playback_note.envelope.is_none(), true);
    //     assert_eq!(playback_note.has_waveforms, true);
    //     assert_eq!(playback_note.waveforms.is_some(), true);
    // }
    // 
    // #[test]
    // fn test_playback_note_with_lfos() {
    //     let playback_note = PlaybackNoteBuilder::default()
    //         .lfos(vec![LFOBuilder::default().build().unwrap()])
    //         .build().unwrap();
    //     // assert_eq!(playback_note.has_envelope, false);
    //     assert_eq!(playback_note.envelope.is_none(), true);
    //     assert_eq!(playback_note.has_waveforms, false);
    //     assert_eq!(playback_note.waveforms.is_none(), true);
    //     assert_eq!(playback_note.has_lfos, true);
    //     assert_eq!(playback_note.lfos.is_some(), true);
    // }
}
