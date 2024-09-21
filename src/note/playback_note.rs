use derive_builder::Builder;

use crate::audio_gen::oscillator::Waveform;
use crate::envelope::envelope::Envelope;
use crate::flanger::Flanger;
use crate::lfo::LFO;
use crate::note::note;
use crate::note::note::Note;

#[derive(Builder, Clone, Debug)]
pub(crate) struct PlaybackNote {
    #[builder(default = "note::default_note()")]
    pub(crate) note: Note,

    #[builder(default = "note::INIT_START_TIME")]
    pub(crate) playback_start_time_ms: f32,

    #[builder(default = "note::INIT_END_TIME")]
    pub(crate) playback_end_time_ms: f32,

    #[builder(default = "vec![Waveform::Sine]")]
    pub(crate) waveforms: Vec<Waveform>,

    // Effects loaded from track.effects
    #[builder(default = "Vec::new()")]
    pub(crate) envelopes: Vec<Envelope>,

    #[builder(default = "Vec::new()")]
    pub(crate) lfos: Vec<LFO>,

    #[builder(default = "Vec::new()")]
    pub(crate) flangers: Vec<Flanger>,
}

#[allow(dead_code)]
impl PlaybackNote {
    pub(crate) fn playback_duration_ms(&self) -> f32 {
        self.playback_end_time_ms - self.playback_start_time_ms
    }

    pub(crate) fn apply_effects(&mut self, sample: f32, sample_position: f32) -> f32 {
        let mut output_sample = sample;

        for envelope in self.envelopes.iter() {
            output_sample = envelope.apply_effect(
                output_sample,
                ((self.playback_start_time_ms - self.note.start_time_ms) /
                    (self.note.end_time_ms() - self.note.start_time_ms)) + sample_position
            );
        }

        for lfo in self.lfos.iter() {
            output_sample = lfo.apply_effect(output_sample, sample_position);
        }

        for flanger in self.flangers.iter_mut() {
            output_sample = flanger.apply_effect(output_sample, sample_position);
        }

        output_sample
    }
}

#[allow(dead_code)]
pub(crate) fn default_playback_note() -> PlaybackNote {
    PlaybackNoteBuilder::default().build().unwrap()
}

#[cfg(test)]
mod test_playback_note {
    use crate::audio_gen::oscillator::Waveform;
    use crate::envelope::envelope;
    use crate::flanger;
    use crate::lfo;
    use crate::note::note;
    use crate::note::playback_note::PlaybackNoteBuilder;

    #[test]
    fn test_default_playback_note() {
        let playback_note = PlaybackNoteBuilder::default().build().unwrap();
        assert_eq!(playback_note.note, note::default_note());
        assert_eq!(playback_note.playback_start_time_ms, note::INIT_START_TIME);
        assert_eq!(playback_note.playback_end_time_ms, note::INIT_END_TIME);
        assert_eq!(playback_note.playback_duration_ms(), note::DEFAULT_DURATION);
        assert_eq!(playback_note.waveforms, vec![Waveform::Sine]);
        assert_eq!(playback_note.envelopes.is_empty(), true);
        assert_eq!(playback_note.lfos.is_empty(), true);
        assert_eq!(playback_note.flangers.is_empty(), true);
    }

    #[test]
    fn test_playback_note_with_waveforms() {
        let playback_note = PlaybackNoteBuilder::default()
            .waveforms(vec![Waveform::Saw])
            .build().unwrap();
        assert_eq!(playback_note.waveforms, vec![Waveform::Saw]);
    }

    #[test]
    fn test_playback_note_with_envelope() {
        let playback_note = PlaybackNoteBuilder::default()
            .envelopes(vec![envelope::default_envelope()])
            .build().unwrap();
        assert_eq!(playback_note.envelopes, vec![envelope::default_envelope()]);
    }
    
    #[test]
    fn test_playback_note_with_lfos() {
        let playback_note = PlaybackNoteBuilder::default()
            .lfos(vec![lfo::default_lfo()])
            .build().unwrap();
        assert_eq!(playback_note.lfos, vec![lfo::default_lfo()]);
    }

    #[test]
    fn test_playback_note_with_flangers() {
        let playback_note = PlaybackNoteBuilder::default()
            .flangers(vec![flanger::default_flanger()])
            .build().unwrap();
        assert_eq!(playback_note.flangers, vec![flanger::default_flanger()]);
    }
}
