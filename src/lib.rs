//! An iterator adapter that allows you to efficiently peek the nth item of an iterator.
//!
//! Itermediate values are memoized and heap allocations are avoided when possible.
//!
//! ## Usage
//!
//! ```rust
//! extern crate peek_nth;
//!
//! use peek_nth::IteratorExt;
//!
//! fn main() {
//!     let mut iter = "Hello, world!".chars().peekable_nth();
//!
//!     assert_eq!(iter.peek_nth(4), Some(&'o'));
//!     assert_eq!(iter.peek_nth(3), Some(&'l'));
//!     assert_eq!(iter.peek_nth(2), Some(&'l'));
//!     assert_eq!(iter.peek_nth(1), Some(&'e'));
//!     assert_eq!(iter.peek_nth(0), Some(&'H'));
//!     assert_eq!(iter.peek_nth(7), Some(&'w'));
//!     assert_eq!(iter.collect::<String>(), "Hello, world!");
//! }
//!```

#[cfg(feature = "smallvec")]
extern crate smallvec;
#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

#[cfg(not(feature = "smallvec"))]
use std::collections::VecDeque;

use std::iter::{DoubleEndedIterator, ExactSizeIterator};


/// Adds a peekable_nth() method to types that implement [`std::iter::Iterator`].
///
/// [`std::iter::Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
pub trait IteratorExt: Iterator + Sized {
    fn peekable_nth(self) -> PeekableNth<Self>;
}

/// An iterator with a peek_nth() method that returns an optional reference to the nth element.
#[derive(Clone, Debug)]
pub struct PeekableNth<I>
where
    I: Iterator,
{
    iter: I,
    #[cfg(feature = "smallvec")]
    next: SmallVec<[I::Item; 64]>,
    #[cfg(not(feature = "smallvec"))]
    next: VecDeque<I::Item>,
}

impl<I> IteratorExt for I
where
    I: Iterator,
{
    #[inline]
    fn peekable_nth(self) -> PeekableNth<I> {
        PeekableNth {
            iter: self,
            #[cfg(feature = "smallvec")]
            next: SmallVec::new(),
            #[cfg(not(feature = "smallvec"))]
            next: VecDeque::new(),
        }
    }
}

impl<I> PeekableNth<I>
where
    I: Iterator,
{
    /// Returns a reference to the next value without advancing the iterator.
    #[inline]
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_nth(0)
    }

    /// Returns a reference to the nth(n) value without advancing the iterator.
    #[inline]
    pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
        for _ in self.next.len()..=n {
            #[cfg(feature = "smallvec")]
            self.next.push(self.iter.next()?);
            #[cfg(not(feature = "smallvec"))]
            self.next.push_back(self.iter.next()?);
        }

        self.next.get(n)
    }
}

impl<I> DoubleEndedIterator for PeekableNth<I>
where
    I: DoubleEndedIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<I::Item> {
        match self.iter.next_back() {
            #[cfg(feature = "smallvec")]
            None if !self.next.is_empty() => self.next.pop(),
            #[cfg(not(feature = "smallvec"))]
            None if !self.next.is_empty() => self.next.pop_back(),
            option => option,
        }
    }
}

impl<I> ExactSizeIterator for PeekableNth<I>
where
    I: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I> Iterator for PeekableNth<I>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.next.is_empty() {
            self.iter.next()
        } else {
            #[cfg(feature = "smallvec")]
            return Some(self.next.remove(0));
            #[cfg(not(feature = "smallvec"))]
            return self.next.pop_front();
        }
    }
}
