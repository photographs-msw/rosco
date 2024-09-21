use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::time;

use crate::audio_gen::oscillator;
use crate::common::constants;
use crate::note::note::Note;
use crate::note::playback_note::PlaybackNote;

#[allow(dead_code)]
pub(crate) fn gen_note(note: &Note, waveforms: Vec<oscillator::Waveform>) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    gen_note_impl::<f32>(&device, &config.into(), note, waveforms);
}

pub(crate) fn gen_notes(playback_notes: Vec<PlaybackNote>, window_duration_ms: u64)
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    gen_notes_impl::<f32>(&device, &config.into(), playback_notes, window_duration_ms);
}

#[allow(dead_code)]
fn gen_note_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig, note: &Note,
                    waveforms: Vec<oscillator::Waveform>)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = 0f32;

    // TODO HOW TO GET POSITION ON EACH ITERATION AND GET VOLUME FACTOR IN THE CALLBACK
    let note_volume = note.volume.clone();
    let frequency = note.frequency.clone();
    let mut next_sample = move || {
        sample_clock = (sample_clock + 1.0) % constants::SAMPLE_RATE;
        note_volume * oscillator::get_note_sample(&waveforms, frequency, sample_clock)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data::<T>(data, channels, &mut next_sample)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(time::Duration::from_millis(note.duration_ms as u64));
}

fn gen_notes_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                     mut playback_notes: Vec<PlaybackNote>, max_note_duration_ms: u64)
{
    let mut sample_clock = 0f32;
    let mut next_sample = move || {
        sample_clock = (sample_clock + 1.0) % constants::SAMPLE_RATE;
        oscillator::get_notes_sample(&mut playback_notes, sample_clock)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data::<f32>(data, channels, &mut next_sample)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(time::Duration::from_millis(max_note_duration_ms));
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
