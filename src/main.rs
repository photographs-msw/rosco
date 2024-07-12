extern crate cpal;

use std::env;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// static NUM_ARGS: usize = 3;
static SAMPLE_RATE: f32 = 44100.0;
static TWO_PI: f32 = 2.0 * std::f32::consts::PI;
// static FREQUENCY: f32 = 440.0;

fn main() {
    let args = get_args();
    let frequency: f32 = args[0].parse().unwrap();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    run::<f32>(&device, &config.into(), frequency);
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, frequency: f32)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    // Produce a sinusoid of maximum amplitude.
    // let sample_rate = 44100.0;
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        (sample_clock * frequency * TWO_PI / SAMPLE_RATE).sin()
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let stream =
        device.build_output_stream(config,
                                   move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                                       write_data::<T>(data, channels, &mut next_value)
                                   },
                                   err_fn,
                                   None)
            .unwrap();
    stream.play().unwrap();

    // Keep the stream alive indefinitely to play sound
    std::thread::sleep(std::time::Duration::from_millis(10000));
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
