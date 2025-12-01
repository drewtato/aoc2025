#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
use std::ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign};

use Direction::*;

type Int = i32;

impl Direction {
    pub fn to_coord(self) -> [Int; 2] {
        match self {
            North => [-1, 0],
            East => [0, 1],
            South => [1, 0],
            West => [0, -1],
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            North => 0,
            East => 1,
            South => 2,
            West => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => North,
            1 => East,
            2 => South,
            3 => West,
            _ => panic!("directions are only 0 to 3"),
        }
    }

    pub fn turn_right(&mut self) {
        *self = match self {
            North => East,
            East => South,
            South => West,
            West => North,
        };
    }

    pub fn turn_left(&mut self) {
        *self = match self {
            North => West,
            East => North,
            South => East,
            West => South,
        }
    }

    pub fn right(mut self) -> Self {
        self.turn_right();
        self
    }

    pub fn left(mut self) -> Self {
        self.turn_left();
        self
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, North | South)
    }

    pub fn distance_to_coord(self, pos: [Int; 2], coord: Int) -> Int {
        match self {
            North => pos[0] - coord,
            East => coord - pos[1],
            South => coord - pos[0],
            West => pos[1] - coord,
        }
    }

    pub fn all() -> [Self; 4] {
        [North, East, South, West]
    }
}

impl<T> Index<Direction> for [T; 4] {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        &self[index.to_index()]
    }
}

impl<T> IndexMut<Direction> for [T; 4] {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self[index.to_index()]
    }
}

impl Add<Direction> for [Int; 2] {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        let [y, x] = self;
        let [dy, dx] = rhs.to_coord();
        [y + dy, x + dx]
    }
}

impl AddAssign<Direction> for [Int; 2] {
    fn add_assign(&mut self, rhs: Direction) {
        *self = *self + rhs
    }
}

impl Sub<Direction> for [Int; 2] {
    type Output = Self;

    fn sub(self, rhs: Direction) -> Self::Output {
        let [y, x] = self;
        let [dy, dx] = rhs.to_coord();
        [y - dy, x - dx]
    }
}

impl SubAssign<Direction> for [Int; 2] {
    fn sub_assign(&mut self, rhs: Direction) {
        *self = *self - rhs
    }
}
