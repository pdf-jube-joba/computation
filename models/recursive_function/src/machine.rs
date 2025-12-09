use serde::Serialize;
use std::fmt::Display;
use utils::number::*;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum RecursiveFunctions {
    ZeroConstant,
    Successor,
    Projection {
        parameter_length: usize,
        projection_num: usize,
    },
    Composition {
        parameter_length: usize,
        outer_func: Box<RecursiveFunctions>,
        inner_funcs: Box<Vec<RecursiveFunctions>>,
    },
    PrimitiveRecursion {
        zero_func: Box<RecursiveFunctions>,
        succ_func: Box<RecursiveFunctions>,
    },
    MuOperator {
        mu_func: Box<RecursiveFunctions>,
    },
}

impl RecursiveFunctions {
    pub fn parameter_length(&self) -> usize {
        match self {
            RecursiveFunctions::ZeroConstant => 0,
            RecursiveFunctions::Successor => 1,
            RecursiveFunctions::Projection {
                parameter_length, ..
            } => *parameter_length,
            RecursiveFunctions::Composition {
                parameter_length, ..
            } => *parameter_length,
            RecursiveFunctions::PrimitiveRecursion { zero_func, .. } => {
                zero_func.parameter_length() + 1
            }
            RecursiveFunctions::MuOperator { mu_func } => mu_func.parameter_length() - 1,
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
            Err("projection number is out of range".to_string())
        } else {
            Ok(Self::Projection {
                parameter_length: len,
                projection_num: num,
            })
        }
    }
    pub fn composition(
        // parameter_length: usize,
        outer_func: RecursiveFunctions,
        inner_funcs: Vec<RecursiveFunctions>,
    ) -> Result<RecursiveFunctions, String> {
        if inner_funcs.len() != outer_func.parameter_length() {
            return Err(
                "length of inner_funcs is different from outer_func's parameter length".to_string(),
            );
        }
        if inner_funcs.is_empty() {
            return Err("inner_funcs is empty".to_string());
        }
        let parameter_length = inner_funcs[0].parameter_length();

        if inner_funcs
            .iter()
            .map(|func| func.parameter_length())
            .any(|len| len != parameter_length)
        {
            return Err("each element of the array has a different length".to_string());
        }
        Ok(Self::Composition {
            parameter_length,
            outer_func: Box::new(outer_func),
            inner_funcs: Box::new(inner_funcs),
        })
    }
    pub fn primitive_recursion(
        zero_func: RecursiveFunctions,
        succ_func: RecursiveFunctions,
    ) -> Result<RecursiveFunctions, String> {
        if zero_func.parameter_length() + 2 != succ_func.parameter_length() {
            Err("parameter length of primitive recursion is invalid".to_string())
        } else {
            Ok(Self::PrimitiveRecursion {
                zero_func: Box::new(zero_func),
                succ_func: Box::new(succ_func),
            })
        }
    }
    pub fn muoperator(func: RecursiveFunctions) -> Result<RecursiveFunctions, String> {
        if func.parameter_length() == 0 {
            Err("parameter length of mu operator is invalid".to_string())
        } else {
            Ok(Self::MuOperator {
                mu_func: Box::new(func),
            })
        }
    }
}

impl Display for RecursiveFunctions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RecursiveFunctions::ZeroConstant => "ZERO".to_string(),
            RecursiveFunctions::Successor => "SUCC".to_string(),
            RecursiveFunctions::Projection {
                parameter_length,
                projection_num,
            } => {
                format!("PROJ[{parameter_length},{projection_num}]")
            }
            RecursiveFunctions::Composition {
                outer_func,
                inner_funcs,
                ..
            } => {
                let inner: String = inner_funcs
                    .iter()
                    .map(|func| format!("{func}"))
                    .reduce(|str1, str2| str1 + &str2)
                    .unwrap_or("no function".to_string());
                format!("COMP[{outer_func},{}]", inner)
            }
            RecursiveFunctions::PrimitiveRecursion {
                zero_func,
                succ_func,
            } => {
                format!("PRIM[z:{} s:{}]", zero_func, succ_func)
            }
            RecursiveFunctions::MuOperator { mu_func } => {
                format!("MUOP[{mu_func}]")
            }
        };
        write!(f, "{str}")
    }
}

