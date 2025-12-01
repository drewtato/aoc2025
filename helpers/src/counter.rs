use std::hash::Hash;

pub type Counter<T> = CounterGen<T, usize>;

#[derive(Debug, Clone)]
pub struct CounterGen<T, C>(pub super::HashMap<T, C>);

impl<A, C> FromIterator<A> for CounterGen<A, C>
where
    C: num_integer::Integer + Clone,
    A: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut counter = Self::default();
        for item in iter {
            counter.add(item);
        }
        counter
    }
}

impl<T, C> Default for CounterGen<T, C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, C> CounterGen<T, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> super::HashMap<T, C> {
        self.0
    }
}

impl<T, C> CounterGen<T, C>
where
    C: num_integer::Integer + Clone,
    T: Hash + Eq,
{
    pub fn add(&mut self, item: T) {
        self.0.entry(item).or_insert_with(C::zero).inc();
    }

    pub fn subtract(&mut self, item: T) {
        self.0.entry(item).or_insert_with(C::zero).dec();
    }

    pub fn subtract_saturating(&mut self, item: T) {
        if let Some(count) = self.0.get_mut(&item) {
            if count.is_zero() {
                return;
            }
            count.dec();
        }
    }

    pub fn remove_zeroes(&mut self) {
        self.0.retain(|_, c| !c.is_zero());
    }

    pub fn get(&self, item: T) -> C {
        self.0.get(&item).cloned().unwrap_or_else(C::zero)
    }

    pub fn get_mut(&mut self, item: T) -> &mut C {
        self.0.entry(item).or_insert_with(C::zero)
    }
}
