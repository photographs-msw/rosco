extern crate derive_builder;

mod audio_gen;
mod track;
mod instrument;
mod multi_instrument;
mod note;
mod oscillator;
mod sequence;
mod utils;
mod midi;
mod sequence_grid;

use crate::instrument::InstrumentBuilder;
use crate::multi_instrument::{MultiInstrumentBuilder};
use crate::note::{Note, NoteBuilder};

fn main() {
    let args = utils::get_cli_args();
    let waveforms_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let volume: f32 = args[2].parse().unwrap();
    let duration_ms: f32 = args[3].parse().unwrap();

    let midi_tracks =
        midi::midi_file_to_tracks("/Users/markweiss/Downloads/test.mid");
    let waveforms_1 = oscillator::get_waveforms(&waveforms_arg);
    let waveform_2 = oscillator::get_waveforms(&waveforms_arg);
    let midi_track_waveforms = vec![waveforms_1, waveform_2];
    let num_tracks = midi_track_waveforms.len();
    let mut midi_multi_instrument = MultiInstrumentBuilder::default()
        .track_waveforms(midi_track_waveforms)
        .num_tracks(num_tracks)
        .add_tracks(midi_tracks)
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
    // builder with default volume
    let note_1: Note = NoteBuilder::default()
        .frequency(frequency)
        .duration_ms(duration_ms)
        .build().unwrap();
    let note_2: Note = NoteBuilder::default()
        .frequency(frequency)
        .volume(volume * 0.75)
        .duration_ms(duration_ms)
        .build().unwrap();
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

    // override default builder volume of 1.0
    let instrument_volume: f32 = 0.9;
    let mut instrument = InstrumentBuilder::default()
        .waveforms(oscillator::get_waveforms(&waveforms_arg))
        .volume(instrument_volume)
        .track()
        .build().unwrap();
    let note_3: Note = NoteBuilder::default()
        .frequency(frequency)
        .duration_ms(duration_ms)
        .build().unwrap();
    let note_4: Note = NoteBuilder::default()
        .frequency(frequency)
        .volume(volume * 0.75)
        .duration_ms(duration_ms)
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
