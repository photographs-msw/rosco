use crate::envelope::Envelope;
use crate::oscillator::Waveform;

pub(crate) trait NoteEnvelope {
    fn envelope(&self) -> Option<Envelope>;
    fn has_envelope(&self) -> bool;
}

#[allow(dead_code)]
pub(crate) trait NoteOscillator{
    fn waveforms(&self) -> Option<Vec<Waveform>>;
    // TODO DO WE NEED THIS OR JUST THE TRAIT?
    fn has_waveforms(&self) -> bool;
}

