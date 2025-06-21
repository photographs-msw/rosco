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

use crate::composition::dsl_1;
// use crate::composition::computer_punk_001;
// use crate::composition::computer_punk_003;

fn main() {
    dsl_1::play();
    // computer_punk_001::play();
    // computer_punk_003::play();
}
