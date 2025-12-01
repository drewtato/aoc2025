use std::cmp::Ordering;
use std::ops::RangeInclusive;

use num_integer::Integer;

/// Like [`std::iter::Sum`] but doesn't require specifying the output type.
pub trait BetterSum {
    type Output;
    fn bsum<I: IntoIterator<Item = Self>>(iter: I) -> Self::Output;
}

macro_rules! impl_better_sum {
    ($($t:ty),* $(,)?) => {$(
        impl BetterSum for $t {
            type Output = $t;

            fn bsum<I: IntoIterator<Item = Self>>(iter: I) -> Self::Output {
                iter.into_iter().sum()
            }
        }

        impl<'a> BetterSum for &'a $t {
            type Output = $t;

            fn bsum<I: IntoIterator<Item = Self>>(iter: I) -> Self::Output {
                iter.into_iter().sum()
            }
        }
    )*};
}

impl_better_sum! { i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize }

/// Extension trait to add the `bsum` method to iterators.
pub trait BetterSumIter {
    type Output;
    fn bsum(self) -> Self::Output;
}

impl<I, S> BetterSumIter for I
where
    I: Iterator<Item = S>,
    S: BetterSum,
{
    type Output = S::Output;

    fn bsum(self) -> Self::Output {
        S::bsum(self)
    }
}

/// Like [`std::iter::Product`] but doesn't require specifying the output type.
pub trait BetterProduct {
    type Output;
    fn bproduct<I: IntoIterator<Item = Self>>(iter: I) -> Self::Output;
}

macro_rules! impl_better_product {
    ($($t:ty),* $(,)?) => {$(
        impl BetterProduct for $t {
            type Output = $t;

            fn bproduct<I: IntoIterator<Item = Self>>(iter: I) -> Self::Output {
                iter.into_iter().product()
            }
        }

        impl<'a> BetterProduct for &'a $t {
            type Output = $t;

            fn bproduct<I: IntoIterator<Item = Self>>(iter: I) -> Self::Output {
                iter.into_iter().product()
            }
        }
    )*};
}

impl_better_product! { i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize  }

/// Extension trait to add the `bproduct` method to iterators.
pub trait BetterProductIter {
    type Output;
    fn bproduct(self) -> Self::Output;
}

impl<I, S> BetterProductIter for I
where
    I: Iterator<Item = S>,
    S: BetterProduct,
{
    type Output = S::Output;

    fn bproduct(self) -> Self::Output {
        S::bproduct(self)
    }
}

/// A double-ended, inclusive range that can go either low to high or high
/// to low.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReversibleRange<I> {
    start: I,
    end: I,
}

impl<I> ReversibleRange<I> {
    pub fn new(range: RangeInclusive<I>) -> Self {
        let (start, end) = range.into_inner();
        Self { start, end }
    }

    pub fn contains(&self, value: &I) -> bool
    where
        I: PartialOrd,
    {
        match self.start.partial_cmp(&self.end) {
            Some(Ordering::Less) => *value >= self.start && *value <= self.end,
            Some(Ordering::Equal) => *value == self.start,
            Some(Ordering::Greater) => *value <= self.start && *value >= self.end,
            None => false,
        }
    }

    pub fn into_inner(self) -> (I, I) {
        (self.start, self.end)
    }

    pub fn into_range(self) -> RangeInclusive<I> {
        self.start..=self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpOrDown {
    Up,
    Down,
    Done,
}
use UpOrDown::*;

impl<I> IntoIterator for ReversibleRange<I>
where
    I: Clone + Integer,
{
    type Item = I;

    type IntoIter = ReversibleRangeIter<I>;

    fn into_iter(self) -> Self::IntoIter {
        let dir = match self.start.partial_cmp(&self.end) {
            Some(Ordering::Less) => Up,
            Some(Ordering::Equal) => Done,
            Some(Ordering::Greater) => Down,
            None => Done,
        };
        ReversibleRangeIter { dir, range: self }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReversibleRangeIter<I> {
    dir: UpOrDown,
    range: ReversibleRange<I>,
}

impl<I> ReversibleRangeIter<I> {
    pub fn into_inner(self) -> ReversibleRange<I> {
        self.range
    }
}

impl<I> Iterator for ReversibleRangeIter<I>
where
    I: Clone + Integer,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self.dir {
            Up => {
                let ret = self.range.start.clone();
                self.range.start.inc();
                Some(ret)
            }
            Down => {
                let ret = self.range.start.clone();
                self.range.start.dec();
                Some(ret)
            }
            Done => None,
        }
    }
}
