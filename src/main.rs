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

use crate::composition::computer_punk_003;

fn main() {
    computer_punk_003::play();
}
