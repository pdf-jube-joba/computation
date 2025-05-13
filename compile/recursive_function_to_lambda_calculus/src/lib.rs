use lambda_calculus_core::example::*;
use lambda_calculus_core::machine::LambdaTerm;
use recursive_function_core::machine::RecursiveFunctions;
use utils::number::*;
use utils::variable::Var;

pub fn number_to_lambda_term(num: Number) -> LambdaTerm {
    fn term(num: Number) -> LambdaTerm {
        if num.is_zero() {
            Var::new_u(1).into()
        } else {
            LambdaTerm::app(LambdaTerm::var(Var::new_u(0)), term(num.pred()))
        }
    }
    LambdaTerm::abs(Var::new_u(0), LambdaTerm::abs(Var::new_u(1), term(num)))
}

pub fn lambda_term_to_number(term: LambdaTerm) -> Option<Number> {
    let (var1, var2, term) = if let LambdaTerm::Abstraction(var1, term) = term {
        if let LambdaTerm::Abstraction(var2, term) = *term {
            (var1, var2, term)
        } else {
            return None;
        }
    } else {
        return None;
    };
    let mut iter_term = *term;
    for i in 0.. {
        match &iter_term {
            LambdaTerm::Variable(var) => {
                if *var == var2 {
                    return Some(i.into());
                } else {
                    return None;
                }
            }
            LambdaTerm::Application(var, term2) => {
                if let LambdaTerm::Variable(v) = *var.clone() {
                    if v == var1 {
                        iter_term = *term2.to_owned();
                        continue;
                    }
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }
    unreachable!()
}

// \xyz.x(\pq.q(py))(\v.z)(\v.v) = \xyz.xMNL
pub fn pred() -> LambdaTerm {
    let m = LambdaTerm::abs(
        3.into(),
        LambdaTerm::abs(
            4.into(),
            LambdaTerm::app(
                LambdaTerm::var(4.into()),
                LambdaTerm::app(LambdaTerm::var(3.into()), LambdaTerm::var(0.into())),
            ),
        ),
    );
    let n = LambdaTerm::abs(5.into(), LambdaTerm::var(1.into()));
    let l = LambdaTerm::abs(5.into(), LambdaTerm::var(5.into()));
    LambdaTerm::abs(
        2.into(),
        LambdaTerm::abs(
            0.into(),
            LambdaTerm::abs(
                1.into(),
                LambdaTerm::app(
                    LambdaTerm::app(LambdaTerm::app(LambdaTerm::var(2.into()), m), n),
                    l,
                ),
            ),
        ),
    )
}

pub fn is_zero() -> LambdaTerm {
    LambdaTerm::abs(
        0.into(),
        LambdaTerm::app(
            LambdaTerm::app(
                LambdaTerm::var(0.into()),
                LambdaTerm::abs(1.into(), false_lambda()),
            ),
            true_lambda(),
        ),
    )
}

// \xyz.y(xyz)
pub fn succ() -> LambdaTerm {
    LambdaTerm::abs(
        2.into(),
        LambdaTerm::abs(
            0.into(),
            LambdaTerm::abs(
                1.into(),
                LambdaTerm::app(
                    LambdaTerm::var(0.into()),
                    LambdaTerm::app(
                        LambdaTerm::app(LambdaTerm::var(2.into()), LambdaTerm::var(0.into())),
                        LambdaTerm::var(1.into()),
                    ),
                ),
            ),
        ),
    )
}

// \x1,,,xn.xi
pub fn projection(n: usize, i: usize) -> Option<LambdaTerm> {
    if n < i {
        None
    } else {
        Some(take_n_abs((0..n).collect(), LambdaTerm::var(i.into())))
    }
}

// \x1,,,xn. outer (inner x1,,,xn) ,,, (inner x1,,,xn)
pub fn composition(n: usize, inner: Vec<LambdaTerm>, outer: LambdaTerm) -> LambdaTerm {
    let mut v = vec![outer];
    v.extend(inner.into_iter().map(|term| {
        fold_left({
            let mut v2 = vec![term];
            v2.extend((0..n).map(|i| LambdaTerm::var(i.into())));
            v2
        })
    }));
    let fold = fold_left(v);
    take_n_abs((0..n).collect(), fold)
}

// THIS = \x0,,,xn. if (iszero x0) (f x1,,,xn) (g (THIS (pred x0) x1,,,xn) (pred x0) x1,,,xn)
pub fn primitive_recursion(n: usize, f: LambdaTerm, g: LambdaTerm) -> LambdaTerm {
    // is_zero 0
    let is_zero = LambdaTerm::app(is_zero(), LambdaTerm::var(0.into()));

    // f x1 ... xn
    let f_new = fold_left({
        let mut v = vec![f];
        v.extend((1..=n).map(|i| LambdaTerm::var(i.into())));
        v
    });

    // g (n+1 (pred 0) 1 ... n) (pred 0) 1 ... n
    let g_new = fold_left({
        let pred_0 = LambdaTerm::app(pred(), LambdaTerm::var(0.into()));
        let p = {
            let mut v = vec![LambdaTerm::var((n + 1).into()), pred_0.clone()];
            v.extend((1..=n).map(|i| LambdaTerm::var(i.into())));
            fold_left(v)
        };
        let mut t = vec![g, p, pred_0];
        t.extend((1..=n).map(|i| LambdaTerm::var(i.into())));
        t
    });
    let fix = if_lambda(is_zero, f_new, g_new);

    // Y (\n+1 1...n. if_lambda is_zero f g) =>
    // n+1 <=> H として H x0 x1 ... xn = if_lambda is_zero f g
    LambdaTerm::app(
        y_combinator(),
        take_n_abs(
            {
                let mut v = vec![n + 1];
                v.extend(0..=n);
                v
            },
            fix,
        ),
    )
}

pub fn mu_recursion(n: usize, f: LambdaTerm) -> LambdaTerm {
    let is_zero = take_n_abs(
        (0..=n).collect(),
        fold_left({
            let mut v = vec![f];
            v.extend((0..=n).map(|i| LambdaTerm::var(i.into())));
            v
        }),
    );
    let rec = fold_left({
        let mut v = vec![
            LambdaTerm::var((n + 1).into()),
            LambdaTerm::app(succ(), LambdaTerm::var(0.into())),
        ];
        v.extend((1..=n).map(|i| LambdaTerm::var(i.into())));
        v
    });
    let fix = if_lambda(is_zero, LambdaTerm::var(0.into()), rec);
    LambdaTerm::app(
        y_combinator(),
        take_n_abs(
            {
                let mut v = vec![n + 1];
                v.extend(0..=n);
                v
            },
            fix,
        ),
    )
}

pub fn compile(func: &RecursiveFunctions) -> LambdaTerm {
    match func {
        RecursiveFunctions::ZeroConstant => number_to_lambda_term(0.into()),
        RecursiveFunctions::Successor => succ(),
        RecursiveFunctions::Projection {
            parameter_length,
            projection_num,
        } => projection(*parameter_length, *projection_num).unwrap(),
        RecursiveFunctions::Composition {
            parameter_length,
            outer_func,
            inner_funcs,
        } => composition(
            *parameter_length,
            inner_funcs.iter().map(compile).collect(),
            compile(outer_func.as_ref()),
        ),
        RecursiveFunctions::PrimitiveRecursion {
            zero_func,
            succ_func,
        } => primitive_recursion(
            zero_func.parameter_length() + 1,
            compile(zero_func.as_ref()),
            compile(succ_func.as_ref()),
        ),
        RecursiveFunctions::MuOperator { mu_func } => {
            mu_recursion(mu_func.parameter_length(), compile(mu_func.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    use lambda_calculus_core::{
        machine::{alpha_eq, is_normal, left_most_reduction},
        manipulation::parse,
    };

    use super::*;

    #[test]
    fn lambda_term_and_number_test() {
        for i in 0..10 {
            let mut lam = number_to_lambda_term(i.into());
            loop {
                eprintln!("{} {}", is_normal(&lam), lam);
                if is_normal(&lam) {
                    break;
                }
                lam = left_most_reduction(lam).unwrap();
            }
            let res = lambda_term_to_number(lam);
            assert_eq!(res, Some(i.into()))
        }
    }

    #[test]
    fn is_zero_test() {
        let zero = number_to_lambda_term(0.into());
        let mut lam = LambdaTerm::app(is_zero(), zero);
        loop {
            eprintln!("{} {}", is_normal(&lam), lam);
            if is_normal(&lam) {
                break;
            }
            lam = left_most_reduction(lam).unwrap();
        }
        assert!(is_true(lam));

        let one = number_to_lambda_term(1.into());
        let mut lam = LambdaTerm::app(is_zero(), one);
        loop {
            eprintln!("{} {}", is_normal(&lam), lam);
            if is_normal(&lam) {
                break;
            }
            lam = left_most_reduction(lam).unwrap();
        }
        assert!(is_false(lam));

        let two = number_to_lambda_term(2.into());
        let mut lam = LambdaTerm::app(is_zero(), two);
        loop {
            eprintln!("{} {}", is_normal(&lam), lam);
            if is_normal(&lam) {
                break;
            }
            lam = left_most_reduction(lam).unwrap();
        }
        assert!(is_false(lam));
    }

    #[test]
    fn proj_test() {
        let pj = projection(2, 0).unwrap();
        let expect = parse::parse_lambda("\\s.\\z.s").unwrap();
        eprintln!("{}", pj);
        assert!(alpha_eq(&pj, &expect));

        let pj = projection(2, 1).unwrap();
        let expect = parse::parse_lambda("\\s.\\z.z").unwrap();
        eprintln!("{}", pj);
        assert!(alpha_eq(&pj, &expect));

        let pj = projection(3, 0).unwrap();
        let expect = parse::parse_lambda("\\x.\\y.\\z.x").unwrap();
        eprintln!("{}", pj);
        assert!(alpha_eq(&pj, &expect));

        let pj = projection(3, 1).unwrap();
        let expect = parse::parse_lambda("\\x.\\y.\\z.y").unwrap();
        eprintln!("{}", pj);
        assert!(alpha_eq(&pj, &expect));

        let pj = projection(3, 2).unwrap();
        let expect = parse::parse_lambda("\\x.\\y.\\z.z").unwrap();
        eprintln!("{}", pj);
        assert!(alpha_eq(&pj, &expect));
    }
}
