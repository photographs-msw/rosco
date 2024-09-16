use std::hash::Hash;

use derive_builder::Builder;

use crate::envelope_pair::EnvelopePair;
use crate::sample_effect_trait::{ApplyEffect, BuilderWrapper, CloneWrapper, NoOpEffect};

// State for an ADSR envelope. User sets the position from the start where attack, decay, sustain
// and release end, and the volume level at each of these positions. The envelope defaults to
// starting from (0, 0) and connecting from their to start, and connecting from the position
// of the end of sustain to the end of the note, which is the release.
#[allow(dead_code)]
#[derive(Builder, Clone, Copy, Debug, Hash)]
#[builder(build_fn(validate = "Self::validate"))]
pub(crate) struct Envelope {
    #[builder(default = "EnvelopePair(0.0, 0.0)")]
    pub(crate) start: EnvelopePair,

    // These three attributes control the shape of the envelope
    //          attack
    //       -          -   decay  --  sustain  -
    // start                                       release
    pub(crate) attack: EnvelopePair,
    pub(crate) decay: EnvelopePair,
    pub(crate) sustain: EnvelopePair,

    #[builder(default = "EnvelopePair(1.0, 0.0)")]
    pub(crate) release: EnvelopePair,
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

pub (crate) fn default_envelope() -> Envelope {
    Envelope {
        start: EnvelopePair(0.0, 0.0),
        attack: EnvelopePair(0.02, 1.0),
        decay: EnvelopePair(0.51, 1.0),
        sustain: EnvelopePair(0.98, 1.0),
        release: EnvelopePair(1.0, 0.0),
    }
}

#[allow(dead_code)]
impl Envelope {
    fn volume_factor(&self, position: f32) -> f32 {
        // if position < 0.0 || position > 1.0 {
        //     println!("POSITION: {}", position);
        //     panic!("Envelope: position must be between 0.0 and 1.0");
        // }
        
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
        let intercept = start_volume - (slope * start_position);
        // y = mx + b, where slope = m and intercept = b, so b = y - mx
        // so the value along the line for any position
        slope * position + intercept
    }
}

impl PartialEq for Envelope {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start &&
            self.attack == other.attack &&
            self.decay == other.decay &&
            self.sustain == other.sustain &&
            self.release == other.release
    }
}
impl Eq for Envelope {}

impl ApplyEffect for Envelope {
    fn apply_effect(&self, sample: f32, _frequency: f32, sample_clock: f32) -> f32 {
       sample * self.volume_factor(sample_clock)
    }
}

impl CloneWrapper<Envelope> for Envelope {
    fn clone(&self) -> Envelope {
        *self
    }
}

impl BuilderWrapper<Envelope> for Envelope {
    fn new() -> Envelope {
        default_envelope()
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct NoOpEnvelope {}
impl NoOpEffect for NoOpEnvelope {}

impl ApplyEffect for NoOpEnvelope {
    fn apply_effect(&self, sample: f32, _frequency: f32, _sample_clock: f32) -> f32 {
        self.no_op(sample, _frequency, _sample_clock) 
    }
}

impl CloneWrapper<NoOpEnvelope> for NoOpEnvelope {
    fn clone(&self) -> NoOpEnvelope {
        *self
    }
}

impl BuilderWrapper<NoOpEnvelope> for NoOpEnvelope {
    fn new() -> NoOpEnvelope {
        NoOpEnvelope {}
    }
}


#[cfg(test)]
mod test_envelope {
    use crate::envelope::{EnvelopeBuilder, EnvelopePair};
    use crate::float_utils::assert_float_eq;

    #[test]
    fn test_volume_factor() {
       let envelope = EnvelopeBuilder::default()
           .attack(EnvelopePair(0.3, 0.9))
           .decay(EnvelopePair(0.35, 0.7))
           .sustain(EnvelopePair(0.6, 0.65))
           .build().unwrap();

        assert_float_eq(envelope.volume_factor(0.0), 0.0);
        assert_float_eq(envelope.volume_factor(0.15), 0.45);
        assert_float_eq(envelope.volume_factor(0.3), 0.9);
        assert_float_eq(envelope.volume_factor(0.35), 0.7);
        assert_float_eq(envelope.volume_factor(0.6), 0.65);
        assert_float_eq(envelope.volume_factor(0.8), 0.325);
        assert_float_eq(envelope.volume_factor(1.0), 0.0);
    }
}
