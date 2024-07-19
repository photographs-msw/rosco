#[allow(dead_code)]
static DEFAULT_VOLUME: f32 = 1.0;

// TODO DO WE NEED THESE TRAITS? GET OWNERSHIP RIGHT
#[derive(Clone, Copy)]
pub(crate) struct Note {
    pub(crate) frequency: f32,
    pub(crate) volume: f32,
    pub(crate) duration_ms: u64,
}

impl Note {

    pub(crate) fn from(frequency: f32, volume: f32, duration_ms: u64) -> Self {
        Note {
            frequency,
            volume,
            duration_ms,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_frequency_duration(frequency: f32, duration_ms: u64) -> Self {
        Note {
            frequency,
            volume: DEFAULT_VOLUME,
            duration_ms,
        }
    }
}