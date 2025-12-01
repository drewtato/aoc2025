use std::ops::RangeBounds;

use atoi::{FromRadix10Checked, FromRadix10SignedChecked};
use bstr::BStr;

/// A `[u8]` parser that works by consuming various items from the front of the
/// stream.
#[derive(Debug, Clone)]
pub struct Consume<'a> {
    slice: &'a BStr,
}

impl<'a> Consume<'a> {
    /// Creates a new `Consumer`.
    #[must_use]
    pub fn new(slice: &'a [u8]) -> Self {
        Self {
            slice: BStr::new(slice),
        }
    }

    /// If the first byte is `byte`, advances by one and returns `true`.
    /// Otherwise, returns `false`.
    pub fn byte(&mut self, byte: u8) -> bool {
        self.byte_with(|b| b == byte).is_some()
    }

    /// Consumes the first byte if it is within `range` and returns it.
    /// Otherwise, returns `None`.
    pub fn range(&mut self, range: impl RangeBounds<u8>) -> Option<u8> {
        self.byte_with(|b| range.contains(&b))
    }

    /// If the slice starts with `prefix`, advance the slice and returns `true`.
    /// Otherwise, returns `false`.
    pub fn prefix(&mut self, prefix: &[u8]) -> bool {
        self.with(|slice| {
            if slice.starts_with(prefix) {
                prefix.len()
            } else {
                0
            }
        })
        .len()
            == prefix.len()
    }

    /// Consumes digits from the front of the string to read an unsigned
    /// integer.
    ///
    /// If the integer overflows, this panics. If the slice did not start with a
    /// digit or sign, this returns `None`.
    pub fn int<I: FromRadix10Checked>(&mut self) -> Option<I> {
        let (Some(n), count) = I::from_radix_10_checked(self.slice) else {
            if self.len() > 30 {
                panic!(
                    "integer type not large enough to parse a number from {:?} (truncated)",
                    BStr::new(&self.slice[..20])
                );
            } else {
                panic!(
                    "integer type not large enough to parse a number from {:?}",
                    self.slice
                );
            }
        };

        if count != 0 {
            self.consume(count);
            Some(n)
        } else {
            None
        }
    }

    /// Consumes digits from the front of the string to read a signed integer.
    ///
    /// If the integer overflows, this panics. If the slice did not start with a
    /// digit or sign, this returns `None`.
    pub fn signed_int<I: FromRadix10SignedChecked>(&mut self) -> Option<I> {
        let (Some(n), count) = I::from_radix_10_signed_checked(self.slice) else {
            if self.len() > 30 {
                panic!(
                    "integer type not large enough to parse a number from {:?} (truncated)",
                    BStr::new(&self.slice[..20])
                );
            } else {
                panic!(
                    "integer type not large enough to parse a number from {:?}",
                    self.slice
                );
            }
        };

        if count != 0 {
            self.consume(count);
            Some(n)
        } else {
            None
        }
    }

    /// Consumes whitespace from the slice, returning the consumed slice.
    pub fn whitespace(&mut self) -> &'a [u8] {
        self.with(|slice| {
            slice
                .iter()
                .take_while(|&b| b.is_ascii_whitespace())
                .count()
        })
    }

    /// Consumes a newline, returning `true` if a newline was consumed.
    pub fn newline(&mut self) -> bool {
        self.byte(b'\n')
    }

    /// Consumes everything up to and including the next newline. Returns the
    /// consumed slice.
    pub fn next_newline(&mut self) -> &'a [u8] {
        let before = self.slice;
        while self.byte_with(|byte| byte != b'\n').is_some() {}
        self.consume(1);
        &before[..(before.len() - self.len())]
    }

    /// Consumes any non-digit characters, except `-` and `+`. Returns the
    /// consumed slice.
    pub fn non_digits(&mut self) -> &'a [u8] {
        let before = self.slice;
        while self
            .byte_with(|byte| !byte.is_ascii_digit() && byte != b'-' && byte != b'+')
            .is_some()
        {}
        &before[..(before.len() - self.len())]
    }

    /// Runs a closure on the entire slice.
    ///
    /// The slice will be advanced by the count returned by the closure, and the
    /// consumed slice will be returned.
    pub fn with(&mut self, f: impl FnOnce(&'a [u8]) -> usize) -> &'a [u8] {
        let before = self.slice;
        let count = f(self.slice);
        self.consume(count);
        &before[..(before.len() - self.len())]
    }

    /// Runs a closure on the first byte.
    ///
    /// If the closure returns `true`, the slice is advanced by one byte and
    /// the consumed byte is returned. Otherwise, returns `None`.
    fn byte_with(&mut self, f: impl FnOnce(u8) -> bool) -> Option<u8> {
        let &first = self.slice.first()?;
        if f(first) {
            self.consume(1);
            Some(first)
        } else {
            None
        }
    }

    /// Gets the underlying slice.
    #[must_use]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    /// Gets the underlying slice as a [`BStr`].
    #[must_use]
    pub fn bstr(&self) -> &'a BStr {
        self.slice
    }

    /// Advances the slice `count` bytes. If the slice is shorter, advances to
    /// the end.
    ///
    /// Returns the consumed portion.
    pub fn consume(&mut self, count: usize) -> &'a [u8] {
        let count = core::cmp::min(count, self.len());
        let (start, end) = self.slice.split_at(count);
        self.slice = BStr::new(end);
        start
    }

    pub fn consume_byte(&mut self) -> Option<u8> {
        let mut slice = self.slice();
        let byte = slice.split_off_first().copied();
        self.slice = BStr::new(slice);
        byte
    }

    /// Gets the length of the slice.
    #[must_use]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns `true` if the slice has a length of zero.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }
}

/// Parses all numbers in a slice into a 2D `Vec`.
pub fn parse_all_numbers<I: FromRadix10SignedChecked>(slice: &[u8]) -> Vec<Vec<I>> {
    let mut con = Consume::new(slice);
    let mut nums = Vec::new();
    while !con.is_empty() {
        let mut row = Vec::new();

        while !con.newline() && !con.is_empty() {
            con.non_digits();
            let n = con
                .signed_int()
                .unwrap_or_else(|| panic!("int not found starting at {:?}", con.consume(30)));
            row.push(n);
        }

        nums.push(row);
    }
    nums
}

#[test]
fn parse_all_numbers_t() {
    let slice = b"hello 3 world 5 7\n42897 278183 949 291371\n45yeyaeye589ashjlk283ad-90\n";
    let v = parse_all_numbers::<i32>(slice);
    let expected = vec![
        vec![3, 5, 7],
        vec![42897, 278183, 949, 291371],
        vec![45, 589, 283, -90],
    ];
    assert_eq!(v, expected);
}
