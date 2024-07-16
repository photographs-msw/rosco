use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::oscillator::{OscType, SAMPLE_RATE};

fn get_freq(oscillators: &Vec<OscType>, frequency: f32, sample_clock: f32) -> f32 {
    let mut freq = 0.0;
    for oscillator in oscillators {
        freq += match oscillator {
            OscType::Sine => crate::oscillator::get_sin_freq(frequency, sample_clock),
            OscType::Triangle => crate::oscillator::get_triangle_freq(frequency, sample_clock),
            OscType::Square => crate::oscillator::get_square_freq(frequency, sample_clock),
            OscType::Saw => crate::oscillator::get_saw_freq(frequency, sample_clock),
        };
    }
    freq
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

pub(crate) struct AudioGen {
    oscillators: Vec<OscType>,
}

impl AudioGen {

    pub(crate) fn from_oscillators(oscillators: Vec<OscType>) -> Self {
        AudioGen {
            oscillators
        }
    }
    pub(crate) fn gen_note(self, frequency: f32, duration_ms: u64)
    {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available");
        let config = device.default_output_config().unwrap();

        self.gen_note_impl::<f32>(&device, &config.into(), frequency, duration_ms);
    }

    fn gen_note_impl<T>(self, device: &cpal::Device, config: &cpal::StreamConfig,
                        frequency: f32, duration_ms: u64)
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
            get_freq(&self.oscillators, frequency, sample_clock)
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
        std::thread::sleep(Duration::from_millis(duration_ms));
    }
}