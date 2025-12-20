pub mod utility {
    use crate::machine::LambdaTerm;
    use utils::variable::Var;

    pub fn apps(first: LambdaTerm, remains: Vec<LambdaTerm>) -> LambdaTerm {
        let mut term = first;
        for remain in remains {
            term = LambdaTerm::App(term.into(), remain.into());
        }
        term
    }

    pub fn app_with_nonepmty(all: Vec<LambdaTerm>) -> LambdaTerm {
        assert!(!all.is_empty());
        let term = all[0].clone();
        let remains = all[1..].to_vec();
        apps(term, remains)
    }

    pub fn lambdas(pres: Vec<Var>, last: LambdaTerm) -> LambdaTerm {
        let mut term = last;
        for pre in pres.into_iter().rev() {
            term = LambdaTerm::Abs(pre, term.into());
        }
        term
    }

    #[macro_export]
    macro_rules! lam {
        ($v:expr, $t:expr) => {
            $crate::machine::LambdaTerm::Abs($v, Box::new($t))
        };
    }

    // e1 e2 ... en = (((e1 e2) e3) ... en)
    #[macro_export]
    macro_rules! app {
        ($( $x:expr ),*) => {
            {
                let alls = vec![$($x),*];
                $crate::manipulation::utility::app_with_nonepmty(alls)
            }
        };
    }

    pub use {app, lam};

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::machine::LambdaTerm;

        fn v(var: &Var) -> LambdaTerm {
            LambdaTerm::Var(var.clone())
        }
        fn abs(var: &Var, body: LambdaTerm) -> LambdaTerm {
            LambdaTerm::Abs(var.clone(), body.into())
        }
        fn app(lhs: LambdaTerm, rhs: LambdaTerm) -> LambdaTerm {
            LambdaTerm::App(lhs.into(), rhs.into())
        }

        #[test]
        fn test_var() {
            let term: Var = "x".into();
            assert_eq!(term.as_str(), "x");
        }
        #[test]
        fn test_lam() {
            let x = Var::from("x");
            let y = Var::from("y");
            let term = lam!(x.clone(), v(&y));
            assert_eq!(term, abs(&x, v(&y)));
        }
        #[test]
        fn test_app() {
            let x = Var::from("x");
            let y = Var::from("y");
            let z = Var::from("z");

            // "x"
            let term = app!(v(&x));
            assert_eq!(term, v(&x));

            // "x y"
            let term = app!(v(&x), v(&y));
            assert_eq!(term, app(v(&x), v(&y)));

            // "(x y) z"
            let term = app!(v(&x), v(&y), v(&z));
            assert_eq!(term, app(app(v(&x), v(&y)), v(&z)));
        }
    }
}

pub mod parse {
    use pest::{iterators::Pair, Parser};
    use utils::variable::Var;

    use crate::{
        machine::LambdaTerm,
        manipulation::utility::{self, app_with_nonepmty},
    };

    #[derive(pest_derive::Parser)]
    #[grammar = "parse.pest"]
    pub struct Ps;

    pub fn parse_exp(p: Pair<Rule>, ref_vars: &mut Vec<Var>) -> Result<LambdaTerm, String> {
        assert_eq!(p.as_rule(), Rule::exp);
        let mut ps = p.into_inner();
        let term = ps.next().unwrap();
        match term.as_rule() {
            Rule::var => {
                let var_str = term.as_str();
                let var: Var = ref_vars
                    .iter()
                    .rev()
                    .find_map(|v| {
                        if v.as_str() == var_str {
                            Some(v.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Var::new(var_str));
                Ok(LambdaTerm::Var(var))
            }
            Rule::abs => {
                let mut ps = term.into_inner();
                let mut count = 0;
                while ps.peek().map(|p| p.as_rule()) == Some(Rule::var) {
                    let var: Var = ps.next().unwrap().as_str().into();
                    ref_vars.push(var);
                    count += 1;
                }
                let exp = parse_exp(ps.next().unwrap(), ref_vars)?;
                let exp = utility::lambdas(
                    ref_vars[ref_vars.len() - count..ref_vars.len()].to_vec(),
                    exp,
                );
                for _ in 0..count {
                    ref_vars.pop();
                }
                Ok(exp)
            }
            Rule::exp_paren => {
                let mut ps = term.into_inner();
                let first = parse_exp(ps.next().unwrap(), ref_vars)?;
                let remains = ps
                    .map(|p| parse_exp(p, ref_vars))
                    .collect::<Result<Vec<_>, _>>()?;
                let term = utility::apps(first, remains);
                Ok(term)
            }
            _ => unreachable!("exp should be \"var\" or \"abs\" or \"exp_paren\""),
        }
    }

    pub fn parse_lambda(code: &str) -> Result<LambdaTerm, String> {
        let mut code = Ps::parse(Rule::exp, code.trim()).map_err(|e| e.to_string())?;
        let p = code.next().unwrap();
        parse_exp(p, &mut Vec::new())
    }

    pub fn parse_lambda_read_to_end(code: &str) -> Result<LambdaTerm, String> {
        let mut code =
            Ps::parse(Rule::lambda_read_to_end, code.trim()).map_err(|e| e.to_string())?;
        let p = code.next().unwrap();
        let ps: Vec<_> = p
            .into_inner()
            .filter(|p| p.as_rule() == Rule::exp)
            .map(|p| parse_exp(p, &mut Vec::new()))
            .collect::<Result<_, _>>()?;
        debug_assert!(!ps.is_empty());
        Ok(app_with_nonepmty(ps))
    }

    #[cfg(test)]
    mod tests {
        use utils::TextCodec;

        use super::*;
        #[test]
        fn parse_test() {
            let code = "x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "[x]");

            let code = "(x)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "[x]");

            let code = " x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "[x]");

            let code = "\\x.x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "\\x.[x]");

            let code = "(\\x.x)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "\\x.[x]");

            let code = "(x y)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "([x] [y])");

            let code = "x y";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "([x] [y])");

            let code = "(x y z)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "(([x] [y]) [z])");

            let code = "x y z";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "(([x] [y]) [z])");

            let code = "(\\x.x y)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "(\\x.[x] [y])");

            let code = "(\\x.x) y";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "(\\x.[x] [y])");

            let code = "(\\x. (x y))";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "\\x.([x] [y])");

            let code = "\\ x y. x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(format!("{}", term.print()), "\\x.\\y.[x]");
        }
    }
}
