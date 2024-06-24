use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bool {
    T,
    F,
}

impl Bool {
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Bool::T, Bool::T) => Bool::T,
            _ => Bool::F,
        }
    }
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Bool::F, Bool::F) => Bool::F,
            _ => Bool::T,
        }
    }
}

impl Not for Bool {
    type Output = Bool;
    fn not(self) -> Self::Output {
        match self {
            Bool::T => Bool::F,
            Bool::F => Bool::T,
        }
    }
}
