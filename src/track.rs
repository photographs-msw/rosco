use derive_builder::Builder;

use crate::sequence::Sequence;

static DEFAULT_TRACK_NAME: &str = "track_name";
static DEFAULT_TRACK_VOLUME: f32 = 1.0;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
pub(crate) struct Track {
    #[builder(default = "String::from(DEFAULT_TRACK_NAME)")]
    pub(crate) name: String,

    #[builder(default = "DEFAULT_TRACK_VOLUME")]
    pub(crate) volume: f32,

    pub(crate) sequence: Sequence,
}

impl Track {}
