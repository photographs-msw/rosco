// We want to set INIT_START_TIME, INIT_END_TIME and DEFAULT_DURATION as builder defaults
// but this doesn't work with the custom setter for end_time_ms, which depends on start_time
// and duration being set. So we set them as constants but don't set defaults in the builder,
// so the user knows they have to set them.
pub(crate) static INIT_START_TIME: f32 = 0.0;
pub(crate) static INIT_END_TIME: f32 = 0.0;
pub(crate) static DEFAULT_FREQUENCY: f32 = 440.0;
pub(crate) static DEFAULT_DURATION: f32 = 0.0;
pub(crate) static DEFAULT_VOLUME: f32 = 1.0;
