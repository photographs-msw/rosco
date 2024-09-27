use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::common::constants::{NYQUIST_FREQUENCY, SAMPLE_RATE};  // khz samples per second
use crate::note::playback_note::{NoteType, PlaybackNote};

static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Clone, Debug, Hash, PartialEq)]
pub(crate) enum Waveform {
    GaussianNoise,
    Saw,
    Sine,
    Square,
    Triangle,
}

#[allow(dead_code)]
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

pub(crate) fn get_note_sample(playback_note: &mut PlaybackNote, sample_clock: f32) -> f32 {
    
    match playback_note.note_type {
        NoteType::Oscillator => {
            let mut sample = 0.0;
            // TODO MOVE WAVEFORMS TO UNDERLYING NOTE
            for waveform in playback_note.waveforms.clone() {
                sample += match waveform {
                    Waveform::GaussianNoise => get_gaussian_noise_sample(),
                    Waveform::Saw => get_saw_sample(playback_note.note.frequency, sample_clock),
                    Waveform::Sine => get_sin_sample(playback_note.note.frequency, sample_clock),
                    Waveform::Square => get_square_sample(playback_note.note.frequency,
                                                          sample_clock),
                    Waveform::Triangle => get_triangle_sample(playback_note.note.frequency,
                                                              sample_clock),
                }
            }
            playback_note.apply_effects(playback_note.note.volume * sample,
                                        sample_clock / SAMPLE_RATE)
        }
        NoteType::Sample => { playback_note.apply_effects(
            playback_note.sampled_note.volume *
                playback_note.sampled_note.clone().next_sample(),
            sample_clock / SAMPLE_RATE)
        }
    }
}

// NOTE: Assumes playback notes of Enum Kind that include Oscillator trait
pub(crate) fn get_notes_sample(playback_notes: &mut Vec<PlaybackNote>, sample_clock: f32) -> f32 {
    let mut out_sample = 0.0;
    for playback_note in playback_notes.iter_mut() {
        let sample =
            match playback_note.note_type {
                NoteType::Oscillator => {
                        get_note_sample(playback_note, sample_clock)
                }
                NoteType::Sample => playback_note.sampled_note.next_sample()
            };
        out_sample += sample;
    }

    if out_sample > NYQUIST_FREQUENCY {
        out_sample = NYQUIST_FREQUENCY;
    } else if out_sample < -NYQUIST_FREQUENCY {
        out_sample = -NYQUIST_FREQUENCY;
    } 
    out_sample
}

// /////////////

pub(crate) fn get_sin_sample(frequency: f32, sample_clock: f32) -> f32 {
    (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
}

pub(crate) fn get_triangle_sample(frequency: f32, sample_clock: f32) -> f32 {
    4.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

pub(crate) fn get_square_sample(frequency: f32, sample_clock: f32) -> f32 {
    if (sample_clock * frequency / SAMPLE_RATE) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub(crate) fn get_saw_sample(frequency: f32, sample_clock: f32) -> f32 {
    2.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

pub(crate) fn get_gaussian_noise_sample() -> f32 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = thread_rng();
    normal.sample(&mut rng)
}
