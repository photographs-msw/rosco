extern crate derive_builder;

mod audio_gen;
mod constants;
mod envelope;
mod envelope_pair;
mod float_utils;
mod grid_note_sequence;
mod instrument;
mod midi;
mod multi_instrument;
mod note;
mod note_sequence_trait;
mod oscillator;
mod playback_note;
mod time_note_sequence;
mod track;
mod track_grid;
mod lfo;

// TODO FIX main TO WORK WITH NEW SPLIT OFF PLAYBACK_NOTE

use crate::envelope::EnvelopeBuilder;
use crate::envelope_pair::EnvelopePair;
use crate::grid_note_sequence::{GridNoteSequence, GridNoteSequenceBuilder};
// use crate::instrument::InstrumentBuilder;
// use crate::multi_instrument::{MultiInstrumentBuilder};
// use crate::note::{Note, NoteBuilder};
use crate::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track_grid::TrackGridBuilder;

fn main() {
    println!("Loading args");
    let (waveforms_arg, frequency, volume, duration_ms) = collect_args();
    println!("Args collected\nwaveforms: {}, frequency: {}, volume: {}, duration_ms: {}",
             waveforms_arg, frequency, volume, duration_ms);

    // ####################################

    println!("Loading MIDI file");
    // Test loading MIDI file and playing back using multi-track, polyphonic grid with one
    // set of waveforms per track, notes per track, playing notes in windows of when they are active
    // and coordinated concurrent playback where one thread prepares the next window to play
    // and the other thread plays the current window
    let midi_grid_tracks =
        midi::midi_file_to_tracks::<GridNoteSequence, GridNoteSequenceBuilder>(
            "/Users/markweiss/Downloads/test.mid");
    println!("Loaded MIDI file into Vec<Track<GridNoteSequence>");

    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.15, 0.9))
        .decay(EnvelopePair(0.35, 0.88))
        .sustain(EnvelopePair(0.85, 0.9))
        .build().unwrap();

    let lfo = lfo::LFOBuilder::default()
        .frequency(44.1)
        .amplitude(0.25)
        .waveforms(vec![oscillator::Waveform::Sine])
        .build().unwrap();
    
    let num_tracks = midi_grid_tracks.len();
    let track_waveforms = vec![oscillator::get_waveforms(&waveforms_arg); num_tracks];
    
    let track_grid = TrackGridBuilder::default()
        .tracks(midi_grid_tracks)
        .track_waveforms(track_waveforms)
        .track_envelopes(vec![envelope; num_tracks])
        // .track_envelopes(vec![Some(envelope::default_envelope()); num_tracks])
        .track_lfos(vec![vec![lfo]; num_tracks])
        .build().unwrap();
    
    println!("Playing MIDI file from TrackGrid GridNoteSequence");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });
    for playback_notes in rx {
        // TEMP DEBUG
        // println!("\n=====> playback_notes\n: {:#?}", playback_note_kinds);
    
        let window_duration_ms = playback_notes[0].playback_duration_ms();
        audio_gen::gen_notes(playback_notes, window_duration_ms as u64);
    }
    println!("Played MIDI file from TrackGrid GridNoteSequence");
    
    // ####################################

    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.15, 0.8))
        .decay(EnvelopePair(0.18, 0.7))
        .sustain(EnvelopePair(0.85, 0.8))
        .build().unwrap();

    let lfo = lfo::LFOBuilder::default()
        .frequency(22.05)
        .amplitude(0.95)
        .waveforms(vec![oscillator::Waveform::Sine])
        .build().unwrap();

    println!("Loading MIDI file");
    let midi_time_tracks =
        midi::midi_file_to_tracks::<TimeNoteSequence, TimeNoteSequenceBuilder>(
            "/Users/markweiss/Downloads/test.mid");
    println!("Loaded MIDI file into Vec<Track<TimeNoteSequence>");
    
    let num_tracks = midi_time_tracks.len();
    let track_waveforms = vec![oscillator::get_waveforms(&waveforms_arg); num_tracks];
    // Test building TrackGrid without envelopes and getting the default
    let track_grid = TrackGridBuilder::default()
        .tracks(midi_time_tracks)
        .track_waveforms(track_waveforms)
        .track_envelopes(vec![envelope; num_tracks])
        // .track_envelopes(vec![Some(envelope::default_envelope()); num_tracks])
        .track_lfos(vec![vec![lfo]; num_tracks])
        .build().unwrap();
    
    println!("Playing MIDI file from TrackGrid TimeNoteSequence");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });
    for playback_notes in rx {
        // TEMP DEBUG
        // println!( "\n=====> PLAYBACK NOTES IN RECEIVE \n: {:#?}", playback_notes);
    
        let window_duration_ms = playback_notes[0].playback_duration_ms();
        audio_gen::gen_notes(playback_notes, window_duration_ms as u64);
    }
    println!("Played MIDI file from TrackGrid TimeNoteSequence");

    // ####################################
    // 
    // println!("Setting up Instrument");
    // // Setup MultiInstrument and Instrument
    // let midi_tracks_2 =
    //     midi::midi_file_to_tracks::<GridNoteSequence, GridNoteSequenceBuilder>(
    //         "/Users/markweiss/Downloads/test.mid");
    // let waveforms_1 = oscillator::get_waveforms(&waveforms_arg);
    // let waveform_2 = oscillator::get_waveforms(&waveforms_arg);
    // let midi_track_waveforms = vec![waveforms_1, waveform_2];
    // let num_tracks = midi_track_waveforms.len();
    // let mut midi_multi_instrument = MultiInstrumentBuilder::default()
    //     .track_waveforms(midi_track_waveforms)
    //     .num_tracks(num_tracks)
    //     .add_tracks(midi_tracks_2)
    //     .build().unwrap();
    // midi_multi_instrument.loop_once();
    // println!("Set up Instrument");
    // 
    // println!("Setting up MultiInstrument");
    // let waveforms_3 = oscillator::get_waveforms(&waveforms_arg);
    // let waveform_4 = oscillator::get_waveforms(&waveforms_arg);
    // let track_waveforms = vec![waveforms_3, waveform_4];
    // let num_tracks = track_waveforms.len();
    // let mut multi_instrument = MultiInstrumentBuilder::default()
    //     .track_waveforms(track_waveforms)
    //     .num_tracks(num_tracks)
    //     .tracks()
    //     .build().unwrap();
    // 
    // println!("Setting up MultiInstrument Envelope");
    // let envelope = EnvelopeBuilder::default()
    //     .attack(EnvelopePair(0.3, 0.9))
    //     .decay(EnvelopePair(0.35, 0.7))
    //     .sustain(EnvelopePair(0.6, 0.65))
    //     .build().unwrap();
    // println!("Set up MultiInstrument Envelope");
    // println!("Set up MultiInstrument");
    // 
    // println!("Setting up Notes for MultiInstrument");
    // // builder with default volume
    // let note_1: Note = NoteBuilder::default()
    //     .frequency(frequency)
    //     .start_time_ms(0.0)
    //     .duration_ms(duration_ms)
    //     .end_time_ms()
    //     .envelope(envelope)
    //     // .envelope(envelope)
    //     .no_track()
    //     .build().unwrap();
    // let note_2: Note = NoteBuilder::default()
    //     .frequency(frequency)
    //     .volume(volume * 0.75)
    //     .start_time_ms(duration_ms)
    //     .duration_ms(duration_ms)
    //     .end_time_ms()
    //     .envelope(envelope)
    //     // .envelope(envelope)
    //     .no_track()
    //     .build().unwrap();
    // println!("Set up Notes for MultiInstrument");
    // 
    // println!("Adding Notes to MultiInstrument");
    // // Test MultiInstrument, primitive concurrent playback that simply gets the next note to play
    // // from each track
    // multi_instrument.add_note_to_track(0, note_1 );
    // multi_instrument.add_note_to_track(1, note_2);
    // multi_instrument.add_note_to_track(0, note_2 );
    // multi_instrument.add_note_to_track(1, note_1);
    // multi_instrument.add_note_to_tracks(note_1);
    // println!("Added Notes to MultiInstrument");
    // println!("Playing Notes on MultiInstrument");
    // multi_instrument.play_track_notes_and_advance();
    // multi_instrument.set_volume_for_track(0, 0.25);
    // multi_instrument.play_track_notes();
    // multi_instrument.set_volume_for_tracks(0.75);
    // multi_instrument.loop_once();
    // multi_instrument.loop_n(2);
    // multi_instrument.play_notes_direct(vec![note_1, note_2]);
    // println!("Played Notes on MultiInstrument");
    // 
    // println!("Setting up Notes for Instrument");
    // // Test single Instrument
    // // override default builder volume of 1.0
    // let instrument_volume: f32 = 0.9;
    // let mut instrument = InstrumentBuilder::default()
    //     .waveforms(oscillator::get_waveforms(&waveforms_arg))
    //     .volume(instrument_volume)
    //     .track()
    //     .build().unwrap();
    // let note_3: Note = NoteBuilder::default()
    //     .frequency(frequency)
    //     .start_time_ms(0.0)
    //     .duration_ms(duration_ms)
    //     .end_time_ms()
    //     .default_envelope()
    //     .no_track()
    //     .build().unwrap();
    // let note_4: Note = NoteBuilder::default()
    //     .frequency(frequency)
    //     .volume(volume * 0.75)
    //     .start_time_ms(duration_ms)
    //     .duration_ms(duration_ms)
    //     .end_time_ms()
    //     .default_envelope()
    //     .no_track()
    //     .build().unwrap();
    // println!("Set up Notes for Instrument");
    // 
    // println!("Adding Notes to Instrument");
    // instrument.add_note(note_3);
    // instrument.add_note(note_4);
    // println!("Added Notes to Instrument");
    // 
    // println!("Playing Notes on Instrument");
    // instrument.play_note_and_advance(0);
    // instrument.set_volume(0.25);
    // instrument.play_note();
    // instrument.set_volume(0.75);
    // instrument.loop_once();
    // instrument.loop_n(2);
    // instrument.reset();
    // instrument.play_note();
    // instrument.play_note_direct(&note_3);
    // println!("Played Notes on Instrument");
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
