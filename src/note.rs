use derive_builder::Builder;

pub(crate) static INIT_START_TIME: f32 = 0.0;
pub(crate) static DEFAULT_VOLUME: f32 = 1.0;

#[derive(Builder, Clone, Copy, Debug)]
pub(crate) struct Note {
    pub(crate) frequency: f32,
    pub(crate) duration_ms: f32,

    #[builder(default = "INIT_START_TIME")]
    #[allow(dead_code)]
    pub(crate) start_time_ms: f32,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,
}

impl Note {}