use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Number(pub usize);

impl Number {
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
    pub fn succ(self) -> Self {
        Number(self.0 + 1)
    }
    pub fn pred(self) -> Self {
        Number(self.0 - 1)
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

impl Add for Number {
    type Output = Number;
    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0 + rhs.0)
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTuple(Vec<Number>);

impl NumberTuple {
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
}

pub fn concat_head(num: Number, NumberTuple(tuple): NumberTuple) -> NumberTuple {
    let vec = std::iter::once(num).chain(tuple).collect::<Vec<_>>();
    NumberTuple(vec)
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
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if !(value.starts_with('(') && value.ends_with(')')) {
            return Err("not tuple".to_string());
        }
        let vec = value
            .get(1..value.len() - 1)
            .unwrap()
            .split(',')
            .map(|str| match str.trim().parse() {
                Ok(n) => Ok(Number(n)),
                Err(_) => Err("parse fail".to_string()),
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
