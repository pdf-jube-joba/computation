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
    parameter_length: usize,
    outer_func: Box<RecursiveFunctions>,
    inner_func: Box<Vec<RecursiveFunctions>>,
}

#[derive(Clone)]
pub struct PrimitiveRecursion {
    zero_func: Box<RecursiveFunctions>,
    succ_func: Box<RecursiveFunctions>,
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
    PrimitiveRecursion(PrimitiveRecursion),
    MuOperator(MuOperator),
}

impl RecursiveFunctions {
    fn parameter_length(&self) -> usize {
        match self {
            RecursiveFunctions::ZeroConstant => 0,
            RecursiveFunctions::Successor => 1,
            RecursiveFunctions::Projection(ref proj) => proj.parameter_length,
            RecursiveFunctions::Composition(ref comp) => comp.parameter_length,
            RecursiveFunctions::PrimitiveRecursion(ref prim) => {
                &prim.zero_func.parameter_length() + 1
            }
            RecursiveFunctions::MuOperator(ref muop) => &muop.func.parameter_length() - 1,
            _ => unimplemented!(),
        }
    }
    fn zero() -> RecursiveFunctions {
        Self::ZeroConstant
    }
    fn succ() -> RecursiveFunctions {
        Self::Successor
    }
    fn projection(len: usize, num: usize) -> Result<RecursiveFunctions, ()> {
        if len < num {
            return Err(());
        } else {
            Ok(Self::Projection(Projection {
                parameter_length: len,
                projection_num: num,
            }))
        }
    }
    fn composition(
        inner_funcs: Vec<RecursiveFunctions>,
        outer_func: RecursiveFunctions,
    ) -> Result<RecursiveFunctions, ()> {
        let share_len = (&inner_funcs[0]).parameter_length();

        if inner_funcs.len() != outer_func.parameter_length() || {
            inner_funcs.len() != 0 && {
                let share_len = (&inner_funcs[0]).parameter_length();
                (&inner_funcs)
                    .iter()
                    .map(|func| func.parameter_length())
                    .any(|len| len != share_len)
            }
        } {
            return Err(());
        } else {
            return Ok(Self::Composition(Composition {
                parameter_length: share_len,
                outer_func: Box::new(outer_func),
                inner_func: Box::new(inner_funcs),
            }));
        }
    }
    fn muoperator(func: RecursiveFunctions) -> Result<RecursiveFunctions, ()> {
        if func.parameter_length() == 0 {
            return Err(());
        } else {
            return Ok(Self::MuOperator(MuOperator {
                func: Box::new(func),
            }));
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

#[cfg(test)]
mod tests {
    use super::Number;
    use super::{interpreter, RecursiveFunctions};

    #[test]
    fn zero_call() {
        let zero = RecursiveFunctions::zero();
        let zero_func = interpreter(zero);
        let result = zero_func.checked_subst(vec![]);
        assert_eq!(result, Ok(Number(0)))
    }
    #[test]
    fn succ_call() {
        let succ = RecursiveFunctions::succ();
        let succ_func = interpreter(succ);
        for i in 0..5 {
            let result = succ_func.checked_subst(vec![Number(i)]);
            assert_eq!(result, Ok(Number(i + 1)))
        }
    }
}
