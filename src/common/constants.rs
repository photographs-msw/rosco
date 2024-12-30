pub(crate) static FLOAT_EPSILON: f32 = 5.0 * f32::EPSILON;

pub(crate) static NO_TRACK: i16 = -1;

// khz samples per second, so 44.1k samples per second
// sample_clock samples / SAMPLE_RATE samples per second = seconds
pub(crate) const SAMPLE_RATE: f32 = 44100.0;
pub(crate) const SAMPLES_PER_MS: f32 = SAMPLE_RATE / 1000.0;
pub(crate) const NYQUIST_FREQUENCY: f32 = SAMPLE_RATE / 2.0;

pub(crate) static DEFAULT_LFO_AMPLITUDE: f32 = 0.5;
