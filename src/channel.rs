use crate::sequence::Sequence;

pub(crate) struct Channel {
    pub(crate) sequence: Sequence,
    pub(crate) volume: f32
}

impl Channel {

    pub(crate) fn from(sequence: Sequence, volume: f32) -> Self {
        Channel {
            sequence,
            volume
        }
    }
}
