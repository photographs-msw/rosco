use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::time;

use crate::oscillator;
use crate::note::Note;

pub(crate) fn gen_note_audio(volume: f32, frequency: f32, duration_ms: f32,
                             waveforms: &Vec<u8>) {
    gen_note_impl::<f32>(volume, frequency, duration_ms, waveforms.clone());
}

pub(crate) fn gen_notes_audio(notes: &Vec<Note>,
                              track_waveforms: &Vec<Vec<u8>>) {
    let note_max_duration_ms=
        f32::max(1000.0, notes.iter().map(|note| note.duration_ms).sum());
    gen_notes_impl::<f32>(notes.clone(), track_waveforms.clone(), note_max_duration_ms);
}

/**
 * This macro generates a closure that will be used to generate the next value in the audio stream
 * It takes a vector of waveforms, volume, frequency, and sample clock as input and returns a closure
 * calls the oscillator::get_waveform_sample function for each waveform in the vector and sums the
 * results. Do this because we can't pass the waveforms vector into the closure directly, because
 * it used in a `move` callback, which means it will be run in another thread with an unknown
 * lifetime. This is a workaround to get the sum of calling all the waveforms into the closure.
 */
macro_rules! generate_next_value {
    ($waveforms:expr, $volume:expr, $frequency:expr, $sample_clock:expr) => {
        move || {
            $waveforms.iter().map(|waveform| {
                $volume * oscillator::get_note_sample(*waveform, $frequency, $sample_clock)
            }).sum()
        }
    };
}

/**
* Same as above but for multiple notes, for each note it calls the oscillator::get_waveform_sample
* with the associated set of waveforms for that note and sums the results.
*/
macro_rules! generate_next_values {
    ($notes:expr, $track_waveforms:expr, $sample_clock:expr) => {
        move || {
            let mut sample: f32 = 0.0;
            for (i, note) in $notes.iter().enumerate() {
                sample += $track_waveforms[i].iter().map(|waveform| {
                    note.volume * oscillator::get_note_sample(*waveform, note.frequency,
                                                              $sample_clock)
                }).sum::<f32>()
            }
            sample
        }
    };
}

//noinspection ALL
fn gen_note_impl<T>(volume: f32, frequency: f32, duration_ms: f32, waveforms: Vec<u8>)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();
    let mut sample_clock = 0f32;
    sample_clock = (sample_clock + 1.0) % oscillator::SAMPLE_RATE;

    let mut next_value = generate_next_value!(waveforms, volume, frequency, sample_clock);

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data::<T>(data, channels, &mut next_value)
        },
        err_fn,
        None
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(time::Duration::from_millis(duration_ms as u64));
}

//noinspection DuplicatedCode
fn gen_notes_impl<T>(notes: Vec<Note>, track_waveforms: Vec<Vec<u8>>, note_max_duration_ms: f32)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();
    let mut sample_clock = 0f32;
    sample_clock = (sample_clock + 1.0) % oscillator::SAMPLE_RATE;

    let mut next_value = generate_next_values!(&notes, track_waveforms, sample_clock);

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data::<T>(data, channels, &mut next_value)
        },
        err_fn,
        None
    ).unwrap();

    stream.play().unwrap();
        std::thread::sleep(time::Duration::from_millis(note_max_duration_ms as u64));
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
