use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::oscillator::get_freq;
use crate::oscillator::get_osc_types;
use crate::oscillator::OscType;
use crate::oscillator::SAMPLE_RATE;

pub(crate) fn gen_note(osc_types_arg: &str, frequency: f32, duration_ms: u64)
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    gen_note_helper::<f32>(&device, &config.into(), &osc_types_arg, frequency, duration_ms);
}

fn gen_note_helper<T>(device: &cpal::Device, config: &cpal::StreamConfig, osc_types_arg: &str,
                      frequency: f32, duration_ms: u64)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let osc_types: Vec<OscType> = get_osc_types(osc_types_arg);

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
