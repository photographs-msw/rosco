use std::hash::{Hash, Hasher};

use crate::common::float_utils::float_eq;

#[derive(Clone, Copy, Debug)]
pub(crate) struct EnvelopePair (
    pub(crate) f32,  // position in the note duration as "percentage" in range 0.0 to 1.0
    pub(crate) f32,  // volume level 0.0 to 1.0
);

impl PartialEq for EnvelopePair {
    fn eq(&self, other: &Self) -> bool {
        float_eq(self.0, other.0) && float_eq(self.1, other.1)
    }
}
impl Eq for EnvelopePair {}

impl Hash for EnvelopePair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
        self.1.to_bits().hash(state);
    }
}
