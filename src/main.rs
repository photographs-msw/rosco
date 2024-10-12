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
use crate::envelope::envelope::EnvelopeBuilder;
use crate::envelope::envelope_pair::EnvelopePair;
use crate::note::playback_note::NoteType;
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track::track_grid::TrackGridBuilder;

fn main() {
    println!("Loading args");
    let (waveforms_arg, frequency, volume, duration_ms) = collect_args();
    println!("Args collected\nwaveforms: {}, frequency: {}, volume: {}, duration_ms: {}",
             waveforms_arg, frequency, volume, duration_ms);

    let oscillators_tables = audio_gen::oscillator::OscillatorTables::new();//generate_sine_table();

    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.25, 0.5))
        .decay(EnvelopePair(0.35, 0.7))
        .sustain(EnvelopePair(0.75, 0.5))
        .build().unwrap();

    let sample_data_2 = audio_gen::audio_gen::read_audio_file(
        "/Users/markweiss/Downloads/punk_computer/001/punk_computer_002_16bit.wav")
        .into_boxed_slice();
    let mut sample_buf_2: Vec<f32> = Vec::with_capacity(note::sampled_note::BUF_STORAGE_SIZE);
    for sample in  sample_data_2[..].iter() {
        sample_buf_2.push(*sample as f32);
    }

    let mut sampled_note_2 = note::sampled_note::SampledNoteBuilder::default()
        .volume(0.0020)
        .start_time_ms(0.0)
        .end_time_ms((sample_data_2.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0)
        .build().unwrap();
    sampled_note_2.set_sample_buf(&sample_buf_2, sample_data_2.len());

    let sampled_playback_note_2 = note::playback_note::PlaybackNoteBuilder::default()
        .note_type(NoteType::Sample)
        .sampled_note(sampled_note_2)
        .playback_start_time_ms(0.0)
        .playback_end_time_ms((sample_data_2.len() as f32 / common::constants::SAMPLE_RATE)
            * 1000.0)
        .playback_sample_start_time(0)
        .playback_sample_end_time(sample_buf_2.len() as u64)
        // .envelopes(vec![envelope])
        // .flangers(vec![flanger::default_flanger()])
        .build().unwrap();
    let mut sampled_playback_note_reverse = sampled_playback_note_2.clone();
    sampled_playback_note_reverse.sampled_note.reverse();

    let mut midi_time_tracks =
        midi::midi::midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            "/Users/markweiss/Downloads/punk_computer/001/punk_computer_001.mid", NoteType::Oscillator);

    let num_tracks = midi_time_tracks.len();
    let track_waveforms =
        vec![audio_gen::oscillator::get_waveforms(&waveforms_arg); num_tracks];

    let track_effects = track::track_effects::TrackEffectsBuilder::default()
        .envelopes(vec![envelope])
        // .flangers(vec![flanger])
        .build().unwrap();
    for track in midi_time_tracks.iter_mut() {
        track.effects = track_effects.clone();
    }
    
    for (i, track) in midi_time_tracks.iter_mut().enumerate() {
        for playback_notes in track.sequence.notes_iter_mut() {
            for playback_note in playback_notes {
                playback_note.note.waveforms = track_waveforms[i].clone();
                playback_note.note.volume = 0.25;
            }
        }
    }
    for track in midi_time_tracks.iter_mut() {
        track.effects = track_effects.clone();
    }
    
    let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();
    sequence.append_notes(&vec![sampled_playback_note_2.clone()]);
    let track = track::track::TrackBuilder::default()
        .sequence(sequence)
        .effects(track_effects)
        .build().unwrap();
    let mut sequence_rev = TimeNoteSequenceBuilder::default().build().unwrap();
    sequence_rev.append_notes(&vec![sampled_playback_note_reverse]);
    let track_rev = track::track::TrackBuilder::default()
        .sequence(sequence_rev)
        .build().unwrap();

    midi_time_tracks.push(track);
    midi_time_tracks.push(track_rev);

    // Test building TrackGrid without envelopes and getting the default
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

fn collect_args () -> (String, f32, f32, f32) {
    let mut waveforms_arg = String::from("sine");
    let mut frequency: f32 = 0.0;
    let mut volume: f32 = 0.0;
    let mut duration_ms: f32 = 0.0;
    for (i, arg) in std::env::args().enumerate() {
        match i {
            // skip program name in 0th args position
            0 => continue,
            1 => waveforms_arg = arg,
            2 => frequency = arg.parse().unwrap(),
            3 => volume = arg.parse().unwrap(),
            4 => duration_ms = arg.parse().unwrap(),
            _ => break,
        }
    }

    (waveforms_arg, frequency, volume, duration_ms)
}
