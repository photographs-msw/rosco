mod audio_gen;
mod instrument;
mod multi_instrument;
mod note;
mod oscillator;
mod utils;
mod sequence;
mod channel;

use crate::instrument::Instrument;
use crate::multi_instrument::MultiInstrument;
use crate::note::Note;
use crate::oscillator::get_osc_types;
use crate::utils::get_args;

fn main() {
    let args = get_args();
    let osc_types_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let volume: f32 = args[2].parse().unwrap();
    let duration_ms: u64 = args[3].parse().unwrap();

    let oscillators_1 = get_osc_types(&osc_types_arg);
    let oscillators_2 = get_osc_types(&osc_types_arg);
    let channel_oscillators = vec![oscillators_1, oscillators_2];
    let mut multi_instrument =
        MultiInstrument::from_channel_oscillators(channel_oscillators);
    let note_1 = Note::from(frequency, volume * 0.5, duration_ms);
    let note_2 = Note::from(frequency / 2.0, volume * 0.75, duration_ms);
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

    let mut instrument = Instrument::from_oscillators(get_osc_types(&osc_types_arg));
    let note_3 = Note::from(frequency, volume * 0.5, duration_ms);
    let note_4 = Note::from(frequency / 2.0, volume * 0.75, duration_ms);
    instrument.add_note(note_3);
    instrument.add_note(note_4);
    instrument.play_note_and_advance();
    instrument.set_volume(0.25);
    instrument.play_note();
    instrument.set_volume(0.75);
    instrument.loop_once();
    instrument.loop_n(2);
    instrument.play_note_direct(&note_1);
}
