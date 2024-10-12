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

    let midi_note_volume = 0.2;
    let sampled_note_volume = 0.002;
    
    // Track Effects
    // Envelopes
    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.25, 0.7))
        .decay(EnvelopePair(0.25, 0.7))
        .sustain(EnvelopePair(0.75, 0.7))
        .build().unwrap();
    // let short_envelope = EnvelopeBuilder::default()
    //     .attack(EnvelopePair(0.1, 0.9))
    //     .decay(EnvelopePair(0.2, 0.85))
    //     .sustain(EnvelopePair(0.95, 0.85))
    //     .build().unwrap();
    
    // Flangers
    let flanger = FlangerBuilder::default()
        .window_size(15)
        .sample_buffer()
        .mix(0.25)
        .build().unwrap();
    
    // /Track Effects
    
    // Load Sample Notes and Tracks
    let start_time = 0.0;
    let sampled_playback_note = build_sampled_playback_note(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001_16bit.wav",
        sampled_note_volume,
        start_time,
    vec![envelope],
    vec![flanger.clone()]);

    let mut sampled_playback_note_reverse = sampled_playback_note.clone();
    sampled_playback_note_reverse.sampled_note.reverse();

    let sample_track = load_sample_tracks(sampled_playback_note);
    let sample_track_rev = load_sample_tracks(sampled_playback_note_reverse);
    
    // Load MIDI Tracks
    let mut midi_time_tracks= load_midi_tracks(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001.mid",
        &waveforms_arg,
        vec![envelope],
        vec![flanger.clone()],
        midi_note_volume);
    
    // Add Sample Tracks
    midi_time_tracks.push(sample_track);
    // midi_time_tracks.push(sample_track_rev);

    // Load and play Track Grid
    let track_grid = TrackGridBuilder::default()
        .tracks(midi_time_tracks)
        .build().unwrap();
    
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });
    for playback_notes in rx {
        audio_gen::audio_gen::gen_notes_stream(playback_notes, oscillators_tables.clone());
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

fn load_midi_tracks(file_path: &str, waveforms_arg: &str, envelopes: Vec<Envelope>,
        flangers: Vec<Flanger>, volume: f32) -> Vec<Track<TimeNoteSequence>> {
    let mut midi_time_tracks =
        midi::midi::midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
           file_path, NoteType::Oscillator);

    let num_tracks = midi_time_tracks.len();
    let track_waveforms =
        vec![audio_gen::oscillator::get_waveforms(waveforms_arg); num_tracks];

    for (i, track) in midi_time_tracks.iter_mut().enumerate() {
        for playback_notes in track.sequence.notes_iter_mut() {
            for playback_note in playback_notes {
                playback_note.note.waveforms = track_waveforms[i].clone();
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
