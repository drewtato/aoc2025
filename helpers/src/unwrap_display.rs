use std::fmt::Display;

pub trait UnwrapDisplay {
    type Output;
    /// Like [`Result::unwrap`], but prints with `Display` instead of
    /// `Debug`.
    ///
    /// # Examples
    ///
    /// Unwrapping an `Ok` works normally.
    ///
    /// ```
    /// # use helpers::UnwrapDisplay;
    /// let opt: Result<i32, &'static str> = Ok(4);
    /// assert_eq!(4, opt.unwrap_display());
    /// ```
    ///
    /// Unwrapping an `Err` prints using `Display` and panics.
    ///
    /// ```should_panic
    /// # use helpers::UnwrapDisplay;
    /// let opt: Result<i32, &'static str> = Err("oh no");
    /// opt.unwrap_display(); // panics
    /// ```
    fn unwrap_display(self) -> Self::Output;
}

impl<T, E> UnwrapDisplay for Result<T, E>
where
    E: Display,
{
    type Output = T;

    fn unwrap_display(self) -> Self::Output {
        self.unwrap_or_else(|e| panic!("{e}"))
    }
}
