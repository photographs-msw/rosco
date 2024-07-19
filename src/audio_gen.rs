use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::oscillator::{get_saw_freq, get_sin_freq, get_square_freq, get_triangle_freq,
                        OscType, SAMPLE_RATE};
use crate::note::Note;

pub(crate) struct AudioGen {
    channel_oscillators: Vec<Vec<OscType>>,
}

impl AudioGen {

    pub(crate) fn from_oscillators(oscillators: Vec<OscType>) -> Self {
        let mut channel_oscillators= Vec::new();
        channel_oscillators.push(oscillators);
        AudioGen {
            channel_oscillators
        }
    }

    pub(crate) fn from_channel_oscillators(channel_oscillators: Vec<Vec<OscType>>) -> Self {
        AudioGen {
            channel_oscillators
        }
    }

    pub(crate) fn gen_note<'a>(&self, note: &Note) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available");
        let config = device.default_output_config().unwrap();
        let oscillators = &self.channel_oscillators[0];//.clone();

        gen_note_impl::<f32>(&device, &config.into(), note, oscillators);
    }

    pub(crate) fn gen_notes(&self, notes: Vec<Note>) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available");
        let config = device.default_output_config().unwrap();

        gen_notes_impl::<f32>(&device, &config.into(), notes,
                              self.channel_oscillators.clone());
    }
}

fn gen_note_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                    note: &Note, oscillators: Vec<OscType>)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;
    let volume = note.volume.clone();
    let frequency = note.frequency.clone();
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        volume * get_freq(oscillators, frequency, sample_clock)
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
    std::thread::sleep(Duration::from_millis(note.duration_ms));
}

fn gen_notes_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                         notes: Vec<Note>, channel_oscillators: Vec<Vec<OscType>>)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        let mut freq = 0.0;
        for (i, note) in notes.iter().enumerate() {
            freq +=
                note.volume *
                    get_freq(channel_oscillators.get(i), note.frequency, sample_clock);
        }
        freq
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data::<f32>(data, channels, &mut next_value)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();
    std::thread::sleep(Duration::from_millis(1000));
}


fn get_freq(oscillators: Vec<OscType>, frequency: f32, sample_clock: f32) -> f32 {
    let mut freq = 0.0;
    for oscillator in oscillators {
        freq += match oscillator {
            OscType::Sine => get_sin_freq(frequency, sample_clock),
            OscType::Triangle => get_triangle_freq(frequency, sample_clock),
            OscType::Square => get_square_freq(frequency, sample_clock),
            OscType::Saw => get_saw_freq(frequency, sample_clock),
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
