use derive_builder::Builder;

use crate::sequence::Sequence;

#[derive(Builder, Clone)]
pub(crate) struct Channel {
    pub(crate) sequence: Sequence,
    pub(crate) volume: f32
}

impl Channel {}
