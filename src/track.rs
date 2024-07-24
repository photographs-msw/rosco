use derive_builder::Builder;

use crate::sequence::Sequence;

#[derive(Builder, Clone)]
pub(crate) struct Track {
    #[builder(default = "track_name")]
    pub(crate) name: String,

    pub(crate) sequence: Sequence,
    pub(crate) volume: f32
}

impl Track {}
