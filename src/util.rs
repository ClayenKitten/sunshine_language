//! Various utility functions and types.

mod monotonic;
mod span;

pub use monotonic::MonotonicVec;
pub use span::Span;

/// Count number of token trees.
macro_rules! count {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { (count!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { count!($($a)*) << 1 };
}
pub(crate) use count;
