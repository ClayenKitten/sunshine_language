//! Various utility functions and types.

use std::ops::{Index, IndexMut};

/// A `MonotonicVec` is a [`Vec`] which can only be grown.
///
/// Once inserted, an element can never be removed or swapped, guaranteeing that any indices into a `MonotonicVec` are stable.
///
/// Inspired by [rustc](https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_span/source_map.rs.html#52)'s internal data structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MonotonicVec<T>(Vec<T>);

impl<T> MonotonicVec<T> {
    /// Constructs a new, empty `MonotonicVec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    pub fn new() -> MonotonicVec<T> {
        MonotonicVec(Vec::new())
    }

    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn push(&mut self, val: T) {
        self.0.push(val);
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

/// Count number of token trees.
macro_rules! count {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { (count!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { count!($($a)*) << 1 };
}
pub(crate) use count;
