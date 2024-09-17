use derive_builder::Builder;

use crate::constants::{DEFAULT_LFO_AMPLITUDE, SAMPLE_RATE};
use crate::oscillator::{get_note_sample, Waveform};

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct LFO {
    #[builder(default = "SAMPLE_RATE / 10.0", setter(custom))]
    pub(crate) frequency: f32,

    #[builder(default = "DEFAULT_LFO_AMPLITUDE")]
    pub(crate) amplitude: f32,

    // LFO can be a complex combination of waveforms, but we ensure square is not included
    // because it is not a continuous waveform
    #[builder(default = "vec![Waveform::Sine]", setter(custom))]
    pub(crate) waveforms: Vec<Waveform>,
}

#[allow(dead_code)]
impl LFOBuilder {
    pub(crate) fn frequency(&mut self, frequency: f32) -> &mut Self {
        if frequency <= 0.0 {
            panic!("LFO frequency must be greater than 0.0");
        }
        if frequency > SAMPLE_RATE / 2.0 {
            panic!("LFO frequency must be less than the Nyquist frequency");
        }
        self.frequency = Some(frequency);
        self
    }

    pub(crate) fn waveforms(&mut self, waveforms: Vec<Waveform>) -> &mut Self {
        if waveforms.contains(&Waveform::Square) {
            panic!("LFO cannot contain square waveform");
        }
        self.waveforms = Some(waveforms);
        self
    }
}

impl LFO {
    pub(crate) fn apply_effect(&self, sample: f32, sample_clock: f32) -> f32 {
        sample +
            (sample * self.amplitude *
                get_note_sample(&self.waveforms, self.frequency, sample_clock))
    }
}

pub(crate) fn default_lfo() -> LFO {
    LFOBuilder::default().build().unwrap()
}
