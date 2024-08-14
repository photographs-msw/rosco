use derive_builder::Builder;

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
    pub(crate) start: EnvelopePair,

    // These three attributes control the shape of the envelope
    //          attack
    //       -          -   decay  --  sustain  -
    // start                                       release
    pub(crate) attack: EnvelopePair,
    pub(crate) decay: EnvelopePair,
    pub(crate) sustain: EnvelopePair,

    #[builder(default = "EnvelopePair(0.0, 1.0)")]
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

#[allow(dead_code)]
impl Envelope {

    pub(crate) fn volume_factor(&self, position: f32) -> f32 {
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
