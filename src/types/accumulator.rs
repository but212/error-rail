use crate::types::ErrorVec;
use core::fmt::Debug;
use core::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A common abstraction for accumulating items (errors or contexts).
///
/// This struct wraps the underlying storage (currently `ErrorVec`) to provide
/// a consistent interface for accumulation logic across `ErrorPipeline` and `Validation`.
///
/// # Type Parameters
///
/// * `T` - The type of items to accumulate
///
/// # Examples
///
/// ```
/// use error_rail::types::accumulator::Accumulator;
///
/// let mut acc = Accumulator::new();
/// acc.push("error1");
/// acc.push("error2");
/// assert_eq!(acc.len(), 2);
///
/// // Create from iterator
/// let acc: Accumulator<&str> = ["a", "b", "c"].into_iter().collect();
/// assert_eq!(acc.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Accumulator<T> {
    items: ErrorVec<T>,
}

impl<T> Accumulator<T> {
    /// Creates a new empty accumulator.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc: Accumulator<String> = Accumulator::new();
    /// assert!(acc.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self { items: ErrorVec::new() }
    }

    /// Adds a single item to the accumulator.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to add
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let mut acc = Accumulator::new();
    /// acc.push("first error");
    /// acc.push("second error");
    /// assert_eq!(acc.len(), 2);
    /// ```
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the last item, or `None` if empty.
    ///
    /// # Returns
    ///
    /// * `Some(T)` - The last item if the accumulator is not empty
    /// * `None` - If the accumulator is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let mut acc = Accumulator::single("error");
    /// assert_eq!(acc.pop(), Some("error"));
    /// assert_eq!(acc.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Extends the accumulator with items from an iterator.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator yielding items to add
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let mut acc = Accumulator::new();
    /// acc.extend(["error1", "error2", "error3"]);
    /// assert_eq!(acc.len(), 3);
    /// ```
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items.extend(iter);
    }

    /// Returns `true` if the accumulator contains no items.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc: Accumulator<&str> = Accumulator::new();
    /// assert!(acc.is_empty());
    ///
    /// let acc = Accumulator::single("error");
    /// assert!(!acc.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the accumulator.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc: Accumulator<&str> = ["a", "b", "c"].into_iter().collect();
    /// assert_eq!(acc.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns an iterator over references to the items.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc: Accumulator<i32> = [1, 2, 3].into_iter().collect();
    /// let sum: i32 = acc.iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.items.iter()
    }

    /// Returns a mutable iterator over the items.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let mut acc: Accumulator<i32> = [1, 2, 3].into_iter().collect();
    /// for item in acc.iter_mut() {
    ///     *item *= 2;
    /// }
    /// assert_eq!(acc.into_inner().as_slice(), &[2, 4, 6]);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.items.iter_mut()
    }

    /// Consumes the accumulator and returns the underlying `ErrorVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc: Accumulator<&str> = ["a", "b"].into_iter().collect();
    /// let vec = acc.into_inner();
    /// assert_eq!(vec.len(), 2);
    /// ```
    #[inline]
    pub fn into_inner(self) -> ErrorVec<T> {
        self.items
    }

    /// Creates an accumulator with a single item.
    ///
    /// This is a convenience method equivalent to creating a new accumulator
    /// and pushing one item.
    ///
    /// # Arguments
    ///
    /// * `item` - The single item to store
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc = Accumulator::single("only error");
    /// assert_eq!(acc.len(), 1);
    /// ```
    #[inline]
    pub fn single(item: T) -> Self {
        let mut acc = Self::new();
        acc.push(item);
        acc
    }

    /// Merges another accumulator into this one.
    ///
    /// All items from `other` are appended to `self`, consuming `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The accumulator to merge into this one
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let mut acc1: Accumulator<&str> = ["a", "b"].into_iter().collect();
    /// let acc2: Accumulator<&str> = ["c", "d"].into_iter().collect();
    /// acc1.merge(acc2);
    /// assert_eq!(acc1.len(), 4);
    /// ```
    #[inline]
    pub fn merge(&mut self, other: Self) {
        self.extend(other);
    }

    /// Maps each item in the accumulator using the provided function.
    ///
    /// Consumes the accumulator and returns a new one with transformed items.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms each item from type `T` to type `U`
    ///
    /// # Type Parameters
    ///
    /// * `F` - The mapping function type
    /// * `U` - The output item type
    ///
    /// # Examples
    ///
    /// ```
    /// use error_rail::types::accumulator::Accumulator;
    ///
    /// let acc: Accumulator<i32> = [1, 2, 3].into_iter().collect();
    /// let doubled = acc.map(|x| x * 2);
    /// assert_eq!(doubled.into_inner().as_slice(), &[2, 4, 6]);
    /// ```
    #[inline]
    pub fn map<F, U>(self, f: F) -> Accumulator<U>
    where
        F: FnMut(T) -> U,
    {
        Accumulator { items: self.items.into_iter().map(f).collect() }
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
        Self { items: iter.into_iter().collect() }
    }
}

impl<T> IntoIterator for Accumulator<T> {
    type Item = T;
    type IntoIter = <ErrorVec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
