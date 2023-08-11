use std::process::Termination;

use lambda_calculus::machine::LambdaTerm;
use recursive_function::machine::Number;

pub fn number_to_lambda_term(num: Number) -> LambdaTerm {
    fn term(num: Number) -> LambdaTerm {
        if num.is_zero() {
            LambdaTerm::var(1)
        } else {
            LambdaTerm::app(LambdaTerm::var(0), term(num.pred()))
        }
    }
    LambdaTerm::abs(0, LambdaTerm::abs(1, term(num)))
}

pub fn lambda_term_to_number(term: LambdaTerm) -> Result<Number, ()> {
    if let  LambdaTerm::Abstraction( var1, term) = term {
        if let LambdaTerm::Abstraction(var2, term) = *term {
            let mut iter_term = *term;
            for i in 0.. {
                match &iter_term {
                    LambdaTerm::Variable(ref var) => {
                        if *var == var2 {
                            return Ok(i.into());
                        } else {
                            return Err(());
                        }
                    }
                    LambdaTerm::Application(ref var, ref term2) => {
                        if let LambdaTerm::Variable(v) = *var.clone() {
                            if v == var1 {
                                iter_term = *term2.to_owned();
                                continue;
                            }
                        } else {
                            return Err(());
                        }
                    }
                    _ => {
                        return Err(());
                    }
                }
            }
            unreachable!()
        } else  {
            return  Err(());
        }
    } else {
        return Err(());
    }
}

pub fn zero() -> LambdaTerm {
    LambdaTerm::abs(0, LambdaTerm::var(0))
}

pub fn succ() -> LambdaTerm {
    LambdaTerm::abs(2, LambdaTerm::abs(0, LambdaTerm::abs(1, 
        LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::app(
            LambdaTerm::app(LambdaTerm::var(2), LambdaTerm::var(0)), LambdaTerm::var(1)
        ))
    )))
}
