use std::{
    fmt::Display,
    ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign},
    str::FromStr,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTuple(Vec<Number>);

impl NumberTuple {
    pub fn unit() -> Self {
        NumberTuple(vec![])
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn split(self) -> Option<(Number, NumberTuple)> {
        if self.is_empty() {
            None
        } else {
            Some((self.0[0].clone(), NumberTuple(self.0[1..].to_owned())))
        }
    }
    pub fn index(&self, index: usize) -> Option<&Number> {
        if self.len() <= index {
            None
        } else {
            Some(&self.0[index])
        }
    }
    pub fn concat(self, tuple: NumberTuple) -> NumberTuple {
        let mut vec = self.0;
        vec.extend(tuple.0);
        NumberTuple(vec)
    }
}

impl Number {
    pub fn concat(self, tuple: NumberTuple) -> NumberTuple {
        let mut vec = vec![self];
        vec.extend(tuple.0);
        NumberTuple(vec)
    }
}

impl From<Vec<usize>> for NumberTuple {
    fn from(value: Vec<usize>) -> Self {
        NumberTuple(value.into_iter().map(Number::from).collect())
    }
}

impl From<Vec<Number>> for NumberTuple {
    fn from(value: Vec<Number>) -> Self {
        NumberTuple(value)
    }
}

impl From<Number> for NumberTuple {
    fn from(value: Number) -> Self {
        NumberTuple(vec![value])
    }
}

impl TryFrom<String> for NumberTuple {
    type Error = anyhow::Error; // Changed from String to anyhow::Error
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if !(value.starts_with('(') && value.ends_with(')')) {
            return Err(anyhow::Error::msg("not tuple"));
        }
        let vec = value
            .get(1..value.len() - 1)
            .unwrap()
            .split(',')
            .map(|str| match str.trim().parse() {
                Ok(n) => Ok(Number(n)),
                Err(_) => Err(anyhow::Error::msg("parse fail")),
            })
            .collect::<Result<_, _>>()?;
        Ok(NumberTuple(vec))
    }
}

impl FromStr for NumberTuple {
    type Err = anyhow::Error; // Changed from String to anyhow::Error
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if !(value.starts_with('(') && value.ends_with(')')) {
            return Err(anyhow::Error::msg("not tuple"));
        }
        let vec = value
            .get(1..value.len() - 1)
            .unwrap()
            .split(',')
            .map(|str| match str.trim().parse() {
                Ok(n) => Ok(Number(n)),
                Err(_) => Err(anyhow::Error::msg("parse fail")),
            })
            .collect::<Result<_, _>>()?;
        Ok(NumberTuple(vec))
    }
}

impl From<NumberTuple> for String {
    fn from(value: NumberTuple) -> Self {
        let mut s = String::new();
        s.push('(');
        for (i, Number(num)) in value.0.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str(&num.to_string());
        }
        s.push(')');
        s
    }
}

impl From<NumberTuple> for Vec<Number> {
    fn from(value: NumberTuple) -> Self {
        value.0
    }
}

impl From<NumberTuple> for Vec<usize> {
    fn from(value: NumberTuple) -> Self {
        value.0.into_iter().map(|num| num.into()).collect()
    }
}

impl Index<Number> for NumberTuple {
    type Output = Number;
    fn index(&self, index: Number) -> &Self::Output {
        &self.0[index.0]
    }
}

impl Index<usize> for NumberTuple {
    type Output = Number;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<Number> for NumberTuple {
    fn index_mut(&mut self, index: Number) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

impl IndexMut<usize> for NumberTuple {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Display for NumberTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push('(');
        for (i, Number(num)) in self.0.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str(&num.to_string());
        }
        s.push(')');
        write!(f, "{}", s)
    }
}
