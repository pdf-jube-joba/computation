#[derive(Debug, Clone, PartialEq)]
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

impl Into<usize> for Number {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTuple(Vec<Number>);

impl NumberTuple {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn split(self) -> Result<(Number, NumberTuple), ()> {
        if self.0.len() == 0 {
            Err(())
        } else {
            Ok((self.0[0].clone(), NumberTuple(self.0[1..].to_owned())))
        }
    }
    pub fn index(&self, index: usize) -> Result<&Number, ()> {
        if self.len() <= index {
            Err(())
        } else {
            Ok(&self.0[index])
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

impl Into<String> for NumberTuple {
    fn into(self) -> String {
        let mut s = String::new();
        s.push('(');
        for (i, Number(num)) in self.0.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            s.push_str(&num.to_string());
        }
        s.push(')');
        s
    }
}

impl Into<Vec<Number>> for NumberTuple {
    fn into(self) -> Vec<Number> {
        self.0
    }
}

impl Into<Vec<usize>> for NumberTuple {
    fn into(self) -> Vec<usize> {
        self.0.into_iter().map(|num| num.into()).collect()
    }
}
