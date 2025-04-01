use std::fmt::Display;
use utils::number::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
    parameter_length: usize,
    projection_num: usize,
}

impl Projection {
    pub fn parameter_length(&self) -> usize {
        self.parameter_length
    }
    pub fn projection_num(&self) -> usize {
        self.projection_num
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Composition {
    pub parameter_length: usize,
    pub outer_func: Box<RecursiveFunctions>,
    pub inner_func: Box<Vec<RecursiveFunctions>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveRecursion {
    pub zero_func: Box<RecursiveFunctions>,
    pub succ_func: Box<RecursiveFunctions>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MuOperator {
    pub mu_func: Box<RecursiveFunctions>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecursiveFunctions {
    ZeroConstant,
    Successor,
    Projection(Projection),
    Composition(Composition),
    PrimitiveRecursion(PrimitiveRecursion),
    MuOperator(MuOperator),
}

impl RecursiveFunctions {
    pub fn parameter_length(&self) -> usize {
        match self {
            RecursiveFunctions::ZeroConstant => 0,
            RecursiveFunctions::Successor => 1,
            RecursiveFunctions::Projection(ref proj) => proj.parameter_length,
            RecursiveFunctions::Composition(ref comp) => comp.parameter_length,
            RecursiveFunctions::PrimitiveRecursion(ref prim) => {
                &prim.zero_func.parameter_length() + 1
            }
            RecursiveFunctions::MuOperator(ref muop) => &muop.mu_func.parameter_length() - 1,
        }
    }
    pub fn zero() -> RecursiveFunctions {
        Self::ZeroConstant
    }
    pub fn succ() -> RecursiveFunctions {
        Self::Successor
    }
    pub fn projection(len: usize, num: usize) -> Result<RecursiveFunctions, String> {
        if len <= num {
            Err("projection number if out of range".to_string())
        } else {
            Ok(Self::Projection(Projection {
                parameter_length: len,
                projection_num: num,
            }))
        }
    }
    pub fn composition(
        parameter_length: usize,
        inner_funcs: Vec<RecursiveFunctions>,
        outer_func: RecursiveFunctions,
    ) -> Result<RecursiveFunctions, String> {
        if inner_funcs.len() != outer_func.parameter_length() {
            return Err(
                "length of inner_funcs is different from outer_func's parameter length".to_string(),
            );
        }
        if inner_funcs
            .iter()
            .map(|func| func.parameter_length())
            .any(|len| len != parameter_length)
        {
            return Err("each element of the array has a different length".to_string());
        }
        Ok(Self::Composition(Composition {
            parameter_length,
            outer_func: Box::new(outer_func),
            inner_func: Box::new(inner_funcs),
        }))
    }
    pub fn primitive_recursion(
        zero_func: RecursiveFunctions,
        succ_func: RecursiveFunctions,
    ) -> Result<RecursiveFunctions, String> {
        if zero_func.parameter_length() + 2 != succ_func.parameter_length() {
            Err("par len of prim is invalid".to_string())
        } else {
            Ok(Self::PrimitiveRecursion(PrimitiveRecursion {
                zero_func: Box::new(zero_func),
                succ_func: Box::new(succ_func),
            }))
        }
    }
    pub fn muoperator(func: RecursiveFunctions) -> Result<RecursiveFunctions, String> {
        if func.parameter_length() == 0 {
            Err("par len of muop is invalid".to_string())
        } else {
            Ok(Self::MuOperator(MuOperator {
                mu_func: Box::new(func),
            }))
        }
    }
}

impl Display for RecursiveFunctions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RecursiveFunctions::ZeroConstant => "zero-const".to_string(),
            RecursiveFunctions::Successor => "successor".to_string(),
            RecursiveFunctions::Projection(Projection {
                parameter_length,
                projection_num,
            }) => {
                format!("proj {parameter_length} {projection_num}")
            }
            RecursiveFunctions::Composition(Composition {
                parameter_length: _,
                outer_func,
                inner_func,
            }) => {
                let inner: String = inner_func
                    .iter()
                    .map(|func| format!("{{{func}}}"))
                    .reduce(|str1, str2| str1 + &str2)
                    .unwrap_or("no function".to_string());
                format!("composition {{{outer_func}}} {}", inner)
            }
            RecursiveFunctions::PrimitiveRecursion(PrimitiveRecursion {
                zero_func,
                succ_func,
            }) => {
                format!("primitive recursion {} {}", zero_func, succ_func)
            }
            RecursiveFunctions::MuOperator(MuOperator { mu_func }) => {
                format!("mu operator {mu_func}")
            }
        };
        write!(f, "{str}")
    }
}

pub struct NaturalFunction {
    parameter_length: usize,
    func: Box<dyn Fn(NumberTuple) -> Number>,
}

impl NaturalFunction {
    pub fn param(&self) -> usize {
        self.parameter_length
    }
    pub fn unchecked_subst(&self, num: NumberTuple) -> Number {
        (self.func)(num)
    }
    pub fn checked_subst(&self, num: NumberTuple) -> Option<Number> {
        if num.len() != self.parameter_length {
            None
        } else {
            Some(self.unchecked_subst(num))
        }
    }
}

pub fn interpreter(func: &RecursiveFunctions) -> NaturalFunction {
    // eprintln!("{func:?}");
    match func {
        RecursiveFunctions::ZeroConstant => NaturalFunction {
            parameter_length: 0,
            func: Box::new(|_| Number(0)),
        },
        RecursiveFunctions::Successor => NaturalFunction {
            parameter_length: 1,
            func: Box::new(|vec| {
                let (f, _) = vec.split().unwrap();
                f.succ()
            }),
        },
        RecursiveFunctions::Projection(proj) => {
            let num = proj.projection_num;
            NaturalFunction {
                parameter_length: func.parameter_length(),
                func: Box::new(move |tuple| tuple.index(num).unwrap().clone()),
            }
        }
        RecursiveFunctions::Composition(composition) => {
            let Composition {
                parameter_length,
                outer_func,
                ref inner_func,
            } = composition;
            let outer_func = interpreter(outer_func);
            let inner_funcs = inner_func.iter().map(interpreter).collect::<Vec<_>>();
            let func: Box<dyn Fn(NumberTuple) -> Number> = Box::new(move |tuple| {
                let result_vec: NumberTuple = inner_funcs
                    .iter()
                    .map(|func| func.unchecked_subst(tuple.clone()))
                    .collect::<Vec<Number>>()
                    .into();
                outer_func.unchecked_subst(result_vec)
            });
            NaturalFunction {
                parameter_length: *parameter_length,
                func,
            }
        }
        RecursiveFunctions::PrimitiveRecursion(prim) => {
            let PrimitiveRecursion {
                zero_func,
                succ_func,
            } = prim.clone();
            let length = &zero_func.parameter_length() + 1;
            let zero_func = interpreter(&zero_func);
            let succ_func = interpreter(&succ_func);
            let this_func_clone = func.clone();
            let function: Box<dyn Fn(NumberTuple) -> Number> = Box::new(move |vector| {
                let (first, cont) = vector.clone().split().unwrap();
                if first.is_zero() {
                    zero_func.unchecked_subst(cont)
                } else {
                    let pred_result = {
                        let pred_input = concat_head(first.clone().pred(), cont.clone());
                        interpreter(&this_func_clone).unchecked_subst(pred_input)
                    };
                    let input = concat_head(pred_result, concat_head(first.pred(), cont));
                    succ_func.unchecked_subst(input)
                }
            });
            NaturalFunction {
                parameter_length: length,
                func: function,
            }
        }
        RecursiveFunctions::MuOperator(muop) => {
            let MuOperator { mu_func } = muop;
            let length = mu_func.parameter_length() - 1;
            let mu_func = interpreter(mu_func);
            let func: Box<dyn Fn(NumberTuple) -> Number> = Box::new(move |vector| {
                let mut i = 0;
                'lp: loop {
                    let result = mu_func.unchecked_subst(concat_head(Number(i), vector.clone()));
                    if result == Number(0) {
                        break 'lp Number(i);
                    }
                    i += 1;
                }
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
        let zero_func = interpreter(&zero);
        let result = zero_func.checked_subst(Vec::<usize>::new().into());
        assert_eq!(result, Some(Number(0)));
        let result = zero_func.checked_subst(vec![0].into());
        assert_eq!(result, None);
    }
    #[test]
    fn succ_call() {
        let succ = RecursiveFunctions::succ();
        let succ_func = interpreter(&succ);
        for i in 0..5 {
            let result = succ_func.checked_subst(vec![i].into());
            assert_eq!(result, Some(Number(i + 1)))
        }
    }
    #[test]
    fn proj_call() {
        let proj = RecursiveFunctions::projection(1, 0).unwrap();
        let proj_func = interpreter(&proj);
        let result = proj_func.checked_subst(vec![0].into());
        assert_eq!(result, Some(Number(0)));
        let result = proj_func.checked_subst(vec![0, 1].into());
        assert_eq!(result, None);

        let proj = RecursiveFunctions::projection(3, 0).unwrap();
        let proj_func = interpreter(&proj);
        let result = proj_func.checked_subst(vec![0, 1, 2].into());
        assert_eq!(result, Some(Number(0)));
    }
    #[test]
    fn comp_call() {
        let succcc = RecursiveFunctions::composition(
            1,
            vec![RecursiveFunctions::succ()],
            RecursiveFunctions::succ(),
        )
        .unwrap();
        let succcc_func = interpreter(&succcc);
        let result = succcc_func.checked_subst(vec![0].into());
        assert_eq!(result, Some(Number(2)));
        assert!(RecursiveFunctions::composition(0, vec![], RecursiveFunctions::succ()).is_err());
        assert!(RecursiveFunctions::composition(
            0,
            vec![RecursiveFunctions::succ()],
            RecursiveFunctions::zero()
        )
        .is_err());
        assert!(RecursiveFunctions::composition(
            1,
            vec![RecursiveFunctions::succ(), RecursiveFunctions::zero()],
            RecursiveFunctions::projection(2, 1).unwrap()
        )
        .is_err());
        let snd_succ = RecursiveFunctions::composition(
            1,
            vec![
                RecursiveFunctions::succ(),
                RecursiveFunctions::succ(),
                RecursiveFunctions::succ(),
            ],
            RecursiveFunctions::projection(3, 1).unwrap(),
        )
        .unwrap();
        let func = interpreter(&snd_succ);
        assert_eq!(func.checked_subst(vec![0].into()), Some(Number(1)));

        let snd_succ = RecursiveFunctions::composition(
            3,
            vec![
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 1).unwrap(),
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 0).unwrap(),
            ],
            RecursiveFunctions::projection(4, 1).unwrap(),
        )
        .unwrap();
        let func = interpreter(&snd_succ);
        assert_eq!(func.checked_subst(vec![0, 1, 2].into()), Some(Number(1)))
    }
    #[test]
    fn prim_call() {
        let zero_func = RecursiveFunctions::projection(1, 0).unwrap();
        let succ_func = RecursiveFunctions::composition(
            3,
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
            RecursiveFunctions::succ(),
        )
        .unwrap();
        let add = RecursiveFunctions::primitive_recursion(zero_func, succ_func).unwrap();
        let add_func = interpreter(&add);
        assert_eq!(add_func.checked_subst(vec![0, 0].into()), Some(Number(0)));
        assert_eq!(add_func.checked_subst(vec![0, 1].into()), Some(Number(1)));
        assert_eq!(add_func.checked_subst(vec![1, 0].into()), Some(Number(1)));
        assert_eq!(add_func.checked_subst(vec![1, 1].into()), Some(Number(2)));
        assert_eq!(add_func.checked_subst(vec![2, 2].into()), Some(Number(4)));
        assert_eq!(add_func.checked_subst(vec![2, 3].into()), Some(Number(5)));
    }
    fn pred_func() -> RecursiveFunctions {
        RecursiveFunctions::primitive_recursion(
            RecursiveFunctions::zero(),
            RecursiveFunctions::projection(2, 1).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn pred_well() {
        let pred_func = interpreter(&pred_func());
        assert_eq!(pred_func.checked_subst(vec![0].into()), Some(Number(0)));
        assert_eq!(pred_func.checked_subst(vec![1].into()), Some(Number(0)));
        assert_eq!(pred_func.checked_subst(vec![2].into()), Some(Number(1)));
        assert_eq!(pred_func.checked_subst(vec![3].into()), Some(Number(2)));
    }
    fn inv_monus() -> RecursiveFunctions {
        RecursiveFunctions::primitive_recursion(
            RecursiveFunctions::projection(1, 0).unwrap(),
            RecursiveFunctions::composition(
                3,
                vec![RecursiveFunctions::projection(3, 0).unwrap()],
                pred_func(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn monus() -> RecursiveFunctions {
        RecursiveFunctions::composition(
            2,
            vec![
                RecursiveFunctions::projection(2, 1).unwrap(),
                RecursiveFunctions::projection(2, 0).unwrap(),
            ],
            inv_monus(),
        )
        .unwrap()
    }
    #[test]
    fn monus_call() {
        let monus = interpreter(&monus());
        assert_eq!(monus.checked_subst(vec![0, 0].into()), Some(Number(0)));
        assert_eq!(monus.checked_subst(vec![0, 1].into()), Some(Number(0)));
        assert_eq!(monus.checked_subst(vec![0, 2].into()), Some(Number(0)));
        assert_eq!(monus.checked_subst(vec![1, 0].into()), Some(Number(1)));
        assert_eq!(monus.checked_subst(vec![2, 0].into()), Some(Number(2)));
        assert_eq!(monus.checked_subst(vec![1, 1].into()), Some(Number(0)));
        assert_eq!(monus.checked_subst(vec![2, 2].into()), Some(Number(0)));
        assert_eq!(monus.checked_subst(vec![2, 1].into()), Some(Number(1)));
    }
    fn id_from_inv_monus() -> RecursiveFunctions {
        RecursiveFunctions::muoperator(inv_monus()).unwrap()
    }
    #[test]
    fn muop_call() {
        let id = interpreter(&id_from_inv_monus());
        assert_eq!(id.checked_subst(vec![0].into()), Some(Number(0)));
        assert_eq!(id.checked_subst(vec![1].into()), Some(Number(1)));
        assert_eq!(id.checked_subst(vec![2].into()), Some(Number(2)));
        assert_eq!(id.checked_subst(vec![3].into()), Some(Number(3)));
    }
}