// this is a struct for the computation state of the recursive function
// this struct can be used to represent each step of the process
// f(g(3), h(5, 2), 1) -> f(8, h(5, 2), 1) -> f(8, 3, 1) -> 12
// Process::Comp { function: f, args: [Process::Comp { function: g, args: [3] }, Process::Comp { function: h, args: [5, 2] }, 1] }
// -> Process::Comp { function: f, args: [8, Process::Comp { function: h, args: [5, 2] }, 1] }
// -> Process::Comp { function: f, args: [8, 3, 1] }
// -> Process::Result(12)
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Process {
    Comp {
        function: RecursiveFunctions,
        args: Vec<Process>,
    },
    // start: Mu { f, 0, (x0, , .., xn), process}
    MuOpComp {
        now_index: Number,
        args: Vec<Number>,
        function: RecursiveFunctions,
        process: Box<Process>, // now computation
    },
    Result(Number),
}

fn numbers_to_processes(numbers: Vec<Number>) -> Vec<Process> {
    numbers.into_iter().map(Process::Result).collect()
}

fn split_head(numbers: &[Number]) -> Option<(Number, Vec<Number>)> {
    numbers
        .first()
        .cloned()
        .map(|head| (head, numbers.get(1..).unwrap_or(&[]).to_vec()))
}

fn prepend_number(number: Number, tail: &[Number]) -> Vec<Number> {
    let mut vec = Vec::with_capacity(1 + tail.len());
    vec.push(number);
    vec.extend(tail.iter().cloned());
    vec
}

