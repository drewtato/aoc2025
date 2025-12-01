use std::fmt::{Debug, Display, Write};

pub trait StringExt {
    /// Formats a single value onto the end of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use helpers::StringExt;
    /// let mut s = "hi ".to_string();
    ///
    /// s.write(5);
    /// s += " ";
    ///
    /// s.write(false);
    ///
    /// assert_eq!(s, "hi 5 false");
    /// ```
    fn write<T: Display>(&mut self, t: T);

    /// Debug formats a single value onto the end of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use helpers::StringExt;
    /// # use std::time::Duration;
    /// let mut s = "hi ".to_string();
    ///
    /// s.write_dbg(Duration::from_secs(5));
    /// s += " ";
    ///
    /// s.write_dbg("hello");
    ///
    /// assert_eq!(s, "hi 5s \"hello\"");
    /// ```
    fn write_dbg<T: Debug>(&mut self, t: T);
}

impl StringExt for String {
    fn write<T: Display>(&mut self, t: T) {
        write!(self, "{t}").unwrap();
    }

    fn write_dbg<T: Debug>(&mut self, t: T) {
        write!(self, "{t:?}").unwrap();
    }
}
