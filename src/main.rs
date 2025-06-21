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
mod composition;
mod meter;
mod dsl;

// use crate::composition::computer_punk_001;
use crate::composition::dsl_1;

fn main() {
    // computer_punk_001::play();
    dsl_1::play();
}
