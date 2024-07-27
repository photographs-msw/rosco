use derive_builder::Builder;

use crate::sequence::Sequence;

static DEFAULT_TRACK_NAME: &str = "track_name";

#[derive(Builder, Clone)]
pub(crate) struct Track {
    #[builder(default = "String::from(DEFAULT_TRACK_NAME)")]
    pub(crate) name: String,

    pub(crate) sequence: Sequence,
    pub(crate) volume: f32
}

impl Track {}
