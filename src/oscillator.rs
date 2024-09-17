use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::constants::SAMPLE_RATE;
use crate::playback_note::PlaybackNote;
use crate::sample_effect_trait::{ApplyEffect, BuilderWrapper};

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
pub(crate) fn get_notes_sample<
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Send
>
(playback_notes: &mut Vec<PlaybackNote<EnvelopeType, LFOType>>, sample_clock: f32) -> f32 {
    
    let mut out_sample = 0.0;
    for playback_note in playback_notes.iter_mut() {
        // get initial note sample value from its volume and waveforms
        let note = playback_note.note;
        let mut sample = note.volume *
            get_note_sample(&playback_note.waveforms.clone().unwrap(), note.frequency, sample_clock);
        
        // Modify sample by applying all signal processing
        sample = playback_note.apply_effects(sample, note.frequency, sample_clock);
        
        // sum each note sample into the final output
        out_sample += sample;
    }

    out_sample
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
