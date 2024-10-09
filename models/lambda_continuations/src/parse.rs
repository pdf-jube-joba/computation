use crate::*;
use anyhow::{bail, Result};
use pest::{iterators::Pair, Parser};
use utils::variable::Var;

#[derive(pest_derive::Parser)]
#[grammar = "continuations.pest"]
struct Ps;

pub fn take_variable(pair: Pair<Rule>) -> Option<Var> {
    if matches!(pair.as_rule(), Rule::variable) {
        Some(pair.as_str().into())
    } else {
        None
    }
}

pub mod nat {
    use super::*;
    use crate::no_ext::Lam;
    use lambda::base::{Base, BaseStruct};

    pub fn parse_lam_nat(code: &str) -> Result<Lam<BaseStruct>> {
        let mut code = Ps::parse(Rule::lam_nat, code)?;
        let Some(pair) = code.next() else {
            bail!("rule が空？");
        };
        match take_lam_nat(pair) {
            Some(e) => Ok(e),
            None => bail!("lam_nat parse 失敗"),
        }
    }

    pub fn take_lam_nat(pair: Pair<Rule>) -> Option<Lam<BaseStruct>> {
        if matches!(pair.as_rule(), Rule::lam_nat) {
            let mut inner = pair.into_inner();
            let pair = inner.next().unwrap();
            take_lam_nat_var(pair.clone())
                .or(take_lam_nat_lam(pair.clone()))
                .or(take_lam_nat_app(pair))
        } else {
            None
        }
    }

    pub fn take_lam_nat_var(pair: Pair<Rule>) -> Option<Lam<BaseStruct>> {
        let v = take_variable(pair)?;
        Some(Lam::Base(Box::new(Base::n_v(v))))
    }

    pub fn take_lam_nat_lam(pair: Pair<Rule>) -> Option<Lam<BaseStruct>> {
        if matches!(pair.as_rule(), Rule::lam_nat_lam) {
            let mut inner = pair.into_inner();

            let x = inner.next().unwrap();
            assert_eq!(x.as_rule(), Rule::variable);
            let x = take_variable(x)?;

            let e = inner.next().unwrap();
            assert_eq!(e.as_rule(), Rule::lam_nat);
            let e = take_lam_nat(e)?;

            Some(Lam::Base(Box::new(Base::n_l(x, e))))
        } else {
            None
        }
    }

    pub fn take_lam_nat_app(pair: Pair<Rule>) -> Option<Lam<BaseStruct>> {
        if matches!(pair.as_rule(), Rule::lam_nat_app) {
            let mut inner = pair.into_inner();

            let e1 = inner.next().unwrap();
            assert_eq!(e1.as_rule(), Rule::lam_nat);
            let e1 = take_lam_nat(e1)?;

            let e2 = inner.next().unwrap();
            assert_eq!(e2.as_rule(), Rule::lam_nat);
            let e2 = take_lam_nat(e2)?;

            Some(Lam::Base(Box::new(Base::n_a(e1, e2))))
        } else {
            None
        }
    }
}

// pub fn take_lam_ext(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext) {
//         let mut inner = pair.into_inner();
//         let pair = inner.next().unwrap();

//         take_lam_ext_var(pair.clone())
//             .or(take_lam_ext_lam(pair.clone()))
//             .or(take_lam_ext_app(pair.clone()))
//             .or(take_lam_ext_zero(pair.clone()))
//             .or(take_lam_ext_succ(pair.clone()))
//             .or(take_lam_ext_pred(pair.clone()))
//             .or(take_lam_ext_ifz(pair.clone()))
//             .or(take_lam_ext_let(pair.clone()))
//             .or(take_lam_ext_rec(pair))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_var(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     let v = take_variable(pair)?;
//     Some(lam_ext::Lam::Var(v))
// }

// pub fn take_lam_ext_lam(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_lam) {
//         let mut inner = pair.into_inner();

//         let x = inner.next().unwrap();
//         assert_eq!(x.as_rule(), Rule::variable);
//         let x = take_variable(x)?;

//         let e = inner.next().unwrap();
//         assert_eq!(e.as_rule(), Rule::lam_ext);
//         let e = take_lam_ext(e)?;

//         Some(lam_ext::Lam::Lam(x, Box::new(e)))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_app(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_app) {
//         let mut inner = pair.into_inner();

