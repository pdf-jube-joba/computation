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

#[derive(Clone)]
pub struct Projection {
    parameter_length: usize,
    projection_num: usize,
}

#[derive(Clone)]
pub struct Composition {
    outer_func: Box<RecursiveFunctions>,
    inner_func: Box<Vec<RecursiveFunctions>>,
}

#[derive(Clone)]
pub struct MuOperator {
    func: Box<RecursiveFunctions>,
}

#[derive(Clone)]
pub enum RecursiveFunctions {
    ZeroConstant,
    Successor,
    Projection(Projection),
    Composition(Composition),
    MuOperator(MuOperator),
}

impl RecursiveFunctions {
    fn parameter_length(&self) -> usize {
        match self {
            RecursiveFunctions::ZeroConstant => 0,
            RecursiveFunctions::Successor => 1,
            RecursiveFunctions::Projection(ref proj) => proj.parameter_length,
            _ => unimplemented!(),
        }
    }
}

pub struct NaturalFunction {
    parameter_length: usize,
    func: Box<dyn Fn(Vec<Number>) -> Number>,
}

impl NaturalFunction {
    pub fn param(&self) -> usize {
        self.parameter_length
    }
    pub fn unchecked_subst(&self, num: Vec<Number>) -> Number {
        (&self.func)(num)
    }
    pub fn checked_subst(&self, num: Vec<Number>) -> Result<Number, ()> {
        if num.len() != self.parameter_length {
            Err(())
        } else {
            Ok(self.unchecked_subst(num))
        }
    }
}

pub fn interpreter(func: RecursiveFunctions) -> NaturalFunction {
    match func {
        RecursiveFunctions::ZeroConstant => NaturalFunction {
            parameter_length: 0,
            func: Box::new(|_| Number(0)),
        },
        RecursiveFunctions::Successor => NaturalFunction {
            parameter_length: 1,
            func: Box::new(|vec| vec[0].clone().succ()),
        },
        RecursiveFunctions::Projection(proj) => {
            let Projection {
                parameter_length,
                projection_num,
            } = proj;
            NaturalFunction {
                parameter_length,
                func: Box::new(move |vec: Vec<Number>| vec[projection_num].clone()),
            }
        }
        RecursiveFunctions::Composition(composition) => {
            let Composition {
                outer_func,
                inner_func,
            } = composition;
            let length = (&outer_func).parameter_length();
            let func: Box<dyn Fn(Vec<Number>) -> Number> = Box::new(move |vector: Vec<Number>| {
                let result_vec: Vec<Number> = inner_func
                    .to_owned()
                    .into_iter()
                    .map(|function| {
                        let result: Number = interpreter(function).unchecked_subst(vector.clone());
                        result
                    })
                    .collect();
                interpreter(*outer_func.to_owned()).unchecked_subst(result_vec)
            });
            NaturalFunction {
                parameter_length: length,
                func,
            }
        }
        RecursiveFunctions::MuOperator(muop) => {
            let MuOperator { func: function } = muop;
            let length = function.parameter_length() - 1;
            let func: Box<dyn Fn(Vec<Number>) -> Number> = Box::new(move |vector| {
                let mut i = 0;
                let result = 'lp: loop {
                    let mut input = vec![Number(i)];
                    input.extend_from_slice(&vector);
                    let result = interpreter(*function.to_owned()).unchecked_subst(input);
                    if result == Number(0) {
                        break 'lp Number(i);
                    }
                    i = i + 1;
                };
                result
            });
            NaturalFunction {
                parameter_length: length,
                func,
            }
        }
    }
}
