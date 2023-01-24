use std::ops::{Index, IndexMut};

/// Various utility functions and types.

/// A `MonotonicVec` is a `Vec` which can only be grown.
///
/// Once inserted, an element can never be removed or swapped, guaranteeing that any indices into a `MonotonicVec` are stable.
///
/// Inspired by [rustc](https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_span/source_map.rs.html#52)'s internal data structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MonotonicVec<T>(Vec<T>);

impl<T> MonotonicVec<T> {
    pub fn new() -> MonotonicVec<T> {
        MonotonicVec(Vec::new())
    }

    pub fn push(&mut self, val: T) {
        self.0.push(val);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> From<Vec<T>> for MonotonicVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> Index<usize> for MonotonicVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T> IndexMut<usize> for MonotonicVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}