//         let e1 = inner.next().unwrap();
//         assert_eq!(e1.as_rule(), Rule::lam_ext);
//         let e1 = take_lam_ext(e1)?;

//         let e2 = inner.next().unwrap();
//         assert_eq!(e2.as_rule(), Rule::lam_ext);
//         let e2 = take_lam_ext(e2)?;

//         Some(lam_ext::Lam::App(Box::new(e1), Box::new(e2)))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_zero(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_zero) {
//         Some(lam_ext::Lam::Zero)
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_succ(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_succ) {
//         let mut inner = pair.into_inner();
//         let e = inner.next().unwrap();
//         let e = take_lam_ext(e)?;
//         Some(lam_ext::Lam::Succ(Box::new(e)))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_pred(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_pred) {
//         let mut inner = pair.into_inner();
//         let e = inner.next().unwrap();
//         let e = take_lam_ext(e)?;
//         Some(lam_ext::Lam::Pred(Box::new(e)))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_ifz(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_ifz) {
//         let mut inner = pair.into_inner();

//         let e1 = inner.next().unwrap();
//         let e1 = take_lam_ext(e1)?;

//         let e2 = inner.next().unwrap();
//         let e2 = take_lam_ext(e2)?;

//         let e3 = inner.next().unwrap();
//         let e3 = take_lam_ext(e3)?;

//         Some(lam_ext::Lam::IfZ(Box::new(e1), Box::new(e2), Box::new(e3)))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_let(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_let) {
//         let mut inner = pair.into_inner();

//         let x = inner.next().unwrap();
//         let x = take_variable(x)?;

//         let e1 = inner.next().unwrap();
//         let e1 = take_lam_ext(e1)?;

//         let e2 = inner.next().unwrap();
//         let e2 = take_lam_ext(e2)?;

//         Some(lam_ext::Lam::Let(x, Box::new(e1), Box::new(e2)))
//     } else {
//         None
//     }
// }

// pub fn take_lam_ext_rec(pair: Pair<Rule>) -> Option<lam_ext::Lam> {
//     if matches!(pair.as_rule(), Rule::lam_ext_rec) {
//         let mut inner = pair.into_inner();

//         let f = inner.next().unwrap();
//         let f = take_variable(f)?;

//         let x = inner.next().unwrap();
//         let x = take_variable(x)?;

//         let e = inner.next().unwrap();
//         let e = take_lam_ext(e)?;

//         Some(lam_ext::Lam::Rec(f, x, Box::new(e)))
//     } else {
//         None
//     }
// }

// pub fn parse_lam_ext(code: &str) -> Result<lam_ext::Lam> {
//     let mut code = Ps::parse(Rule::lam_ext, code)?;
//     let Some(pair) = code.next() else {
//         bail!("rule が空？");
//     };
//     match take_lam_ext(pair) {
//         Some(e) => Ok(e),
//         None => bail!("lam_nat parse 失敗"),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lam() {
        let code = "x";
        let res = Ps::parse(Rule::variable, code);
        res.unwrap();
    }
    #[test]
    fn lam_nat() {
        let check_parse = |code: &str| {
            let e = super::nat::parse_lam_nat(code).unwrap();
            println!("{e:?}");
        };

        let code = "x";
        check_parse(code);

        let code = "fun x => x";
        check_parse(code);

        let code = "@(fun x => x fun x => x)";
        check_parse(code);

        let code = "@((fun x => x) (fun x => (x)))";
        check_parse(code);
    }
    #[test]
    fn lam_ext() {
        let check_parse = |code: &str| {
            // let e = parse_lam_ext(code).unwrap();
            // println!("{e:?}");
        };

        let code = "x";
        check_parse(code);

        let code = "fun x => x";
        check_parse(code);

        let code = "@(fun x => x fun x => x)";
        check_parse(code);

        let code = "@((fun x => x) (fun x => (x)))";
        check_parse(code);

        let code = "zero";
        check_parse(code);

        let code = "succ zero";
        check_parse(code);

        let code = "pred succ zero";
        check_parse(code);

        let code = "ifz x then x else x";
        check_parse(code);

        let code = "let id = fun x => x in @(id id)";
        check_parse(code);

        let code = "rec f x = zero";
        check_parse(code);
    }
}
