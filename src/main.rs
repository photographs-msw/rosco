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
use crate::sequence::grid_note_sequence::{GridNoteSequence, GridNoteSequenceBuilder};
// use crate::instrument::InstrumentBuilder;
// use crate::multi_instrument::{MultiInstrumentBuilder};
// use crate::note::{Note, NoteBuilder};
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track::track_grid::TrackGridBuilder;

fn main() {
    println!("Loading args");
    let (waveforms_arg, frequency, volume, duration_ms) = collect_args();
    println!("Args collected\nwaveforms: {}, frequency: {}, volume: {}, duration_ms: {}",
             waveforms_arg, frequency, volume, duration_ms);

    // ####################################

    // println!("Loading MIDI file");
    // Test loading MIDI file and playing back using multi-track, polyphonic grid with one
    // set of waveforms per track, notes per track, playing notes in windows of when they are active
    // and coordinated concurrent playback where one thread prepares the next window to play
    // and the other thread plays the current window
    // let mut midi_grid_tracks =
    //     midi::midi::midi_file_to_tracks::<GridNoteSequence, GridNoteSequenceBuilder>(
    //         "/Users/markweiss/Downloads/test.mid", NoteType::Oscillator);
    // println!("Loaded MIDI file into Vec<Track<GridNoteSequence>");

    let oscillators_tables = audio_gen::oscillator::OscillatorTables::new();//generate_sine_table();

    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.05, 0.5))
        .decay(EnvelopePair(0.15, 0.5))
        .sustain(EnvelopePair(0.85, 0.5))
        .build().unwrap();

    // ####################################

    println!("Play SampledNote");

    let sample_data = audio_gen::audio_gen::read_audio_file("/Users/markweiss/Downloads/test2.wav")
        .into_boxed_slice();
    let mut sample_buf: Vec<f32> = Vec::with_capacity(note::sampled_note::BUF_STORAGE_SIZE);
    for sample in  sample_data[..].iter() {
        sample_buf.push(*sample as f32);
    }
    let sample_data_2 = audio_gen::audio_gen::read_audio_file("/Users/markweiss/Downloads/punk_computer_002_16bit.wav")
        .into_boxed_slice();
    let mut sample_buf_2: Vec<f32> = Vec::with_capacity(note::sampled_note::BUF_STORAGE_SIZE);
    for sample in  sample_data_2[..].iter() {
        sample_buf_2.push(*sample as f32);
    }

    // let envelope = EnvelopeBuilder::default()
    //     .attack(EnvelopePair(0.25, 0.9))
    //     .decay(EnvelopePair(0.35, 0.88))
    //     .sustain(EnvelopePair(0.75, 0.9))
    //     .build().unwrap();

    let mut sampled_note = note::sampled_note::SampledNoteBuilder::default()
        .volume(0.00009)
        .start_time_ms(0.0)
        .end_time_ms((sample_data.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0)
        .build().unwrap();
    sampled_note.set_sample_buf(&sample_buf, sample_data.len());

    let mut sampled_note_2 = note::sampled_note::SampledNoteBuilder::default()
        .volume(0.0005)
        .start_time_ms(0.0)
        .end_time_ms((sample_data_2.len() as f32 / common::constants::SAMPLE_RATE) * 1000.0)
        .build().unwrap();
    sampled_note_2.set_sample_buf(&sample_buf_2, sample_data_2.len());

    let sampled_playback_note = note::playback_note::PlaybackNoteBuilder::default()
        .note_type(NoteType::Sample)
        .sampled_note(sampled_note)
        .playback_start_time_ms(0.0)
        .playback_end_time_ms((sample_data.len() as f32 / common::constants::SAMPLE_RATE)
            * 1000.0)
        .playback_sample_start_time(0)
        .playback_sample_end_time(sample_buf.len() as u64)
        .envelopes(vec![envelope])
        // .flangers(vec![flanger::default_flanger()])
        .build().unwrap();
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

    // for i in 0..2 {
    //     let next_sampled_note = sampled_playback_note.clone();
    //     audio_gen::audio_gen::gen_notes_stream(vec![next_sampled_note], oscillators_tables.clone());
    // }
    // audio_gen::audio_gen::gen_notes_stream(vec![ sampled_playback_note_2.clone()], oscillators_tables.clone());
    println!("Played SampledNote");

    //
    // let lfo = lfo::LFOBuilder::default()
    //     .frequency(44.1)
    //     .amplitude(0.25)
    //     .waveforms(vec![audio_gen::oscillator::Waveform::Sine])
    //     .build().unwrap();
    //
    // let flange = flanger::FlangerBuilder::default()
    //     .window_size(20)
    //     .sample_buffer()
    //     .build().unwrap();
    //
    // let track_waveforms = audio_gen::oscillator::get_waveforms(&waveforms_arg);
    // for track in midi_grid_tracks.iter_mut() {
    //     for playback_notes in track.sequence.sequence_iter_mut() {
    //         for playback_note in playback_notes {
    //             playback_note.note.waveforms = track_waveforms.clone();
    //         }
    //     }
    // }
    //
    // let track_effects = track::track_effects::TrackEffectsBuilder::default()
    //     .envelopes(vec![envelope])
    //     .lfos(vec![lfo])
    //     .flangers(vec![flange])
    //     .build().unwrap();
    // for track in midi_grid_tracks.iter_mut() {
    //     track.effects = track_effects.clone();
    // }
    //
    // let track_grid = TrackGridBuilder::default()
    //     .tracks(midi_grid_tracks)
    //     .build().unwrap();
    
    println!("Playing MIDI file from TrackGrid GridNoteSequence");
    // let (tx, rx) = std::sync::mpsc::channel();
    // std::thread::spawn(move || {
    //     for playback_notes in track_grid {
    //         tx.send(playback_notes).unwrap();
    //     }
    // });
    // for playback_notes in rx {
    //     audio_gen::audio_gen::gen_notes_stream(playback_notes);
    // }
    println!("Played MIDI file from TrackGrid GridNoteSequence");
    
    // ####################################
    
    println!("Loading MIDI file");
    let mut midi_time_tracks =
        midi::midi::midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            "/Users/markweiss/Downloads/punk_computer_002.mid", NoteType::Oscillator);
    println!("Loaded MIDI file into Vec<Track<TimeNoteSequence>");
    
    let num_tracks = midi_time_tracks.len();
    let track_waveforms =
        vec![audio_gen::oscillator::get_waveforms(&waveforms_arg); num_tracks];
    
    let flanger = flanger::FlangerBuilder::default()
        .window_size(30)
        .sample_buffer()
        .build().unwrap();
    
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
                playback_note.note.volume = 0.7;
            }
        }
    }
    for track in midi_time_tracks.iter_mut() {
        track.effects = track_effects.clone();
    }
    
    let mut sequence = TimeNoteSequenceBuilder::default()
        .build().unwrap();
    sequence.append_notes(&vec![sampled_playback_note_2.clone()]);
    let track = track::track::TrackBuilder::default()
        .sequence(sequence)
        .effects(track_effects)
        .build().unwrap();
    
    // midi_time_tracks.push(track);
    
    // Test building TrackGrid without envelopes and getting the default
    let track_grid = TrackGridBuilder::default()
        .tracks(midi_time_tracks)
        .build().unwrap();
    
    println!("Playing MIDI file from TrackGrid TimeNoteSequence");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });
    for playback_notes in rx {
        audio_gen::audio_gen::gen_notes_stream(playback_notes, oscillators_tables.clone());
    }
    println!("Played MIDI file from TrackGrid TimeNoteSequence");

    audio_gen::audio_gen::gen_notes_stream(vec![sampled_playback_note_2.clone()], oscillators_tables.clone());
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
