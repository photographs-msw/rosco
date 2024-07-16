mod audio_gen;
mod oscillator;
mod instrument;
mod utils;

use crate::instrument::Instrument;
use crate::oscillator::get_osc_types;
use crate::utils::get_args;

fn main() {
    let args = get_args();
    let osc_types_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let duration_ms: u64 = args[2].parse().unwrap();
    let oscillators = get_osc_types(&osc_types_arg);
    let instrument = Instrument::from_oscillators(oscillators);

    instrument.play_note(frequency, duration_ms);
}
