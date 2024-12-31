use derive_builder::Builder;
use crate::effect::delay::Delay;
use crate::envelope::envelope::Envelope;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::LFO;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackEffects {
    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) envelopes: Vec<Envelope>,

    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) lfos: Vec<LFO>,

    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) flangers: Vec<Flanger>,

    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) delays: Vec<Delay>,
}

pub(crate) fn no_op_effects() -> TrackEffects {
    TrackEffectsBuilder::default().build().unwrap()
}