impl Process {
    pub fn new(func: RecursiveFunctions, args: Vec<Number>) -> Result<Self, String> {
        if args.len() != func.parameter_length() {
            return Err("length of args is different from function's parameter length".to_string());
        }
        let args = numbers_to_processes(args);
        Ok(Process::Comp {
            function: func,
            args,
        })
    }
    pub fn result(&self) -> Option<Number> {
        match self {
            Process::Result(num) => Some(num.clone()),
            _ => None,
        }
    }
    pub fn eval_one_step(&self) -> Option<Self> {
        match self {
            Process::Comp { function, args } => {
                debug_assert!(function.parameter_length() == args.len());

                // if args has some arg of which can be eval, then we need to eval one step
                let mut args = args.clone();
                let mut evaluated_numbers = Vec::new();
                for arg in args.iter_mut() {
                    if let Some(result) = arg.eval_one_step() {
                        *arg = result;
                        return Some(Process::Comp {
                            function: function.clone(),
                            args: args.clone(),
                        });
                    } else {
                        evaluated_numbers.push(arg.result().unwrap());
                    }
                }
                let args_as_tuple = evaluated_numbers;

                // if all args are evaluated, then we can eval the function
                match function {
                    RecursiveFunctions::ZeroConstant => Some(Process::Result(Number(0))),
                    RecursiveFunctions::Successor => {
                        let (head, _) = split_head(&args_as_tuple).unwrap();
                        Some(Process::Result(head.succ()))
                    }
                    RecursiveFunctions::Projection {
                        parameter_length,
                        projection_num,
                    } => {
                        debug_assert!(parameter_length == &args_as_tuple.len());
                        Some(Process::Result(args_as_tuple[*projection_num].clone()))
                    }
                    RecursiveFunctions::Composition {
                        parameter_length,
                        outer_func,
                        inner_funcs,
                    } => {
                        debug_assert!(parameter_length == &args_as_tuple.len());
                        let inner_processes: Vec<Process> = inner_funcs
                            .iter()
                            .map(|func| {
                                let inner_args = numbers_to_processes(args_as_tuple.clone());
                                Process::Comp {
                                    function: func.clone(),
                                    args: inner_args,
                                }
                            })
                            .collect();
                        Some(Process::Comp {
                            function: outer_func.as_ref().clone(),
                            args: inner_processes,
                        })
                    }
                    RecursiveFunctions::PrimitiveRecursion {
                        zero_func,
                        succ_func,
                    } => {
                        let (first, cont) = split_head(&args_as_tuple).unwrap();
                        if first.is_zero() {
                            // Prim(fz, fs)(0, ..., xn) = fz(x1, .., xn)
                            Some(Process::Comp {
                                function: zero_func.as_ref().clone(),
                                args: numbers_to_processes(cont),
                            })
                        } else {
                            // Prim(fz, fs)(x0 + 1, ..., xn) = fs(Prim(fz, fs)(x0, .., xn), x0, .., xn)
                            let pred_process: Process = Process::Comp {
                                function: function.clone(),
                                args: numbers_to_processes(prepend_number(
                                    first.clone().pred(),
                                    &cont,
                                )),
                            };
                            let mut vec = vec![pred_process];
                            vec.extend(numbers_to_processes(prepend_number(first.pred(), &cont)));
                            Some(Process::Comp {
                                function: succ_func.as_ref().clone(),
                                args: vec,
                            })
                        }
                    }
                    RecursiveFunctions::MuOperator { mu_func } => {
                        // Mu { f, 0, (x0, .., xn), process == f(0, x0, .., xn) }
                        let arg = prepend_number(Number(0), &args_as_tuple);
                        Some(Process::MuOpComp {
                            now_index: 0.into(),
                            args: args_as_tuple.clone(),
                            function: mu_func.as_ref().clone(),
                            process: Box::new(Process::Comp {
                                function: mu_func.as_ref().clone(),
                                args: numbers_to_processes(arg),
                            }),
                        })
                    }
                }
            }
            // Muop(f)(x1, .., xn) := minimum { i | f(i, x1, .., xn) = 0 }
            Process::MuOpComp {
                now_index,
                args,
                function,
                process,
            } => {
                // computation process:
                // Mu { f, i, (x0, .., xn), process != result(v) }
                // => eval process
                if let Some(result) = process.eval_one_step() {
                    return Some(Process::MuOpComp {
                        now_index: now_index.clone(),
                        args: args.clone(),
                        function: function.clone(),
                        process: Box::new(result),
                    });
                }
                // here: process == result(v)

                // Mu { f, i, (x0, .., xn), process == result(v) }
                // => if v == 0 => result(i)
                // => else => Mu { f, i + 1, (x0, .., xn), process == f(i + 1, x0, .., xn) }
                let Some(result) = process.result() else {
                    unreachable!("process is not result");
                };

                if result.is_zero() {
                    Some(Process::Result(now_index.clone()))
                } else {
                    let next_index = now_index.clone().succ();
                    let arg = prepend_number(next_index.clone(), args);
                    Some(Process::MuOpComp {
                        now_index: next_index,
                        args: args.clone(),
                        function: function.clone(),
                        process: Box::new(Process::Comp {
                            function: function.clone(),
                            args: numbers_to_processes(arg),
                        }),
                    })
                }
            }
            Process::Result(_) => None,
        }
    }
    pub fn eval_end(mut self) -> Option<Number> {
        loop {
            if let Some(result) = self.eval_one_step() {
                return result.eval_end();
            } else {
                self = self.eval_one_step().unwrap();
            }
        }
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Process::Comp { function, args } => {
                let args_str: String = args
                    .iter()
                    .map(|arg| format!("{arg},"))
                    .reduce(|str1, str2| str1 + &str2)
                    .unwrap_or_default();
                write!(f, "{function}({args_str})")
            }
            Process::MuOpComp {
                now_index,
                args,
                function,
                process,
            } => write!(f, "Mu[{function},{args:?}:{now_index}]({process})"),
            Process::Result(num) => write!(f, "{num}"),
        }
    }
}

// this is a strcut for any functions which may not be recursive functions
pub struct NaturalFunction {
    parameter_length: usize,
    func: Box<dyn Fn(Vec<Number>) -> Number>,
}

