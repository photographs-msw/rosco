extern crate derive_builder;

mod audio_gen;
mod common;
mod effect;
mod envelope;
mod instrument;
mod midi;
mod note;
mod sequence;
mod track;

use crate::audio_gen::oscillator::{get_waveforms, Waveform};
use crate::effect::{flanger, lfo};
use crate::effect::flanger::{Flanger, FlangerBuilder};
use crate::envelope::envelope::{Envelope, EnvelopeBuilder};
use crate::envelope::envelope_pair::EnvelopePair;
use crate::note::playback_note::{NoteType, PlaybackNote};
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track::track::Track;
use crate::track::track_grid::TrackGridBuilder;

fn main() {
    // Init
    let computer_punk_version = "001";
    println!("playing 'computer punk {}'", computer_punk_version);

    println!("Loading args");
    let waveforms_arg = collect_args();
    println!("Args collected\nwaveforms: {}", waveforms_arg);
    let oscillators_tables = audio_gen::oscillator::OscillatorTables::new();//generate_sine_table();

    let midi_note_volume = 0.09;
    let sampled_note_volume = 0.01;

    // Track Effects
    // Envelopes
    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.45, 0.6))
        .decay(EnvelopePair(0.55, 0.7))
        .sustain(EnvelopePair(0.65, 0.6))
        .build().unwrap();
    let short_envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.1, 0.9))
        .decay(EnvelopePair(0.2, 0.85))
        .sustain(EnvelopePair(0.95, 0.85))
        .build().unwrap();

    // Flangers
    let flanger = FlangerBuilder::default()
        .window_size(20)
        .sample_buffer()
        .mix(0.5)
        .build().unwrap();

    // /Track Effects

    // Load Sample Notes and Tracks
    let start_time = 0.0;
    let sampled_playback_note = build_sampled_playback_note(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_16bit.wav",
        sampled_note_volume / 5.0,
        start_time,
        vec![envelope],
        vec![]
    );

    let mut sampled_playback_note_reverse = sampled_playback_note.clone();
    sampled_playback_note_reverse.sampled_note.reverse();
    sampled_playback_note_reverse.sampled_note.volume *= 1.5;

    let sample_track = load_sample_tracks(sampled_playback_note);
    let sample_track_rev = load_sample_tracks(sampled_playback_note_reverse);

    // Load MIDI Tracks
    let waveforms =
        vec![Waveform::Sine, Waveform::Saw, Waveform::Triangle, Waveform::Triangle, Waveform::Sine];
    let mut tracks = Vec::new();
    let mut midi_time_tracks_1 = load_midi_tracks(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_1.mid",
        waveforms.clone(),
        vec![envelope],
        vec![flanger.clone()],
        midi_note_volume);
    let mut midi_time_tracks_2 = load_midi_tracks(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_2.mid",
        waveforms.clone(),
        vec![envelope],
        vec![],
        midi_note_volume);
    let mut midi_time_tracks_3 = load_midi_tracks(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_3.mid",
        waveforms.clone(),
        vec![envelope],
        vec![],
        midi_note_volume);

    // Add Sample Tracks
    tracks.append(&mut midi_time_tracks_1);
    // tracks.append(&mut midi_time_tracks_2);
    tracks.append(&mut midi_time_tracks_3);
    // tracks.push(sample_track);
    // tracks.push(sample_track_rev);
    
    // TEMP DEBUG
    println!("{:#?}", midi_time_tracks_3);

    // Load and play Track Grid
    let track_grid = TrackGridBuilder::default()
        .tracks(tracks)
        .build().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });
    let mut loop_playback_notes = Vec::new();
    for playback_notes in rx {
        audio_gen::audio_gen::gen_notes_stream(playback_notes.clone(), oscillators_tables.clone());
        loop_playback_notes.push(playback_notes);
    }
    for _ in 0 .. 1 {
       for (i, playback_notes) in loop_playback_notes.iter_mut().enumerate() {
           if i % 2 == 0 {
               for playback_note in playback_notes.iter_mut() {
                   let new_flanger = FlangerBuilder::default()
                       .window_size(50)
                       .sample_buffer()
                       .mix(0.9)
                       .build().unwrap();
                   playback_note.flangers = vec![new_flanger];
               }
           }
           audio_gen::audio_gen::gen_notes_stream(playback_notes.clone(),
                                                  oscillators_tables.clone());
       } 
    }
}

fn build_sampled_playback_note(file_path: &str, volume: f32, start_time: f32,
        envelopes: Vec<Envelope>, flangers: Vec<Flanger>) -> PlaybackNote {
    let sample_data=
        audio_gen::audio_gen::read_audio_file(file_path).into_boxed_slice();
    let mut sample_buf: Vec<f32> = Vec::with_capacity(note::sampled_note::BUF_STORAGE_SIZE);
    for sample in  sample_data[..].iter() {
        sample_buf.push(*sample as f32);
    }
    let mut sampled_note = note::sampled_note::SampledNoteBuilder::default()
        .volume(volume)
        .start_time_ms(start_time)
        .end_time_ms((sample_data.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0)
        .build().unwrap();
    sampled_note.set_sample_buf(&sample_buf, sample_data.len());

    note::playback_note::PlaybackNoteBuilder::default()
        .note_type(NoteType::Sample)
        .sampled_note(sampled_note)
        .playback_start_time_ms(start_time)
        .playback_end_time_ms(
            start_time +
                ((sample_data.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0))
        .playback_sample_start_time(start_time as u64)
        .playback_sample_end_time(sample_buf.len() as u64)
        .envelopes(envelopes)
        .flangers(flangers)
        .build().unwrap()
}

fn load_midi_tracks(file_path: &str, waveforms: Vec<Waveform>, envelopes: Vec<Envelope>,
        flangers: Vec<Flanger>, volume: f32) -> Vec<Track<TimeNoteSequence>> {
    let mut midi_time_tracks =
        midi::midi::midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
           file_path, NoteType::Oscillator);

    for (i, track) in midi_time_tracks.iter_mut().enumerate() {
        for playback_notes in track.sequence.notes_iter_mut() {
            for playback_note in playback_notes {
                playback_note.note.waveforms = waveforms.clone();
                playback_note.note.volume = volume;
                playback_note.envelopes = envelopes.clone();
                playback_note.flangers = flangers.clone();
            }
        }
    }

    midi_time_tracks
}

fn load_sample_tracks(sampled_playback_note: PlaybackNote) -> Track<TimeNoteSequence> {
    let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();
    sequence.append_notes(&vec![sampled_playback_note.clone()]);
    track::track::TrackBuilder::default()
        .sequence(sequence)
        .build().unwrap()
}

// fn build_sample_track() -> Track<TimeNoteSequence> {
//
// }

fn collect_args () -> String {
    let mut waveforms_arg = String::from("sine");
    for (i, arg) in std::env::args().enumerate() {
        match i {
            // skip program name in 0th args position
            0 => continue,
            1 => waveforms_arg = arg,
            _ => break,
        }
    }

    waveforms_arg
}
