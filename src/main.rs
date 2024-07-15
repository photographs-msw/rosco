extern crate cpal;

use std::env;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

static SAMPLE_RATE: f32 = 44100.0;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

enum OscType {
    Sine,
    Triangle,
    Square,
    Saw,
}

fn main() {
    let args = get_args();
    let osc_types_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let duration_ms: u64 = args[2].parse().unwrap();

    run(&osc_types_arg, frequency, duration_ms);
}

fn run(osc_types_arg: &str, frequency: f32, duration_ms: u64)
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    run_gen::<f32>(&device, &config.into(), &osc_types_arg, frequency, duration_ms);
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    for output_frame in output.chunks_mut(channels) {
        let sample_to_write = next_sample();
        let value = T::from_sample::<f32>(sample_to_write);
        for output_sample in output_frame.iter_mut() {
            *output_sample = value;
        }
    }
}

fn get_sin_freq(frequency: f32, sample_clock: f32) -> f32 {
    (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
}

fn get_triangle_freq(frequency: f32, sample_clock: f32) -> f32 {
    4.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

fn get_square_freq(frequency: f32, sample_clock: f32) -> f32 {
    if (sample_clock * frequency / SAMPLE_RATE) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn get_saw_freq(frequency: f32, sample_clock: f32) -> f32 {
    2.0 * ((frequency / SAMPLE_RATE * sample_clock)
        - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
        .floor()).abs()
        - 1.0
}

fn run_gen<T>(device: &cpal::Device, config: &cpal::StreamConfig, osc_types_arg: &str,
              frequency: f32, duration_ms: u64)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut osc_types: Vec<OscType> = Vec::new();
    let osc_type_args = osc_types_arg.split(",");
    for osc_type_arg in osc_type_args {
        let osc_type: OscType = match osc_type_arg {
            "sine" => OscType::Sine,
            "triangle" => OscType::Triangle,
            "square" => OscType::Square,
            "saw" => OscType::Saw,
            _ => OscType::Sine,
        };
        osc_types.push(osc_type);
    }

    fn get_freq(osc_types: &Vec<OscType>, frequency: f32, sample_clock: f32) -> f32 {
        let num_osc_types = osc_types.len();
        let mut freq = 0.0;
        for osc_type in osc_types {
            let next_freq = match osc_type {
                OscType::Sine => get_sin_freq(frequency, sample_clock) / num_osc_types as f32,
                OscType::Triangle => get_triangle_freq(frequency, sample_clock) / num_osc_types as f32,
                OscType::Square => get_square_freq(frequency, sample_clock) / num_osc_types as f32,
                OscType::Saw => get_saw_freq(frequency, sample_clock) / num_osc_types as f32,
            };
            freq += next_freq;
        }
        println!("{}", freq);
        freq
    }

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        get_freq(&osc_types, frequency, sample_clock)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data::<T>(data, channels, &mut next_value)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}

fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    return args;
}
