use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::time;

use crate::audio_gen::get_sample;
use crate::common::constants;
use crate::common::constants::SAMPLE_RATE;
use crate::note::playback_note::PlaybackNote;

static WAV_SPEC: hound::WavSpec  = hound::WavSpec {
    channels: 1,
    sample_rate: constants::SAMPLE_RATE as u32,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

#[allow(dead_code)]
pub(crate) fn gen_note_stream(playback_note: PlaybackNote) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    gen_note_stream_impl::<f32>(&device, &config.into(), playback_note);
}

pub(crate) fn gen_notes_stream(playback_notes: Vec<PlaybackNote>, window_duration_ms: f32)
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    gen_notes_stream_impl::<f32>(&device, &config.into(), playback_notes,
                                 window_duration_ms as u64);
}

// This works to generate a note buffer from playback_note.note and load
// it into playback_note.sampled_note
// Can extend to future NoteTypes that are generators
// TODO gen_notes_version
#[allow(dead_code)]
pub(crate) fn gen_note_buffer(playback_note: &mut PlaybackNote) {
    let num_samples = (
        playback_note.playback_duration_ms().ceil() * 1000.0 * constants::SAMPLE_RATE) as usize;
    let mut sample_clock = 0f32;
    for _ in 0..num_samples {
        let sample = get_sample::get_note_sample(playback_note, sample_clock);
        playback_note.sampled_note.append_sample(sample);
        sample_clock = (sample_clock + 1.0) % constants::SAMPLE_RATE;
    }
}

// TODO PARAMETERIZE SAMPLE TYPE TO SUPPORT LOFI AND 32-BIT
#[allow(dead_code)]
pub(crate) fn read_audio_file(file_path: &str) -> Vec<i16> {
    let mut reader = hound::WavReader::open(file_path).unwrap();
    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    samples
}

// TODO PARAMETERIZE SAMPLE TYPE TO SUPPORT LOFI AND 32-BIT
#[allow(dead_code)]
pub(crate) fn write_audio_file(file_path: &str, samples: Vec<f32>) {
    let mut writer = hound::WavWriter::create(file_path, WAV_SPEC).unwrap();
    for sample in samples {
        writer.write_sample(sample.round() as i16).unwrap();
    }
    writer.finalize().unwrap();
}

#[allow(dead_code)]
fn gen_note_stream_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                           mut playback_note: PlaybackNote)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_clock = -1.0 / SAMPLE_RATE;
    let duration_ms = playback_note.playback_duration_ms();

    let mut next_sample = move || {
        sample_clock = (sample_clock + 1.0) % constants::SAMPLE_RATE;
        get_sample::get_note_sample(&mut playback_note, sample_clock)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_stream::<T>(data, channels, &mut next_sample)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(time::Duration::from_millis(duration_ms.ceil() as u64));
}

fn gen_notes_stream_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                            mut playback_notes: Vec<PlaybackNote>, max_note_duration_ms: u64)
{
    // TEMP DEBUG
    // println!("max_note_duration_ms: {}", max_note_duration_ms);
    
    let mut sample_clock = -1.0 / SAMPLE_RATE;
    let mut next_sample = move || {
        sample_clock = (sample_clock + 1.0) % constants::SAMPLE_RATE;
        get_sample::get_notes_sample(&mut playback_notes, sample_clock)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_stream::<f32>(data, channels, &mut next_sample)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(time::Duration::from_millis(max_note_duration_ms));
}

fn write_stream<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
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