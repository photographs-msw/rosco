use derive_builder::Builder;

use crate::constants::NO_TRACK;
use crate::sequence::Sequence;

static DEFAULT_TRACK_VOLUME: f32 = 1.0;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
pub(crate) struct Track {
    #[builder(default = "NO_TRACK")]
    pub(crate) num: i16,

    #[builder(default = "DEFAULT_TRACK_VOLUME")]
    pub(crate) volume: f32,

    pub(crate) sequence: Sequence,
}

impl Track {}
