use lambda_calculus::machine::{utils::*, LambdaTerm};
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
    if let LambdaTerm::Abstraction(var1, term) = term {
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
        } else {
            return Err(());
        }
    } else {
        return Err(());
    }
}

// \xyz.x(\pq.q(py))(\v.z)(\v.v) = \xyz.xMNL
pub fn pred() -> LambdaTerm {
    let m = LambdaTerm::abs(
        3,
        LambdaTerm::abs(
            4,
            LambdaTerm::app(
                LambdaTerm::var(4),
                LambdaTerm::app(LambdaTerm::var(3), LambdaTerm::var(0)),
            ),
        ),
    );
    let n = LambdaTerm::abs(5, LambdaTerm::var(1));
    let l = LambdaTerm::abs(5, LambdaTerm::var(5));
    LambdaTerm::abs(
        2,
        LambdaTerm::abs(
            0,
            LambdaTerm::abs(
                1,
                LambdaTerm::app(
                    LambdaTerm::app(LambdaTerm::app(LambdaTerm::var(2), m), n),
                    l,
                ),
            ),
        ),
    )
}

pub fn is_zero() -> LambdaTerm {
    LambdaTerm::abs(
        0,
        LambdaTerm::app(
            LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::abs(1, false_lambda())),
            true_lambda(),
        ),
    )
}

// \xyz.y(xyz)
pub fn succ() -> LambdaTerm {
    LambdaTerm::abs(
        2,
        LambdaTerm::abs(
            0,
            LambdaTerm::abs(
                1,
                LambdaTerm::app(
                    LambdaTerm::var(0),
                    LambdaTerm::app(
                        LambdaTerm::app(LambdaTerm::var(2), LambdaTerm::var(0)),
                        LambdaTerm::var(1),
                    ),
                ),
            ),
        ),
    )
}

// \x1,,,xn.xi
pub fn proj(n: usize, i: usize) -> Result<LambdaTerm, ()> {
    if i <= n {
        Err(())
    } else {
        Ok(take_n_abs((0..n).collect(), LambdaTerm::var(i)))
    }
}

pub fn composition(n: usize, inner: Vec<LambdaTerm>, outer: LambdaTerm) -> LambdaTerm {
    let mut v = vec![outer];
    v.extend(inner.into_iter().map(|term| {
        fold_left({
            let mut v2 = vec![term];
            v2.extend((0..n).map(LambdaTerm::var));
            v2
        })
    }));
    let fold = fold_left(v);
    take_n_abs((0..n).collect(), fold)
}

pub fn primitive_recursion(n: usize, f: LambdaTerm, g: LambdaTerm) -> LambdaTerm {
    // is_zero 0
    let is_zero = LambdaTerm::app(is_zero(), LambdaTerm::var(0));

    // f x1 ... xn
    let f = fold_left({
        let mut v = vec![f];
        v.extend((1..=n).map(LambdaTerm::var));
        v
    });

    // g (n+1 (pred 0) 1 ... n) (pred 0) 1 ... n
    let g = fold_left({
        let pred_0 = LambdaTerm::app(pred(), LambdaTerm::var(0));
        let p = {
            let mut v = vec![LambdaTerm::var(n + 1), pred_0.clone()];
            v.extend((1..=n).map(LambdaTerm::var));
            fold_left(v)
        };
        let mut t = vec![g, p, pred_0];
        t.extend((1..=n).map(LambdaTerm::var));
        t
    });
    let fix = if_lambda(is_zero, f, g);

    // Y (\n+1 1...n. if_lambda is_zero f g) =>
    // n+1 <=> H として H x0 x1 ... xn = if_lambda is_zero f g
    LambdaTerm::app(
        y_combinator(),
        take_n_abs(
            {
                let mut v = vec![n + 1];
                v.extend(1..=n);
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
            v.extend((0..=n).map(LambdaTerm::var));
            v
        }),
    );
    let rec = fold_left({
        let mut v = vec![
            LambdaTerm::var(n + 1),
            LambdaTerm::app(succ(), LambdaTerm::var(0)),
        ];
        v.extend((1..=n).map(LambdaTerm::var));
        v
    });
    let fix = if_lambda(is_zero, LambdaTerm::var(0), rec);
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

#[cfg(test)]
mod tests {
    use lambda_calculus::{
        machine::{alpha_eq, is_normal, left_most_reduction},
        manipulation::parse,
    };

    use super::*;

    #[test]
    fn lambda_term_and_number_test() {
        for i in 0..10 {
            let mut lam = number_to_lambda_term(i.into());
            loop {
                eprintln!("{} {}", is_normal(lam.clone()), lam.to_string());
                if is_normal(lam.clone()) {
                    break;
                }
                lam = left_most_reduction(lam);
            }
            let res = lambda_term_to_number(lam);
            assert_eq!(res, Ok(i.into()))
        }
    }

    #[test]
    fn is_zero_test() {
        let zero = number_to_lambda_term(0.into());
        let mut lam = LambdaTerm::app(is_zero(), zero);
        loop {
            eprintln!("{} {}", is_normal(lam.clone()), lam.to_string());
            if is_normal(lam.clone()) {
                break;
            }
            lam = left_most_reduction(lam);
        }
        assert!(is_true(lam));

        let one = number_to_lambda_term(1.into());
        let mut lam = LambdaTerm::app(is_zero(), one);
        loop {
            eprintln!("{} {}", is_normal(lam.clone()), lam.to_string());
            if is_normal(lam.clone()) {
                break;
            }
            lam = left_most_reduction(lam);
        }
        assert!(is_false(lam));

        let two = number_to_lambda_term(2.into());
        let mut lam = LambdaTerm::app(is_zero(), two);
        loop {
            eprintln!("{} {}", is_normal(lam.clone()), lam.to_string());
            if is_normal(lam.clone()) {
                break;
            }
            lam = left_most_reduction(lam);
        }
        assert!(is_false(lam));
    }

    #[test]
    fn proj_test() {
        let pj = proj(2, 0).unwrap();
        let expect = parse::parse_lambda("\\0.\\1.0").unwrap();
        eprintln!("{}", pj.to_string());
        assert!(alpha_eq(&pj, &expect));

        let pj = proj(2, 1).unwrap();
        let expect = parse::parse_lambda("\\0.\\1.1").unwrap();
        eprintln!("{}", pj.to_string());
        assert!(alpha_eq(&pj, &expect));

        let pj = proj(3, 0).unwrap();
        let expect = parse::parse_lambda("\\0.\\1.\\2.0").unwrap();
        eprintln!("{}", pj.to_string());
        assert!(alpha_eq(&pj, &expect));

        let pj = proj(3, 1).unwrap();
        let expect = parse::parse_lambda("\\0.\\1.\\2.1").unwrap();
        eprintln!("{}", pj.to_string());
        assert!(alpha_eq(&pj, &expect));

        let pj = proj(3, 2).unwrap();
        let expect = parse::parse_lambda("\\0.\\1.\\2.2").unwrap();
        eprintln!("{}", pj.to_string());
        assert!(alpha_eq(&pj, &expect));
    }
}
