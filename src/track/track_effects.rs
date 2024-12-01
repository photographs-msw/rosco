use derive_builder::Builder;

use crate::envelope::envelope::Envelope;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::LFO;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackEffects {
    #[builder(default = "Vec::new()")]
    pub(crate) envelopes: Vec<Envelope>,

    #[builder(default = "Vec::new()")]
    pub(crate) lfos: Vec<LFO>,

    #[builder(default = "Vec::new()")]
    pub(crate) flangers: Vec<Flanger>,
}

pub(crate) fn no_op_effects() -> TrackEffects {
    TrackEffectsBuilder::default().build().unwrap()
}