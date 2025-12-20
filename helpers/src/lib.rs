#![feature(array_try_from_fn, iter_from_coroutine)]

use std::io::stdin;
use std::ops::{Add, Div, Mul};
use std::str::FromStr;

pub use std::array::{from_fn as from_fn_array, try_from_fn};
pub use std::cmp::Reverse;
pub use std::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};
pub use std::convert::identity;
pub use std::fmt::{Debug, Display};
pub use std::iter::{
    empty as empty_iter, from_coroutine as coroutine_iter, from_fn as fn_iter, once as once_iter,
    once_with as once_with_iter, repeat as repeat_iter, repeat_n as repeat_n_iter,
    repeat_with as repeat_with_iter, successors,
};
pub use std::mem::{replace, swap, take};

pub use arrayvec::{self, ArrayVec};
pub use atoi::{self, ascii_to_digit, atoi as parse_ascii};
pub use binary_heap_plus::{self, BinaryHeap};
pub use bstr::{self, BStr, BString, ByteSlice, ByteVec};
pub use itertools::Itertools;
pub use num_integer::*;
pub use petgraph;
pub use primal::*;
pub use rand::{self, Rng};
pub use regex::bytes::Regex;
pub use rustc_hash::{FxBuildHasher, FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

mod unwrap_display;
pub use unwrap_display::*;

mod rand_ext;
pub use rand_ext::*;

mod display_ext;
pub use display_ext::*;

mod consume;
pub use consume::*;

mod counter;
pub use counter::*;

mod better_iter;
pub use better_iter::*;

mod direction;
pub use direction::*;

mod output;
pub use output::*;

/// Short version of [`Default::default`].
pub fn def<D: Default>() -> D {
    D::default()
}

/// Computes the triangular number.
///
/// # Example
/// ```
/// # use helpers::triangular_number;
/// for (n, ans) in [0, 1, 3, 6, 10, 15, 21, 28, 36, 45, 55].into_iter().enumerate() {
///     assert_eq!(triangular_number(n), ans);
/// }
/// ```
pub fn triangular_number<N>(n: N) -> N
where
    N: Add<Output = N> + Mul<Output = N> + Div<Output = N> + TryFrom<u8> + Copy,
    N::Error: Debug,
{
    n * (n + 1u8.try_into().unwrap()) / 2u8.try_into().unwrap()
}

/// Reads a value from standard input.
///
/// Panics if reading from stdin fails. Returns an error if parsing the
/// resulting string fails.
pub fn read_value<T>() -> Result<T, T::Err>
where
    T: FromStr,
{
    stdin().lines().next().unwrap().unwrap().trim().parse()
}

/// Waits for a newline from stdin.
pub fn pause() {
    let line = stdin().lines().next().unwrap().unwrap();
    if line.trim() == "q" {
        std::process::exit(0)
    }
}

/// Creates a [`HashSet`] from a list of values.
///
/// # Examples
///
/// ```
/// # use helpers::{hashset, HashSet};
/// let set = hashset! { 0, 1, 2 };
/// assert_eq!(set, HashSet::from_iter([0, 1, 2]));
/// ```
#[macro_export]
macro_rules! hashset {
	($($i:expr),* $(,)?) => {
		HashSet::from_iter([$($i),*])
	};
}

/// Creates a [`HashMap`] from a list of values.
///
/// # Examples
///
/// ```
/// # use helpers::{hashmap, HashMap};
/// let map = hashmap! {
///     0 => "a",
///     1 => "b",
///     2 => "c",
/// };
/// assert_eq!(map, HashMap::from_iter([
///     (0, "a"),
///     (1, "b"),
///     (2, "c"),
/// ]));
/// ```
#[macro_export]
macro_rules! hashmap {
	($($k:expr => $v:expr),* $(,)?) => {
		HashMap::from_iter([$(($k, $v)),*])
	};
}

/// Creates a [`BTreeSet`] from a list of values.
///
/// # Examples
///
/// ```
/// # use helpers::{btreeset, BTreeSet};
/// let set = btreeset! { 0, 1, 2 };
/// assert_eq!(set, BTreeSet::from_iter([0, 1, 2]));
/// ```
#[macro_export]
macro_rules! btreeset {
	($($i:expr),* $(,)?) => {
		BTreeSet::from_iter([$($i),*])
	};
}

/// Creates a [`BTreeMap`] from a list of values.
///
/// # Examples
///
/// ```
/// # use helpers::{btreemap, BTreeMap};
/// let map = btreemap! {
///     0 => "a",
///     1 => "b",
///     2 => "c",
/// };
/// assert_eq!(map, BTreeMap::from_iter([
///     (0, "a"),
///     (1, "b"),
///     (2, "c"),
/// ]));
/// ```
#[macro_export]
macro_rules! btreemap {
	($($k:expr => $v:expr),* $(,)?) => {
		BTreeMap::from_iter([$(($k, $v)),*])
	};
}
