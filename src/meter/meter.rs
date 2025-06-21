use derive_builder::Builder;
use crate::common::float_utils::float_eq;
use crate::meter::durations;
use crate::meter::durations::DurationType;

pub(crate) static DEFAULT_BEAT_UNIT_DURATION_MS: f32 = 0.0;
pub(crate) static DEFAULT_BEAT_UNIT: DurationType = DurationType::Quarter;
pub(crate) static DEFAULT_TEMPO: u8 = 120;

// struct representing a musical meter
#[derive(Builder, Clone, Copy, Debug)]
#[builder(build_fn(validate = "Self::validate"))]
pub(crate) struct Meter {
    pub(crate) beats_per_measure: u8,
 
    #[builder(default = "DEFAULT_BEAT_UNIT")]
    pub(crate) beat_unit: durations::DurationType,
    
    // tempo in beat units per minute
    #[builder(default = "DEFAULT_TEMPO")]
    pub(crate) tempo: u8,

    // optional base fixed duration for a beat unit
    #[builder(default = "DEFAULT_BEAT_UNIT_DURATION_MS")]
    pub(crate) beat_unit_duration_ms: f32,
}

impl MeterBuilder {
    pub(crate) fn validate(&self) -> Result<Meter, String> {
        let beats_per_measure = self.beats_per_measure.ok_or("Beats per measure is required")?;
        if beats_per_measure == 0 {
            return Err(String::from("Meter: beats_per_measure must be greater than 0"));
        }

        let tempo = self.tempo.unwrap_or(DEFAULT_TEMPO);
        if tempo == 0 {
            return Err(String::from("Meter: tempo must be greater than 0"));
        }

        let beat_unit = self.beat_unit.unwrap_or(DEFAULT_BEAT_UNIT);
        let beat_unit_duration_ms = self.beat_unit_duration_ms.unwrap_or(DEFAULT_BEAT_UNIT_DURATION_MS);
        if beat_unit_duration_ms < 0.0 {
            return Err(String::from("Meter: beat_unit_duration_ms must not be negative"));
        }

        Ok(Meter {
            beats_per_measure,
            beat_unit,
            tempo,
            beat_unit_duration_ms 
        })
    }
}

impl Meter {
    #[allow(dead_code)]
    pub(crate) fn new(beats_per_measure: u8, beat_unit: DurationType, tempo: u8) -> Self {
        Self {
            beats_per_measure,
            beat_unit,
            tempo,
            beat_unit_duration_ms: DEFAULT_BEAT_UNIT_DURATION_MS
        }
    }

    // return the duration in ms of a note, converted
    #[allow(dead_code)]
    pub(crate) fn beat_duration(&self) -> f32 {
        60000.0 / (self.tempo as f32 * self.beat_unit.to_factor())
    }

    // return the duration of a note of a given duration type
    #[allow(dead_code)]
    pub(crate) fn note_duration(&self, duration_type: DurationType) -> f32 {
        let duration_factor: f32 = duration_type.to_factor() / self.beat_unit.to_factor();
        self.beat_duration() * duration_factor
    }
}

impl PartialEq for Meter {
    fn eq(&self, other: &Self) -> bool {
        self.beats_per_measure == other.beats_per_measure &&
        self.beat_unit == other.beat_unit &&
        self.tempo == other.tempo &&
        float_eq(self.beat_unit_duration_ms, other.beat_unit_duration_ms)
    }
}

impl Eq for Meter {}

#[cfg(test)]
mod test_meter {
    use super::*;

    #[test]
    fn test_meter_builder_success() {
        let meter = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        assert_eq!(meter.beats_per_measure, 4);
        assert_eq!(meter.beat_unit, DurationType::Quarter);
        assert_eq!(meter.tempo, 120);
    }

    #[test]
    fn test_meter_builder_common_time_signatures() {
        // 4/4 time at 120 BPM
        let meter_4_4 = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        // 3/4 time (waltz) at 180 BPM
        let meter_3_4 = MeterBuilder::default()
            .beats_per_measure(3)
            .beat_unit(DurationType::Quarter)
            .tempo(180)
            .build()
            .unwrap();

        // 6/8 time at 90 BPM
        let meter_6_8 = MeterBuilder::default()
            .beats_per_measure(6)
            .beat_unit(DurationType::Eighth)
            .tempo(90)
            .build()
            .unwrap();

        assert_eq!(meter_4_4.beats_per_measure, 4);
        assert_eq!(meter_3_4.beats_per_measure, 3);
        assert_eq!(meter_6_8.beats_per_measure, 6);
        assert_eq!(meter_6_8.beat_unit, DurationType::Eighth);
    }

    #[test]
    fn test_meter_builder_validation_zero_beats_per_measure() {
        let result = MeterBuilder::default()
            .beats_per_measure(0)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("beats_per_measure must be greater than 0"));
    }

    #[test]
    fn test_meter_builder_validation_zero_tempo() {
        let result = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(0)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tempo must be greater than 0"));
    }

    #[test]
    fn test_meter_builder_validation_negative_beat_unit_duration() {
        let result = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .beat_unit_duration_ms(-100.0)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("beat_unit_duration_ms must not be negative"));
    }

    #[test]
    fn test_meter_builder_with_custom_beat_unit_duration() {
        let meter = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .beat_unit_duration_ms(500.0)
            .build()
            .unwrap();

        assert_eq!(meter.beats_per_measure, 4);
        assert_eq!(meter.beat_unit, DurationType::Quarter);
        assert_eq!(meter.tempo, 120);
        assert_eq!(meter.beat_unit_duration_ms, 500.0);
    }

    #[test]
    fn test_meter_builder_default_values() {
        let meter = MeterBuilder::default()
            .beats_per_measure(4)
            .build()
            .unwrap();

        assert_eq!(meter.beats_per_measure, 4);
        assert_eq!(meter.beat_unit, DEFAULT_BEAT_UNIT);
        assert_eq!(meter.tempo, DEFAULT_TEMPO);
        assert_eq!(meter.beat_unit_duration_ms, DEFAULT_BEAT_UNIT_DURATION_MS);
    }

    #[test]
    fn test_meter_beat_duration_calculation() {
        let meter = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        // At 120 BPM with quarter note beat unit:
        // 60000ms / (120 * 0.25) = 60000 / 30 = 2000ms per beat
        let expected_beat_duration = 60000.0 / (120.0 * 0.25);
        assert_eq!(meter.beat_duration(), expected_beat_duration);
    }

    #[test]
    fn test_meter_note_duration_calculation() {
        let meter = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        // Whole note should be 4x the beat duration
        let whole_note_duration = meter.note_duration(DurationType::Whole);
        let expected_whole_duration = meter.beat_duration() * 4.0;
        assert_eq!(whole_note_duration, expected_whole_duration);

        // Eighth note should be 0.5x the beat duration
        let eighth_note_duration = meter.note_duration(DurationType::Eighth);
        let expected_eighth_duration = meter.beat_duration() * 0.5;
        assert_eq!(eighth_note_duration, expected_eighth_duration);
    }

    #[test]
    fn test_meter_equality() {
        let meter1 = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        let meter2 = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        assert_eq!(meter1, meter2);
    }

    #[test]
    fn test_meter_backward_compatibility_with_new() {
        // Test that the old `new` method still works
        let meter_new = Meter::new(4, DurationType::Quarter, 120);

        let meter_builder = MeterBuilder::default()
            .beats_per_measure(4)
            .beat_unit(DurationType::Quarter)
            .tempo(120)
            .build()
            .unwrap();

        assert_eq!(meter_new, meter_builder);
    }
}
