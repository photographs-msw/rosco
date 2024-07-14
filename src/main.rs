extern crate cpal;

use std::env;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

static SAMPLE_RATE: f32 = 44100.0;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;

fn main() {
    let args = get_args();
    let osc_type = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let duration_ms: u64 = args[2].parse().unwrap();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    match osc_type.as_str() {
        "sine" => run_sin_gen::<f32>(&device, &config.into(), frequency, duration_ms),
        "triangle" => run_triangle_gen::<f32>(&device, &config.into(), frequency, duration_ms),
        "square" => run_square_gen::<f32>(&device, &config.into(), frequency, duration_ms),
        "saw" => run_saw_gen::<f32>(&device, &config.into(), frequency, duration_ms),
        _ => run_sin_gen::<f32>(&device, &config.into(), frequency, duration_ms),
    }
}

macro_rules! create_stream {
    ($device:expr, $config:expr, $channels:expr, $err_fn:expr, $next_value:expr) => {
        $device.build_output_stream(
            $config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data::<T>(data, $channels, $next_value)
            },
            $err_fn,
            None,
        ).unwrap()
    };
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

fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    return args;
}

fn run_sin_gen<T>(device: &cpal::Device, config: &cpal::StreamConfig, frequency: f32,
                  duration_ms: u64)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;
    // Produce a sinusoid of maximum amplitude.
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = create_stream!(device, config, channels, err_fn, &mut next_value);
    stream.play().unwrap();
    // Keep the stream alive indefinitely to play sound
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}

fn run_triangle_gen<T>(device: &cpal::Device, config: &cpal::StreamConfig, frequency: f32,
                       duration_ms: u64)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;
    // Produce a sinusoid of maximum amplitude.
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        4.0 * ((frequency / SAMPLE_RATE * sample_clock)
            - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
            .floor()).abs()
            - 1.0
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = create_stream!(device, config, channels, err_fn, &mut next_value);
    stream.play().unwrap();
    // Keep the stream alive indefinitely to play sound
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}

fn run_square_gen<T>(device: &cpal::Device, config: &cpal::StreamConfig, frequency: f32,
                     duration_ms: u64)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;
    // Produce a sinusoid of maximum amplitude.
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        if (sample_clock * frequency / SAMPLE_RATE) % 1.0 < 0.5 {
            1.0
        } else {
            -1.0
        }
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = create_stream!(device, config, channels, err_fn, &mut next_value);
    stream.play().unwrap();
    // Keep the stream alive indefinitely to play sound
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}

fn run_saw_gen<T>(device: &cpal::Device, config: &cpal::StreamConfig, frequency: f32,
                  duration_ms: u64)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;
    // Produce a sinusoid of maximum amplitude.
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        2.0 * ((frequency / SAMPLE_RATE * sample_clock)
            - ((frequency / SAMPLE_RATE * sample_clock) + 0.5)
            .floor()).abs()
            - 1.0
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = create_stream!(device, config, channels, err_fn, &mut next_value);
    stream.play().unwrap();
    // Keep the stream alive indefinitely to play sound
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}
