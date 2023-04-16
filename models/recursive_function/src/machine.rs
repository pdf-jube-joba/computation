#[derive(Debug, Clone, PartialEq)]
pub struct Number(usize);

impl Number {
    fn is_zero(self) -> bool {
        self.0 == 0
    }
    fn succ(self) -> Self {
        Number(self.0 + 1)
    }
}

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTuple(Vec<Number>);

impl TryFrom<String> for NumberTuple {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if !(value.starts_with('(') && value.ends_with(')')) {
            return Err("not tuple".to_string());
        }
        let vec = value.get(1..value.len()-1).unwrap()
            .split(',')
            .map(|str| {
                match str.trim().parse() {
                    Ok(n) => Ok(Number(n)),
                    Err(_) => Err("parse fail".to_string())
                }
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