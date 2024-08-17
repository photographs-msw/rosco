use std::hash::{Hash, Hasher};

use derive_builder::Builder;
use float_eq::float_eq;

use crate::constants::{FLOAT_EQ_TOLERANCE, NO_TRACK};
use crate::envelope::{Envelope, EnvelopePair};

pub(crate) static INIT_START_TIME: f32 = 0.0;
pub(crate) static DEFAULT_VOLUME: f32 = 1.0;

#[allow(dead_code)]
#[derive(Builder, Clone, Copy, Debug)]
pub(crate) struct Note {
    pub(crate) frequency: f32,
    pub(crate) duration_ms: f32,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) start_time_ms: f32,

    #[builder(setter(custom))]
    #[allow(dead_code)]
    pub (crate) end_time_ms: f32,

    // user can call default_envelope() to build with no-op envelope or can add custom envelope
    #[builder(public, setter(custom))]
    pub(crate) envelope: Envelope,

    #[builder(public, setter(custom))]
    pub(crate) track_num: i16,
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

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        float_eq!(self.frequency, other.frequency, rmax <= FLOAT_EQ_TOLERANCE) &&
            float_eq!(self.duration_ms, other.duration_ms, rmax <= FLOAT_EQ_TOLERANCE) &&
            float_eq!(self.volume, other.volume, rmax <= FLOAT_EQ_TOLERANCE) &&
            float_eq!(self.start_time_ms, other.end_time_ms, rmax <= FLOAT_EQ_TOLERANCE) &&
            float_eq!(self.end_time_ms, other.end_time_ms, rmax <= FLOAT_EQ_TOLERANCE) &&
            self.track_num == other.track_num
    }
}

#[allow(dead_code)]
impl NoteBuilder {
    pub(crate) fn end_time_ms(&mut self) -> &mut Self {
        let start_time_ms = self.start_time_ms.unwrap();
        let duration_ms = self.duration_ms.unwrap();
        self.end_time_ms = Some(start_time_ms + duration_ms);
        self
    }

    pub (crate) fn envelope(&mut self, envelope: Envelope) -> &mut Self {
        self.envelope = Some(envelope);
        self
    }

    // overriding setting in builder allowing the caller to add default no-op envelope on build
    pub(crate) fn default_envelope(&mut self) -> &mut Self {
        self.envelope = Some(Envelope {
            start: EnvelopePair(0.0, 0.0),
            attack: EnvelopePair(0.5, 1.0),
            decay: EnvelopePair(0.5, 1.0),
            sustain: EnvelopePair(0.5, 1.0),
            release: EnvelopePair(1.0, 0.0),
        });
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

pub(crate) fn max_note_duration_ms(notes: &Vec<Note>) -> u64 {
    notes.iter()
        .map(|note| note.duration_ms as u64)
        .max()
        .unwrap()
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
            .default_envelope()
            .no_track()
            .clone()
    }
}
