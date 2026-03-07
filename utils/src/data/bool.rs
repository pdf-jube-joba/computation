use std::ops::{BitAnd, BitOr, Not};

use crate::TextCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bool {
    T,
    F,
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

impl BitAnd for Bool {
    type Output = Bool;
    fn bitand(self, other: Self) -> Self::Output {
        match (self, other) {
            (Bool::T, Bool::T) => Bool::T,
            _ => Bool::F,
        }
    }
}

impl BitOr for Bool {
    type Output = Bool;
    fn bitor(self, other: Self) -> Self::Output {
        match (self, other) {
            (Bool::F, Bool::F) => Bool::F,
            _ => Bool::T,
        }
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> Self {
        match b {
            Bool::T => true,
            Bool::F => false,
        }
    }
}

impl TextCodec for Bool {
    fn parse(text: &str) -> Result<Self, String> {
        match text.trim() {
            "T" | "true" | "1" => Ok(Bool::T),
            "F" | "false" | "0" => Ok(Bool::F),
            other => Err(format!("Invalid boolean text: {}", other)),
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Bool::T => write!(f, "T"),
            Bool::F => write!(f, "F"),
        }
    }
}
