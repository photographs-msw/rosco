use crate::envelope::Envelope;
use crate::oscillator::Waveform;

pub(crate) trait NoteEnvelope {
    fn envelope(&self) -> Option<Envelope>;
    fn has_envelope(&self) -> bool;
}

pub(crate) trait NoteOscillator{
    fn waveforms(&self) -> Option<Vec<Waveform>>;
    fn has_waveforms(&self) -> bool;
}

