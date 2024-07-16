use crate::audio_gen::AudioGen;
use crate::oscillator::OscType;

pub(crate) struct Instrument<> {
    audio_gen: AudioGen,
}

impl Instrument {

    pub fn from_oscillators(oscillators: Vec<OscType>) -> Self {
        Instrument {
           audio_gen: AudioGen::from_oscillators(oscillators),
        }
    }

    pub fn play_note(self, frequency: f32, duration_ms: u64) {
        self.audio_gen.gen_note(frequency, duration_ms);
    }
}