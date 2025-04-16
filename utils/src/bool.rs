use std::{
    fmt::Display,
    ops::{Neg, Not},
    str::FromStr,
};

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


// !T = F
impl Not for Bool {
    type Output = Bool;
    fn not(self) -> Self::Output {
        match self {
            Bool::T => Bool::F,
            Bool::F => Bool::T,
        }
    }
}

impl FromStr for Bool {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "T" => Ok(Bool::T),
            "F" => Ok(Bool::F),
            _ => Err(anyhow::anyhow!("fail to parse {s}")),
        }
    }
}

impl Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bool::T => write!(f, "T"),
            Bool::F => write!(f, "F"),
        }
    }
}
