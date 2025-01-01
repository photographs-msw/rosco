use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Pair<T> (
    pub(crate) T,
    pub(crate) T,
);

// impl<T> PartialEq for Pair<T> {
//     fn eq(&self, other: &Self) -> bool {
//         float_eq(self.0, other.0) && float_eq(self.1, other.1)
//     }
// }
// impl<T> Eq for Pair<T> {}
// 
// impl<T> Hash for Pair<T> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.0.to_bits().hash(state);
//         self.1.to_bits().hash(state);
//     }
// }
