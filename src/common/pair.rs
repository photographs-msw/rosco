use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(dead_code)]
pub(crate) struct Pair<T> (
    pub(crate) T,
    pub(crate) T,
);
