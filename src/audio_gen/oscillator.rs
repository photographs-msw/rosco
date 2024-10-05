use rand::thread_rng;
use rand_distr::{Distribution, Normal};

static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
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

pub(crate) fn get_sin_sample(frequency: f32, sample_position: f32) -> f32 {
    (frequency * sample_position * TWO_PI).sin()
}

pub(crate) fn get_triangle_sample(frequency: f32, sample_position: f32) -> f32 {
    4.0 * ((frequency * sample_position)
        - ((frequency * sample_position) + 0.5)
        .floor()).abs()
        - 1.0
}

pub(crate) fn get_square_sample(frequency: f32, sample_position: f32) -> f32 {
    if (sample_position * frequency) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub(crate) fn get_saw_sample(frequency: f32, sample_position: f32) -> f32 {
    2.0 * ((frequency * sample_position)
        - ((frequency * sample_position) + 0.5)
        .floor()).abs()
        - 1.0
}

pub(crate) fn get_gaussian_noise_sample() -> f32 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = thread_rng();
    normal.sample(&mut rng)
}