impl NaturalFunction {
    pub fn param(&self) -> usize {
        self.parameter_length
    }
    pub fn unchecked_subst(&self, nums: Vec<Number>) -> Number {
        (self.func)(nums)
    }
    pub fn checked_subst(&self, nums: Vec<Number>) -> Option<Number> {
        if nums.len() != self.parameter_length {
            None
        } else {
            Some(self.unchecked_subst(nums))
        }
    }
}

pub fn interpreter(func: &RecursiveFunctions) -> NaturalFunction {
    match func {
        RecursiveFunctions::ZeroConstant => NaturalFunction {
            parameter_length: 0,
            func: Box::new(|_| Number(0)),
        },
        RecursiveFunctions::Successor => NaturalFunction {
            parameter_length: 1,
            func: Box::new(|vec| vec.first().cloned().unwrap().succ()),
        },
        RecursiveFunctions::Projection { projection_num, .. } => {
            let num = *projection_num;
            NaturalFunction {
                parameter_length: func.parameter_length(),
                func: Box::new(move |tuple| tuple[num].clone()),
            }
        }
        RecursiveFunctions::Composition {
            parameter_length,
            outer_func,
            ref inner_funcs,
        } => {
            let outer_func = interpreter(outer_func);
            let inner_funcs = inner_funcs.iter().map(interpreter).collect::<Vec<_>>();
            let func: Box<dyn Fn(Vec<Number>) -> Number> = Box::new(move |tuple| {
                let result_vec: Vec<Number> = inner_funcs
                    .iter()
                    .map(|func| func.unchecked_subst(tuple.clone()))
                    .collect();
                outer_func.unchecked_subst(result_vec)
            });
            NaturalFunction {
                parameter_length: *parameter_length,
                func,
            }
        }
        RecursiveFunctions::PrimitiveRecursion {
            zero_func,
            succ_func,
        } => {
            let length = zero_func.parameter_length() + 1;
            let zero_func = interpreter(zero_func);
            let succ_func = interpreter(succ_func);
            let this_func_clone = func.clone();
            let function: Box<dyn Fn(Vec<Number>) -> Number> = Box::new(move |vector| {
                let (first, cont) = split_head(&vector).unwrap();
                if first.is_zero() {
                    zero_func.unchecked_subst(cont)
                } else {
                    let pred_result = {
                        let pred_input = prepend_number(first.clone().pred(), &cont);
                        interpreter(&this_func_clone).unchecked_subst(pred_input)
                    };
                    let input = prepend_number(pred_result, &prepend_number(first.pred(), &cont));
                    succ_func.unchecked_subst(input)
                }
            });
            NaturalFunction {
                parameter_length: length,
                func: function,
            }
        }
        RecursiveFunctions::MuOperator { mu_func } => {
            let length = mu_func.parameter_length() - 1;
            let mu_func = interpreter(mu_func);
            let func: Box<dyn Fn(Vec<Number>) -> Number> = Box::new(move |vector| {
                let mut i = 0;
                'lp: loop {
                    let result = mu_func.unchecked_subst(prepend_number(Number(i), &vector));
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
    use std::vec;

    use super::{interpreter, RecursiveFunctions};
    use super::{Number, Process};

    fn nums(values: &[usize]) -> Vec<Number> {
        values.iter().map(|&v| Number(v)).collect()
    }

    #[test]
    fn zero_call() {
        let zero = RecursiveFunctions::zero();
        let zero_func = interpreter(&zero);
        let result = zero_func.checked_subst(Vec::<Number>::new());
        assert_eq!(result, Some(Number(0)));
        let result = zero_func.checked_subst(nums(&[0]));
        assert_eq!(result, None);
    }
    #[test]
    fn succ_call() {
        let succ = RecursiveFunctions::succ();
        let succ_func = interpreter(&succ);
        for i in 0..5 {
            let result = succ_func.checked_subst(nums(&[i]));
            assert_eq!(result, Some(Number(i + 1)))
        }
    }
    #[test]
    fn proj_call() {
        let proj = RecursiveFunctions::projection(1, 0).unwrap();
        let proj_func = interpreter(&proj);
        let result = proj_func.checked_subst(nums(&[0]));
        assert_eq!(result, Some(Number(0)));
        let result = proj_func.checked_subst(nums(&[0, 1]));
        assert_eq!(result, None);

        let proj = RecursiveFunctions::projection(3, 0).unwrap();
        let proj_func = interpreter(&proj);
        let result = proj_func.checked_subst(nums(&[0, 1, 2]));
        assert_eq!(result, Some(Number(0)));
    }
    #[test]
    fn comp_call() {
        let succcc = RecursiveFunctions::composition(
            RecursiveFunctions::succ(),
            vec![RecursiveFunctions::succ()],
        )
        .unwrap();
        let succcc_func = interpreter(&succcc);
        let result = succcc_func.checked_subst(nums(&[0]));
        assert_eq!(result, Some(Number(2)));
        assert!(RecursiveFunctions::composition(RecursiveFunctions::succ(), vec![]).is_err());
        assert!(RecursiveFunctions::composition(
            RecursiveFunctions::zero(),
            vec![RecursiveFunctions::succ()],
        )
        .is_err());
        assert!(RecursiveFunctions::composition(
            RecursiveFunctions::projection(2, 1).unwrap(),
            vec![RecursiveFunctions::succ(), RecursiveFunctions::zero()],
        )
        .is_err());
        let snd_succ = RecursiveFunctions::composition(
            RecursiveFunctions::projection(3, 1).unwrap(),
            vec![
                RecursiveFunctions::succ(),
                RecursiveFunctions::succ(),
                RecursiveFunctions::succ(),
            ],
        )
        .unwrap();
        let func = interpreter(&snd_succ);
        assert_eq!(func.checked_subst(nums(&[0])), Some(Number(1)));

        let snd_succ = RecursiveFunctions::composition(
            RecursiveFunctions::projection(4, 1).unwrap(),
            vec![
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 1).unwrap(),
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 0).unwrap(),
            ],
        )
        .unwrap();
        let func = interpreter(&snd_succ);
        assert_eq!(func.checked_subst(nums(&[0, 1, 2])), Some(Number(1)))
    }
    #[test]
    fn prim_call() {
        let zero_func = RecursiveFunctions::projection(1, 0).unwrap();
        let succ_func = RecursiveFunctions::composition(
            RecursiveFunctions::succ(),
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
        )
        .unwrap();
        let add = RecursiveFunctions::primitive_recursion(zero_func, succ_func).unwrap();
        let add_func = interpreter(&add);
        assert_eq!(add_func.checked_subst(nums(&[0, 0])), Some(Number(0)));
        assert_eq!(add_func.checked_subst(nums(&[0, 1])), Some(Number(1)));
        assert_eq!(add_func.checked_subst(nums(&[1, 0])), Some(Number(1)));
        assert_eq!(add_func.checked_subst(nums(&[1, 1])), Some(Number(2)));
        assert_eq!(add_func.checked_subst(nums(&[2, 2])), Some(Number(4)));
        assert_eq!(add_func.checked_subst(nums(&[2, 3])), Some(Number(5)));
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
        assert_eq!(pred_func.checked_subst(nums(&[0])), Some(Number(0)));
        assert_eq!(pred_func.checked_subst(nums(&[1])), Some(Number(0)));
        assert_eq!(pred_func.checked_subst(nums(&[2])), Some(Number(1)));
        assert_eq!(pred_func.checked_subst(nums(&[3])), Some(Number(2)));
    }
    fn inv_monus() -> RecursiveFunctions {
        RecursiveFunctions::primitive_recursion(
            RecursiveFunctions::projection(1, 0).unwrap(),
            RecursiveFunctions::composition(
                pred_func(),
                vec![RecursiveFunctions::projection(3, 0).unwrap()],
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn monus() -> RecursiveFunctions {
        RecursiveFunctions::composition(
            inv_monus(),
            vec![
                RecursiveFunctions::projection(2, 1).unwrap(),
                RecursiveFunctions::projection(2, 0).unwrap(),
            ],
        )
        .unwrap()
    }
    #[test]
    fn monus_call() {
        let monus = interpreter(&monus());
        assert_eq!(monus.checked_subst(nums(&[0, 0])), Some(Number(0)));
        assert_eq!(monus.checked_subst(nums(&[0, 1])), Some(Number(0)));
        assert_eq!(monus.checked_subst(nums(&[0, 2])), Some(Number(0)));
        assert_eq!(monus.checked_subst(nums(&[1, 0])), Some(Number(1)));
        assert_eq!(monus.checked_subst(nums(&[2, 0])), Some(Number(2)));
        assert_eq!(monus.checked_subst(nums(&[1, 1])), Some(Number(0)));
        assert_eq!(monus.checked_subst(nums(&[2, 2])), Some(Number(0)));
        assert_eq!(monus.checked_subst(nums(&[2, 1])), Some(Number(1)));
    }
    fn id_from_inv_monus() -> RecursiveFunctions {
        RecursiveFunctions::muoperator(inv_monus()).unwrap()
    }
    #[test]
    fn muop_call() {
        let id = interpreter(&id_from_inv_monus());
        assert_eq!(id.checked_subst(nums(&[0])), Some(Number(0)));
        assert_eq!(id.checked_subst(nums(&[1])), Some(Number(1)));
        assert_eq!(id.checked_subst(nums(&[2])), Some(Number(2)));
        assert_eq!(id.checked_subst(nums(&[3])), Some(Number(3)));
    }
    #[test]
    fn process_test_zero() {
        let zero = RecursiveFunctions::zero();
        let mut process = Process::new(zero, vec![]).unwrap();
        let res = loop {
            eprintln!("{process}");
            if let Some(r) = process.result() {
                break r;
            }
            process = process.eval_one_step().unwrap();
        };
        assert_eq!(res, Number(0));
    }
    #[test]
    fn process_test_succ() {
        let succ = RecursiveFunctions::succ();
        let mut process = Process::new(succ.clone(), vec![Number(0)]).unwrap();
        let res = loop {
            eprintln!("{process}");
            if let Some(r) = process.result() {
                break r;
            }
            process = process.eval_one_step().unwrap();
        };
        assert_eq!(res, Number(1));

        let mut process = Process::new(succ, vec![Number(1)]).unwrap();
        let res = loop {
            eprintln!("{process}");
            if let Some(r) = process.result() {
                break r;
            }
            process = process.eval_one_step().unwrap();
        };
        assert_eq!(res, Number(2));
    }
    #[test]
    fn process_test_comp() {
        let succ_succ = RecursiveFunctions::composition(
            RecursiveFunctions::succ(),
            vec![RecursiveFunctions::succ()],
        )
        .unwrap();
        let mut process = Process::new(succ_succ.clone(), vec![Number(0)]).unwrap();
        let res = loop {
            eprintln!("{process}");
            if let Some(r) = process.result() {
                break r;
            }
            process = process.eval_one_step().unwrap();
        };
        assert_eq!(res, Number(2));
    }
    #[test]
    fn process_test_prim() {
        let zero_func = RecursiveFunctions::projection(1, 0).unwrap();
        let succ_func = RecursiveFunctions::composition(
            RecursiveFunctions::succ(),
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
        )
        .unwrap();
        let add = RecursiveFunctions::primitive_recursion(zero_func, succ_func).unwrap();
        let mut process = Process::new(add.clone(), vec![Number(2), Number(3)]).unwrap();
        let res = loop {
            eprintln!("{process}");
            if let Some(r) = process.result() {
                break r;
            }
            process = process.eval_one_step().unwrap();
        };
        assert_eq!(res, Number(5));
    }
    #[test]
    fn process_test_muop() {
        let id = RecursiveFunctions::muoperator(inv_monus()).unwrap();
        let mut process = Process::new(id.clone(), vec![Number(3)]).unwrap();
        let res = loop {
            eprintln!("{process}");
            if let Some(r) = process.result() {
                break r;
            }
            process = process.eval_one_step().unwrap();
        };
        assert_eq!(res, Number(3));
    }
}
