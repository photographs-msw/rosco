extern crate derive_builder;

mod audio_gen;
mod channel;
mod instrument;
mod multi_instrument;
mod note;
mod oscillator;
mod sequence;
mod utils;

use crate::instrument::InstrumentBuilder;
use crate::multi_instrument::{MultiInstrumentBuilder};
use crate::note::{Note, NoteBuilder};
use crate::utils::get_cli_args;

fn main() {
    let args = get_cli_args();
    let osc_types_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let volume: f32 = args[2].parse().unwrap();
    let duration_ms: u64 = args[3].parse().unwrap();

    let oscillators_1 = oscillator::get_osc_types(&osc_types_arg);
    let oscillators_2 = oscillator::get_osc_types(&osc_types_arg);
    let channel_oscillators = vec![oscillators_1, oscillators_2];
    let num_channels = channel_oscillators.len();
    let mut multi_instrument = MultiInstrumentBuilder::default()
        .channel_oscillators(channel_oscillators)
        .num_channels(num_channels)
        .channels()
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
    multi_instrument.add_note_to_channel(0, note_1 );
    multi_instrument.add_note_to_channel(1, note_2);
    multi_instrument.add_note_to_channel(0, note_2 );
    multi_instrument.add_note_to_channel(1, note_1);
    multi_instrument.add_note_to_channels(note_1);
    multi_instrument.play_channel_notes_and_advance();
    multi_instrument.set_volume_for_channel(0, 0.25);
    multi_instrument.play_channel_notes();
    multi_instrument.set_volume_for_channels(0.75);
    multi_instrument.loop_once();
    multi_instrument.loop_n(2);
    multi_instrument.play_notes_direct(vec![note_1, note_2]);

    // override default builder volume of 1.0
    let instrument_volume: f32 = 0.9;
    let mut instrument = InstrumentBuilder::default().
        oscillators(oscillator::get_osc_types(&osc_types_arg))
        .volume(instrument_volume)
        .channel()
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
