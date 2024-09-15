use derive_builder::Builder;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::playback_note::PlaybackNote;
use crate::sample_effect_trait::ApplyEffect;

pub(crate) static SAMPLE_RATE: f32 = 44100.0;
pub(crate) static DEFAULT_LFO_AMPLITUDE: f32 = 0.5;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Clone, Debug, Hash, PartialEq)]
pub(crate) enum Waveform {
    GaussianNoise,
    Saw,
    Sine,
    Square,
    Triangle,
}

pub(crate) fn get_waveforms(waveform_arg: &str) -> Vec<Waveform> {
    waveform_arg.split(",")
        .map( |waveform| {
            let matched = match waveform {
                "gaussian_noise" => Waveform::GaussianNoise,
                "saw" => Waveform::Saw,
                "sine" => Waveform::Sine,
                "square" => Waveform::Square,
                "triangle" => Waveform::Triangle,
                _ => Waveform::Sine,
            };
            matched
        })
        .collect()
}

pub(crate) fn get_note_sample(waveforms: &Vec<Waveform>, frequency: f32, sample_clock: f32) -> f32 {
    let mut freq = 0.0;
    for waveform in waveforms {
        freq += match waveform {
            Waveform::GaussianNoise => get_gaussian_noise_sample(),
            Waveform::Saw => get_saw_sample(frequency, sample_clock),
            Waveform::Sine => get_sin_sample(frequency, sample_clock),
            Waveform::Square => get_square_sample(frequency, sample_clock),
            Waveform::Triangle => get_triangle_sample(frequency, sample_clock),
        };
    }
    freq
}

// NOTE: Assumes playback notes of Enum Kind that include Oscillator trait
pub(crate) fn get_notes_sample(playback_notes: &Vec<PlaybackNote>, sample_clock: f32) -> f32 
    // where PlaybackNoteKind: NoteOscillator
{
    let mut freq = 0.0;
    for playback_note in playback_notes.iter() {
        let note = playback_note.note;
        let mut volume = note.volume;
        
        if playback_note.has_envelope {
            volume *= playback_note
                .envelope.unwrap()
                .volume_factor(sample_clock / SAMPLE_RATE);
        }
        
        if playback_note.has_lfos {
            for lfo in playback_note.lfos.clone().unwrap() {
                // volume += lfo.get_lfo_sample(volume, sample_clock);
                volume = lfo.apply_effect(volume, note.frequency, sample_clock);
            }
        }
        
        // if playback_note.has_waveforms {
        freq += volume *
            get_note_sample(&playback_note.waveforms.clone().unwrap(), note.frequency, sample_clock);
        // } else {
        //     panic!("PlaybackNote must have waveforms");
        // }
    }

    freq
}

// /////////////

fn get_sin_sample(frequency: f32, sample_clock: f32) -> f32 {
    (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
}

fn get_triangle_sample(frequency: f32, sample_clock: f32) -> f32 {
    4.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

fn get_square_sample(frequency: f32, sample_clock: f32) -> f32 {
    if (sample_clock * frequency / SAMPLE_RATE) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn get_saw_sample(frequency: f32, sample_clock: f32) -> f32 {
    2.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

fn get_gaussian_noise_sample() -> f32 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = thread_rng();
    normal.sample(&mut rng)
}

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
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

#[allow(dead_code)]
impl LFO {
    fn get_lfo_sample(&self, sample: f32, sample_clock: f32) -> f32 {
        sample * self.amplitude * get_note_sample(&self.waveforms, self.frequency, sample_clock)
    }
}

impl ApplyEffect for LFO {
    fn apply_effect(&self, sample: f32, _frequency: f32, sample_clock: f32) -> f32 {
        sample + self.get_lfo_sample(sample, sample_clock)
    }
}
