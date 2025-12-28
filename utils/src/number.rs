use std::ops::{Add, AddAssign, Sub, SubAssign};

use serde::Serialize;

// Natural number represented in little-endian byte array
// i.e., least significant byte first
// n = \sum_{i=0}^{len-1} bytes[i] * 256^i
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub fn as_usize(&self) -> Result<usize, String> {
        let bytes = self.trimmed_bytes();
        if bytes.len() > size_of::<usize>() {
            return Err("Number too large to fit in usize".to_string());
        }
        const SIZE: usize = size_of::<usize>();
        let mut buf = [0u8; SIZE];
        for (i, &b) in bytes.iter().take(SIZE).enumerate() {
            buf[i] = b;
        }
        Ok(usize::from_le_bytes(buf))
    }
    pub fn as_u64(&self) -> Option<u64> {
        let bytes = self.trimmed_bytes();
        if bytes.len() > size_of::<u64>() {
            return None;
        }
        let mut buf = [0u8; size_of::<u64>()];
        for (i, &b) in bytes.iter().enumerate() {
            buf[i] = b;
        }
        Some(u64::from_le_bytes(buf))
    }
    pub fn to_decimal_string(&self) -> String {
        let mut bytes = self.trimmed_bytes();
        if bytes.is_empty() {
            return "0".to_string();
        }
        let mut digits = Vec::new();
        while !bytes.is_empty() {
            let mut carry = 0u16;
            for i in (0..bytes.len()).rev() {
                let cur = (carry << 8) + bytes[i] as u16;
                bytes[i] = (cur / 10) as u8;
                carry = cur % 10;
            }
            digits.push((carry as u8 + b'0') as char);
            while bytes.last() == Some(&0) {
                bytes.pop();
            }
        }
        digits.iter().rev().collect()
    }
    pub fn as_u8array(&self) -> &[u8] {
        &self.0
    }
    pub fn from_u8array(arr: &[u8]) -> Self {
        Number(arr.to_vec())
    }

    fn trimmed_bytes(&self) -> Vec<u8> {
        let mut bytes = self.0.clone();
        while bytes.last() == Some(&0) {
            bytes.pop();
        }
        bytes
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

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(value) = self.as_u64() {
            serializer.serialize_u64(value)
        } else {
            serializer.serialize_str(&self.to_decimal_string())
        }
    }
}
