use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use crate::playback_note::PlaybackNote;

pub(crate) static SAMPLE_RATE: f32 = 44100.0;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Clone, Debug)]
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
fn modify_sample_lfo(sample: f32, lfo_freq: f32, lfo_amp: f32, sample_clock: f32) -> f32 {
    // Phase of the LFO: sine of (sample_clock * lfo_freq * TWO_PI / SAMPLE_RATE)
    //  i.e. clock position * frequency of the LFO, positioned in radius, normalized by sample rate
    // This is scaled by the amplitude of the LFO and then used to scale the sample
    sample * (lfo_amp * (sample_clock * lfo_freq * TWO_PI / SAMPLE_RATE).sin())
}
