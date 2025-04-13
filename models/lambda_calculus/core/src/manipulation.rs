pub mod utility {
    use crate::machine::LambdaTerm;
    use utils::variable::Var;

    pub fn apps(first: LambdaTerm, remains: Vec<LambdaTerm>) -> LambdaTerm {
        let mut term = first;
        for remain in remains {
            term = LambdaTerm::app(term, remain);
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
            term = LambdaTerm::abs(pre, term);
        }
        term
    }

    #[macro_export]
    macro_rules! var {
        ($v:expr) => {
            utils::variable::Var::from(($v)).into()
        };
    }

    #[macro_export]
    macro_rules! lam {
        ($v:expr, $t:expr) => {
            $crate::machine::LambdaTerm::abs($v, $t)
        };
    }

    // e1 e2 ... en = (((e1 e2) e3) ... en)
    #[macro_export]
    macro_rules! app {
        ($( $x:expr ),*) => {
            {
                let mut alls = vec![];
                $(
                    alls.push($x);
                )*
                $crate::manipulation::utility::app_with_nonepmty(alls)
            }
        };
    }

    pub use {app, lam, var};

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::machine::LambdaTerm;

        #[test]
        fn test_var() {
            let term: Var = var!("x");
            assert_eq!(term, Var::from("x"));
        }
        #[test]
        fn test_lam() {
            let term = lam!(var!("x"), var!("y"));
            assert_eq!(term, LambdaTerm::abs(var!("x"), LambdaTerm::var(var!("y"))));
        }
        #[test]
        fn test_app() {
            // "x"
            let term = app!(var!("x"));
            assert_eq!(term, LambdaTerm::var(var!("x")));

            // "x y"
            let term = app!(var!("x"), var!("y"));
            assert_eq!(
                term,
                LambdaTerm::app(LambdaTerm::var(var!("x")), LambdaTerm::var(var!("y")))
            );

            // "(x y) z"
            let term = app!(var!("x"), var!("y"), var!("z"));
            assert_eq!(
                term,
                LambdaTerm::app(
                    LambdaTerm::app(LambdaTerm::var(var!("x")), LambdaTerm::var(var!("y"))),
                    LambdaTerm::var(var!("z"))
                )
            );
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

    pub fn parse_exp(p: Pair<Rule>) -> Result<LambdaTerm, String> {
        assert_eq!(p.as_rule(), Rule::exp);
        let mut ps = p.into_inner();
        let term = ps.next().unwrap();
        match term.as_rule() {
            Rule::var => {
                let var: Var = term.as_str().into();
                Ok(LambdaTerm::var(var))
            }
            Rule::abs => {
                let mut ps = term.into_inner();
                let mut vars = vec![];
                while ps.peek().map(|p| p.as_rule()) == Some(Rule::var) {
                    let var: Var = ps.next().unwrap().as_str().into();
                    vars.push(var);
                }
                let exp = parse_exp(ps.next().unwrap())?;
                Ok(utility::lambdas(vars, exp))
            }
            Rule::exp_paren => {
                let mut ps = term.into_inner();
                let first = parse_exp(ps.next().unwrap())?;
                let remains = ps.map(|p| parse_exp(p)).collect::<Result<Vec<_>, _>>()?;
                let term = utility::apps(first, remains);
                Ok(term)
            }
            _ => unreachable!("exp should be \"var\" or \"abs\" or \"exp_paren\""),
        }
    }

    pub fn parse_lambda(code: &str) -> Result<LambdaTerm, String> {
        let mut code = Ps::parse(Rule::exp, code.trim()).map_err(|e| e.to_string())?;
        let p = code.next().unwrap();
        parse_exp(p)
    }

    pub fn parse_lambda_read_to_end(code: &str) -> Result<LambdaTerm, String> {
        let mut code =
            Ps::parse(Rule::lambda_read_to_end, code.trim()).map_err(|e| e.to_string())?;
        let p = code.next().unwrap();
        let ps: Vec<_> = p
            .into_inner()
            .filter(|p| p.as_rule() == Rule::exp)
            .map(|p| parse_exp(p))
            .collect::<Result<_, _>>()?;
        debug_assert!(!ps.is_empty());
        Ok(app_with_nonepmty(ps))
    }

    #[cfg(test)]
    mod tests {
        use crate::manipulation::utility::{app, lam, var};

        use super::*;
        #[test]
        fn parse_test() {
            let code = "x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, var!("x"));

            let code = "(x)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, var!("x"));

            let code = " x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, var!("x"));

            let code = "\\x.x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, lam!(var!("x"), var!("x")));

            let code = "(\\x.x)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, lam!(var!("x"), var!("x")));

            let code = "(x y)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, app!(var!("x"), var!("y")));

            let code = "x y";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, app!(var!("x"), var!("y")));

            let code = "(x y z)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, app!(var!("x"), var!("y"), var!("z")));

            let code = "x y z";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, app!(var!("x"), var!("y"), var!("z")));

            let code = "(\\x.x y)";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, app!(lam!(var!("x"), var!("x")), var!("y")));

            let code = "(\\x.x) y";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, app!(lam!(var!("x"), var!("x")), var!("y")));

            let code = "(\\x. (x y))";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, lam!(var!("x"), app!(var!("x"), var!("y"))));

            let code = "\\ x y. x";
            let term = parse_lambda_read_to_end(code).unwrap();
            assert_eq!(term, lam!(var!("x"), lam!(var!("y"), var!("x"))));
        }
    }
}
