use derive_builder::Builder;

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
            start: EnvelopePair(1.0, 1.0),
            attack: EnvelopePair(1.0, 1.0),
            decay: EnvelopePair(1.0, 1.0),
            sustain: EnvelopePair(1.0, 1.0),
            release: EnvelopePair(1.0, 1.0),
        });
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EnvelopePair (
    pub(crate) f32,  // position in the note duration as "percentage", again in range 0.0 to 1.0
    pub(crate) f32,  // volume level 0.0 to 1.0
);

// State for an ADSR envelope. User sets the position from the start where attack, decay, sustain
// and release end, and the volume level at each of these positions. The envelope defaults to
// starting from (0, 0) and connecting from their to start, and connecting from the position
// of the end of sustain to the end of the note, which is the release.
#[allow(dead_code)]
#[derive(Builder, Clone, Copy, Debug)]
#[builder(build_fn(validate = "Self::validate"))]
pub(crate) struct Envelope {
    #[builder(default = "EnvelopePair(0.0, 0.0)")]
    start: EnvelopePair,

    // These three attributes control the shape of the envelope
    //          attack
    //       -          -   decay  --  sustain  - 
    // start                                       release
    attack: EnvelopePair,
    decay: EnvelopePair,
    sustain: EnvelopePair,

    #[builder(default = "EnvelopePair(0.0, 1.0)")]
    release: EnvelopePair,
}

impl EnvelopeBuilder {
    pub(crate) fn validate(&self) -> Result<Envelope, String> {
        let attack = self.attack.unwrap();
        let decay = self.decay.unwrap();
        let sustain = self.sustain.unwrap();

        if attack.0 > decay.0 || decay.0 > sustain.0  {
            return Err(
                String::from("Envelope: attack, decay, sustain, release must be in order"));
        }
        if attack.0 < 0.0 || attack.0 > 1.0 || attack.1 < 0.0 || attack.1 > 1.0 {
            return Err(
                String::from("Envelope: attack position and volume must be between 0.0 and 1.0"));
        }
        if decay.0 < 0.0 || decay.0 > 1.0 || decay.1 < 0.0 || decay.1 > 1.0 {
            return Err(
                String::from("Envelope: decay position and volume must be between 0.0 and 1.0"));
        }
        if sustain.0 < 0.0 || sustain.0 > 1.0 || sustain.1 < 0.0 || sustain.1 > 1.0 {
            return Err(
                String::from("Envelope: sustain position and volume must be between 0.0 and 1.0"));
        }

        Ok(Envelope {
            start: EnvelopePair(0.0, 0.0),
            attack,
            decay,
            sustain,
            release: EnvelopePair(1.0, 0.0),
        })
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

#[allow(dead_code)]
impl Envelope {

    pub(crate) fn volume_for_duration_position(&self, position: f32) -> f32 {
        if position < self.attack.0 {
            self.volume_for_segment_position(self.start, self.attack, position)
        } else if position < self.decay.0 {
            self.volume_for_segment_position(self.attack, self.decay, position)
        } else if position < self.sustain.0 {
            self.volume_for_segment_position(self.decay, self.sustain, position)
        } else {
            self.volume_for_segment_position(self.sustain, self.release, position)
        }
    }

    fn volume_for_segment_position(&self, start: EnvelopePair, end: EnvelopePair,
                                   position: f32) -> f32 {
        let start_position= start.0;
        let start_volume = start.1;
        let end_position = end.0;
        let end_volume= end.1;

        let slope = (end_volume - start_volume) / (end_position - start_position);
        let intercept = start_volume - slope * start_position;
        // y = mx + b, where slope = m and intercept = b
        // so the value along the line for any position
        slope * position + intercept
    }
}

pub(crate) fn max_note_duration_ms(notes: &Vec<Note>) -> u64 {
    notes.iter()
        .map(|note| note.duration_ms as u64)
        .max()
        .unwrap()
}

mod test_note {
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
                .default_envelope()
                .clone()
        }
    }
}
