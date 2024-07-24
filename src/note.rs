use derive_builder::Builder;
use nodi::midly::num::u28;

pub(crate) static DEFAULT_VOLUME: f32 = 1.0;

#[derive(Builder, Clone, Copy)]
pub(crate) struct Note {
    pub(crate) frequency: f32,
    pub(crate) duration_ms: u64,

    #[builder(default = "0")]
    pub(crate) start_time_ms: u64,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,
}

impl Note {}