use std::hash::{Hash, Hasher};

use derive_builder::Builder;

use crate::constants::NO_TRACK;
use crate::float_utils::float_eq;

// We want to set INIT_START_TIME, INIT_END_TIME and DEFAULT_DURATION as builder defaults
// but this doesn't work with the custom setter for end_time_ms, which depends on start_time
// and duration being set. So we set them as constants but don't set defaults in the builder,
// so the user knows they have to set them.
pub(crate) static INIT_START_TIME: f32 = 0.0;
pub(crate) static INIT_END_TIME: f32 = 0.0;
pub(crate) static DEFAULT_DURATION: f32 = 0.0;
pub(crate) static DEFAULT_VOLUME: f32 = 1.0;

#[allow(dead_code)]
#[derive(Builder, Clone, Copy, Debug)]
pub(crate) struct Note {
    pub(crate) frequency: f32,
    pub(crate) duration_ms: f32,
    pub(crate) volume: f32,
    pub(crate) start_time_ms: f32,

    #[builder(setter(custom))]
    #[allow(dead_code)]
    pub (crate) end_time_ms: f32,

    #[builder(public, setter(custom))]
    pub(crate) track_num: i16,
}

pub(crate) fn default_note() -> Note {
    NoteBuilder::default()
        .frequency(0.0)
        .duration_ms(0.0)
        .volume(DEFAULT_VOLUME)
        .start_time_ms(INIT_START_TIME)
        .duration_ms(0.0)
        .end_time_ms()
        .no_track()
        .build().unwrap()
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        float_eq(self.frequency, other.frequency) &&
        float_eq(self.duration_ms, other.duration_ms) &&
        float_eq(self.volume, other.volume) &&
        float_eq(self.start_time_ms, other.start_time_ms) &&
        float_eq(self.end_time_ms, other.end_time_ms) &&
        self.track_num == other.track_num
    }
}
impl Eq for Note {}

impl Hash for Note {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frequency.to_bits().hash(state);
        self.duration_ms.to_bits().hash(state);
        self.volume.to_bits().hash(state);
        self.start_time_ms.to_bits().hash(state);
        self.end_time_ms.to_bits().hash(state);
        self.track_num.hash(state);
    }
}

#[allow(dead_code)]
impl NoteBuilder {
    pub(crate) fn end_time_ms(&mut self) -> &mut Self {
        if self.start_time_ms.is_none() {
            self.start_time_ms = Some(INIT_START_TIME);
        }
        if self.duration_ms.is_none() {
            self.duration_ms = Some(DEFAULT_DURATION);
        }
        let start_time_ms = self.start_time_ms.unwrap();
        let duration_ms = self.duration_ms.unwrap();
        self.end_time_ms = Some(start_time_ms + duration_ms);
        self
    }
    
    pub(crate) fn no_track(&mut self) -> &mut Self {
        self.track_num = Some(NO_TRACK);
        self
    }

    pub(crate) fn track_num(&mut self, track_num: i16) -> &mut Self {
        self.track_num = Some(track_num);
        self
    }
}

#[allow(dead_code)]
impl Note {
    pub(crate) fn is_playing(&self, time_ms: f32) -> bool {
        time_ms >= self.start_time_ms && time_ms < self.end_time_ms
    }

    pub(crate) fn is_before_playing(&self, time_ms: f32) -> bool {
        time_ms < self.start_time_ms
    }

    pub(crate) fn is_after_playing(&self, time_ms: f32) -> bool {
        time_ms >= self.end_time_ms
    }

    pub(crate) fn duration_position(&self, cur_time_ms: f32) -> f32 {
        (cur_time_ms - self.start_time_ms) / self.duration_ms
    }
}

#[cfg(test)]
mod test_note {
    use crate::note::NoteBuilder;

    #[test]
    fn test_is_playing() {
        let note = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();

        assert_eq!(note.is_playing(0.0), true);
        assert_eq!(note.is_playing(500.0), true);
        assert_eq!(note.is_playing(1000.0), false);
    }

    #[test]
    fn test_is_before_playing() {
        let note = setup_note()
            .start_time_ms(0.01)
            .end_time_ms()
            .build().unwrap();

        assert_eq!(note.is_before_playing(0.0), true);
        assert_eq!(note.is_before_playing(0.02), false);
    }

    #[test]
    fn test_is_after_playing() {
        let note = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();

        assert_eq!(note.is_after_playing(0.0), false);
        assert_eq!(note.is_after_playing(500.0), false);
        assert_eq!(note.is_after_playing(1000.0), true);
    }

    #[test]
    fn test_duration_position() {
        let note = setup_note()
            .start_time_ms(0.0)
            .end_time_ms()
            .build().unwrap();

        assert_eq!(note.duration_position(0.0), 0.0);
        assert_eq!(note.duration_position(500.0), 0.5);
        assert_eq!(note.duration_position(1000.0), 1.0);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .frequency(440.0)
            .duration_ms(1000.0)
            .volume(1.0)
            .no_track()
            .clone()
    }
}
