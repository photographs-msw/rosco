use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Pair<T> (
    pub(crate) T,
    pub(crate) T,
);
