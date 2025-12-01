use std::fmt::{Debug, Display};

pub struct Output<T>(pub Vec<T>);

impl<T> Display for Output<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(o) = iter.next() {
            write!(f, "{}", o)?;
        }
        for o in iter {
            write!(f, ",{}", o)?;
        }
        Ok(())
    }
}

impl<T> Debug for Output<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(o) = iter.next() {
            write!(f, "{:?}", o)?;
        }
        for o in iter {
            write!(f, ",{:?}", o)?;
        }
        Ok(())
    }
}

impl<T> FromIterator<T> for Output<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let v = iter.into_iter().collect();
        Self(v)
    }
}
