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

    #[builder(public, setter(custom))]
    #[allow(dead_code)]
    pub (crate) end_time_ms: f32,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,

    #[builder(setter(custom), default = "INIT_START_TIME")]
    pub(crate) cur_playing_time_ms: f32,
}

#[allow(dead_code)]
impl Note {
    pub(crate) fn end_time_ms(&mut self) -> &mut Self {
        self.end_time_ms = self.start_time_ms + self.duration_ms;
        self
    }

    pub(crate) fn cur_playing_time_ms(&mut self, cur_playing_time_ms: f32) -> &mut Self {
        self.cur_playing_time_ms = cur_playing_time_ms;
        self
    }

    pub(crate) fn is_playing(&self, time_ms: f32) -> bool {
        time_ms >= self.start_time_ms && time_ms < self.end_time_ms
    }

    pub(crate) fn is_before_playing(&self, time_ms: f32) -> bool {
        time_ms < self.start_time_ms
    }

    pub(crate) fn is_after_playing(&self, time_ms: f32) -> bool {
        time_ms >= self.end_time_ms
    }
}