use derive_builder::Builder;

use crate::envelope;
use crate::envelope::Envelope;
use crate::flange;
use crate::flange::Flange;
use crate::lfo;
use crate::lfo::LFO;
use crate::oscillator::Waveform;
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

    #[builder(default = "vec![Waveform::Sine]")]
    pub(crate) waveforms: Vec<Waveform>,

    #[builder(default = "envelope::default_envelope()")]
    pub(crate) envelope: Envelope,

    #[builder(default = "vec![lfo::default_lfo()]")]
    pub(crate) lfos: Vec<LFO>,
    
    #[builder(default = "flange::no_op_flange()")]
    pub(crate) flange: Flange,
}

#[allow(dead_code)]
impl PlaybackNote {
    pub(crate) fn playback_duration_ms(&self) -> f32 {
        self.playback_end_time_ms - self.playback_start_time_ms
    }
    
    pub(crate) fn apply_effects(&mut self, sample: f32, sample_position: f32) -> f32 {
        let mut output_sample = sample;
        
        output_sample = self.envelope.apply_effect(
            output_sample,
            ((self.playback_start_time_ms - self.note.start_time_ms) /
                (self.note.end_time_ms() - self.note.start_time_ms)) + sample_position
        );
                                                      // + (sample_position / 1000.0));
        
        for lfo in self.lfos.iter() {
            output_sample = lfo.apply_effect(output_sample, sample_position);
        }
        
        output_sample = self.flange.apply_effect(output_sample, sample_position);
        
        output_sample
    }
}

#[allow(dead_code)]
pub(crate) fn default_playback_note() -> PlaybackNote {
    PlaybackNoteBuilder::default().build().unwrap()
}

#[cfg(test)]
mod test_playback_note {
    use crate::envelope;
    use crate::lfo;
    use crate::oscillator::Waveform;
    use crate::playback_note::PlaybackNoteBuilder;

    #[test]
    fn test_default_playback_note() {
        let playback_note = PlaybackNoteBuilder::default().build().unwrap();
        assert_eq!(playback_note.note, crate::note::default_note());
        assert_eq!(playback_note.playback_start_time_ms, crate::note::INIT_START_TIME);
        assert_eq!(playback_note.playback_end_time_ms, crate::note::INIT_END_TIME);
        assert_eq!(playback_note.playback_duration_ms(), crate::note::DEFAULT_DURATION);
        assert_eq!(playback_note.envelope, envelope::default_envelope());
        assert_eq!(playback_note.waveforms, vec![Waveform::Sine]);
        assert_eq!(playback_note.lfos, vec![lfo::default_lfo()]);
    }

    #[test]
    fn test_playback_note_with_envelope() {
        let playback_note = PlaybackNoteBuilder::default()
            .envelope(crate::envelope::default_envelope())
            .build().unwrap();
        assert_eq!(playback_note.envelope, envelope::default_envelope());
    }
    
    #[test]
    fn test_playback_note_with_waveforms() {
        let playback_note = PlaybackNoteBuilder::default()
            .waveforms(vec![Waveform::Saw])
            .build().unwrap();
        assert_eq!(playback_note.waveforms, vec![Waveform::Saw]);
    }
    
    #[test]
    fn test_playback_note_with_lfos() {
        let playback_note = PlaybackNoteBuilder::default()
            .lfos(vec![lfo::default_lfo()])
            .build().unwrap();
        assert_eq!(playback_note.lfos, vec![lfo::default_lfo()]);
    }
}
