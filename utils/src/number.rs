use std::ops::{Add, AddAssign, Sub, SubAssign};

use serde::Serialize;

// Natural number represented in little-endian byte array
// i.e., least significant byte first
// n = \sum_{i=0}^{len-1} bytes[i] * 256^i
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct Number(Vec<u8>);

impl Number {
    pub fn is_zero(&self) -> bool {
        self.0.is_empty() || self.0.iter().all(|&b| b == 0)
    }
    pub fn succ(&self) -> Self {
        let one = Number(vec![1]);
        self.clone() + one
    }
    pub fn pred(&self) -> Self {
        let one = Number(vec![1]);
        self.clone() - one
    }
    pub fn as_usize(&self) -> usize {
        let mut bytes = [0u8; 8];
        for (i, &b) in self.0.iter().take(8).enumerate() {
            bytes[i] = b;
        }
        usize::from_le_bytes(bytes)
    }
    pub fn as_u8array(&self) -> &[u8] {
        &self.0
    }
}

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number(value.to_le_bytes().to_vec())
    }
}

impl From<Vec<u8>> for Number {
    fn from(value: Vec<u8>) -> Self {
        Number(value)
    }
}

impl Add for Number {
    type Output = Number;
    fn add(self, rhs: Self) -> Self::Output {
        let mut v = vec![];
        let mut carry = 0;
        for i in 0..std::cmp::max(self.0.len(), rhs.0.len()) {
            let a = if i < self.0.len() { self.0[i] } else { 0 };
            let b = if i < rhs.0.len() { rhs.0[i] } else { 0 };
            let sum = a as u16 + b as u16 + carry;
            v.push((sum & 0xFF) as u8);
            carry = sum >> 8;
        }
        Number(v)
    }
}

impl Add<usize> for Number {
    type Output = Number;
    fn add(self, rhs: usize) -> Self::Output {
        self + Number::from(rhs)
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs
    }
}

impl AddAssign<usize> for Number {
    fn add_assign(&mut self, rhs: usize) {
        *self = self.clone() + rhs
    }
}

// saturating subtraction
impl Sub for Number {
    type Output = Number;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut v = vec![];
        let mut borrow = 0;
        for i in 0..std::cmp::max(self.0.len(), rhs.0.len()) {
            let a = if i < self.0.len() { self.0[i] } else { 0 };
            let b = if i < rhs.0.len() { rhs.0[i] } else { 0 };
            let sub = (a as i16) - (b as i16) - (borrow as i16);
            if sub < 0 {
                v.push((sub + 256) as u8);
                borrow = 1;
            } else {
                v.push(sub as u8);
                borrow = 0;
            }
        }
        Number(v)
    }
}

impl Sub<usize> for Number {
    type Output = Number;
    fn sub(self, rhs: usize) -> Self::Output {
        self - Number::from(rhs)
    }
}

impl SubAssign for Number {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs
    }
}

impl SubAssign<usize> for Number {
    fn sub_assign(&mut self, rhs: usize) {
        *self = self.clone() - rhs
    }
}
