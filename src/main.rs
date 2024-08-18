extern crate derive_builder;

mod audio_gen;
mod track;
mod instrument;
mod multi_instrument;
mod note;
mod oscillator;
mod sequence;
mod midi;
mod track_grid;
mod envelope;
mod constants;

use crate::envelope::{EnvelopeBuilder, EnvelopePair};
use crate::instrument::InstrumentBuilder;
use crate::multi_instrument::{MultiInstrumentBuilder};
use crate::note::{Note, NoteBuilder};
use crate::track_grid::{TrackGrid, TrackGridBuilder};

fn main() {
    let (waveforms_arg, frequency, volume, duration_ms) = collect_args();

    let bpm = midi::get_bpm("/Users/markweiss/Downloads/test.mid");
    // Test loading MIDI file and playing back using multi-track, polyphonic grid with one
    // set of waveforms per track, notes per track, playing notes in windows of when they are active
    // and coordinated concurrent playback where one thread prepares the next window to play
    // and the other thread plays the current window
    // let track_grid: TrackGrid = TrackGridBuilder::default()
    //     .tracks(midi::midi_file_to_tracks("/Users/markweiss/Downloads/test.mid"))
    //     .track_waveforms(vec![oscillator::get_waveforms(&waveforms_arg)])
    //     .sample_clock_index(0.0)
    //     .bpm(bpm)
    //     .build().unwrap();
    // 
    // let (tx, rx) = std::sync::mpsc::channel();
    // std::thread::spawn(move || {
    //     for notes_window in track_grid {
    //         tx.send(notes_window).unwrap();
    //     }
    // });
    // for notes_window in rx {
    //     let window_duration_ms = notes_window.window_duration_ms();
    //     audio_gen::gen_notes(notes_window.notes_data.notes,
    //                          notes_window.notes_data.notes_waveforms,
    //                          window_duration_ms as u64);
    // }

    // Setup MultiInstrument and Instrument
    let midi_tracks_2 =
        midi::midi_file_to_tracks("/Users/markweiss/Downloads/test.mid");
    let waveforms_1 = oscillator::get_waveforms(&waveforms_arg);
    let waveform_2 = oscillator::get_waveforms(&waveforms_arg);
    let midi_track_waveforms = vec![waveforms_1, waveform_2];
    let num_tracks = midi_track_waveforms.len();
    let mut midi_multi_instrument = MultiInstrumentBuilder::default()
        .track_waveforms(midi_track_waveforms)
        .num_tracks(num_tracks)
        .add_tracks(midi_tracks_2)
        .build().unwrap();
    midi_multi_instrument.loop_once();

    let waveforms_3 = oscillator::get_waveforms(&waveforms_arg);
    let waveform_4 = oscillator::get_waveforms(&waveforms_arg);
    let track_waveforms = vec![waveforms_3, waveform_4];
    let num_tracks = track_waveforms.len();
    let mut multi_instrument = MultiInstrumentBuilder::default()
        .track_waveforms(track_waveforms)
        .num_tracks(num_tracks)
        .tracks()
        .build().unwrap();

    let envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.3, 0.9))
        .decay(EnvelopePair(0.35, 0.7))
        .sustain(EnvelopePair(0.6, 0.65))
        .build().unwrap();
    // builder with default volume
    let note_1: Note = NoteBuilder::default()
        .frequency(frequency)
        .start_time_ms(0.0)
        .duration_ms(duration_ms)
        .end_time_ms()
        .default_envelope()
        // .envelope(envelope)
        .no_track()
        .build().unwrap();
    let note_2: Note = NoteBuilder::default()
        .frequency(frequency)
        .volume(volume * 0.75)
        .start_time_ms(duration_ms)
        .duration_ms(duration_ms)
        .end_time_ms()
        .default_envelope()
        // .envelope(envelope)
        .no_track()
        .build().unwrap();

    // Test MultiInstrument, primitive concurrent playback that simply gets the next note to play
    // from each track
    multi_instrument.add_note_to_track(0, note_1 );
    multi_instrument.add_note_to_track(1, note_2);
    multi_instrument.add_note_to_track(0, note_2 );
    multi_instrument.add_note_to_track(1, note_1);
    multi_instrument.add_note_to_tracks(note_1);
    multi_instrument.play_track_notes_and_advance();
    multi_instrument.set_volume_for_track(0, 0.25);
    multi_instrument.play_track_notes();
    multi_instrument.set_volume_for_tracks(0.75);
    multi_instrument.loop_once();
    multi_instrument.loop_n(2);
    multi_instrument.play_notes_direct(vec![note_1, note_2]);

    // Test single Instrument
    // override default builder volume of 1.0
    let instrument_volume: f32 = 0.9;
    let mut instrument = InstrumentBuilder::default()
        .waveforms(oscillator::get_waveforms(&waveforms_arg))
        .volume(instrument_volume)
        .track()
        .build().unwrap();
    let note_3: Note = NoteBuilder::default()
        .frequency(frequency)
        .start_time_ms(0.0)
        .duration_ms(duration_ms)
        .end_time_ms()
        .default_envelope()
        .no_track()
        .build().unwrap();
    let note_4: Note = NoteBuilder::default()
        .frequency(frequency)
        .volume(volume * 0.75)
        .start_time_ms(duration_ms)
        .duration_ms(duration_ms)
        .end_time_ms()
        .default_envelope()
        .no_track()
        .build().unwrap();
    instrument.add_note(note_3);
    instrument.add_note(note_4);
    instrument.play_note_and_advance();
    instrument.set_volume(0.25);
    instrument.play_note();
    instrument.set_volume(0.75);
    instrument.loop_once();
    instrument.loop_n(2);
    instrument.reset();
    instrument.play_note();
    instrument.play_note_direct(&note_3);
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
