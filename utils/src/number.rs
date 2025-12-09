use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use serde::Serialize;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct Number(pub usize);

impl Number {
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
    pub fn succ(self) -> Self {
        Number(self.0 + 1)
    }
    pub fn pred(self) -> Self {
        if self.is_zero() {
            Number(0)
        } else {
            Number(self.0 - 1)
        }
    }
}

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number(value)
    }
}

impl From<Number> for usize {
    fn from(value: Number) -> Self {
        value.0
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Number {
    type Output = Number;
    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0 + rhs.0)
    }
}

impl Add<usize> for Number {
    type Output = Number;
    fn add(self, rhs: usize) -> Self::Output {
        Number(self.0 + rhs)
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl AddAssign<usize> for Number {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Sub for Number {
    type Output = Number;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.0 > rhs.0 {
            Number(self.0 - rhs.0)
        } else {
            Number(0)
        }
    }
}

impl Sub<usize> for Number {
    type Output = Number;
    fn sub(self, rhs: usize) -> Self::Output {
        if self.0 > rhs {
            Number(self.0 - rhs)
        } else {
            Number(0)
        }
    }
}

impl SubAssign for Number {
    fn sub_assign(&mut self, rhs: Self) {
        if self.0 > rhs.0 {
            self.0 -= rhs.0;
        } else {
            self.0 = 0;
        }
    }
}

impl SubAssign<usize> for Number {
    fn sub_assign(&mut self, rhs: usize) {
        if self.0 > rhs {
            self.0 -= rhs;
        } else {
            self.0 = 0;
        }
    }
}
