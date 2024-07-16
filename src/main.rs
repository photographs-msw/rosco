mod audio_gen;
mod oscillator;
mod instrument;

use std::env;
use crate::instrument::Instrument;
use crate::oscillator::get_osc_types;

fn main() {
    let args = get_args();
    let osc_types_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let duration_ms: u64 = args[2].parse().unwrap();
    let oscillators = get_osc_types(&osc_types_arg);
    let instrument = Instrument::from_oscillators(oscillators);

    instrument.play_note(frequency, duration_ms);
}

fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    return args;
}
