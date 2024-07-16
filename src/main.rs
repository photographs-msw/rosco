mod gen_audio;
mod oscillator;

use std::env;
use crate::gen_audio::gen_note;

fn main() {
    let args = get_args();
    let osc_types_arg = args[0].clone();
    let frequency: f32 = args[1].parse().unwrap();
    let duration_ms: u64 = args[2].parse().unwrap();

    gen_note(&osc_types_arg, frequency, duration_ms);
}

fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    return args;
}
