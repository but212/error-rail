use crate::types::ErrorVec;
use core::fmt::Debug;
use core::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A common abstraction for accumulating items (errors or contexts).
///
/// This struct wraps the underlying storage (currently `ErrorVec`) to provide
/// a consistent interface for accumulation logic across `ErrorPipeline` and `Validation`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Accumulator<T> {
    items: ErrorVec<T>,
}

impl<T> Accumulator<T> {
    /// Creates a new empty accumulator.
    #[inline]
    pub fn new() -> Self {
        Self {
            items: ErrorVec::new(),
        }
    }

    /// Adds a single item to the accumulator.
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the last item, or None if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Extends the accumulator with items from an iterator.
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items.extend(iter);
    }

    /// Returns true if the accumulator is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the accumulator.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns an iterator over the items.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.items.iter()
    }

    /// Returns a mutable iterator over the items.
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.items.iter_mut()
    }

    /// Consumes the accumulator and returns the underlying `ErrorVec`.
    #[inline]
    pub fn into_inner(self) -> ErrorVec<T> {
        self.items
    }
}

impl<T: PartialOrd> PartialOrd for Accumulator<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.items.partial_cmp(&other.items)
    }
}

impl<T: Ord> Ord for Accumulator<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.items.cmp(&other.items)
    }
}

impl<T: Hash> Hash for Accumulator<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl<T> From<ErrorVec<T>> for Accumulator<T> {
    fn from(items: ErrorVec<T>) -> Self {
        Self { items }
    }
}

impl<T> FromIterator<T> for Accumulator<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl<T> IntoIterator for Accumulator<T> {
    type Item = T;
    type IntoIter = smallvec::IntoIter<[T; 2]>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
