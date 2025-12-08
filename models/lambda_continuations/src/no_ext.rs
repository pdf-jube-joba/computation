use crate::{
    lambda::{lam_to_value, num_to_exp, Lam, LamValue},
    traits::{LambdaExt, Step},
};

impl Step for Lam {
    type Value = LamValue;
    fn is_value(&self) -> Option<Self::Value> {
        lam_to_value(self.clone())
    }
    fn step(self) -> Option<Self> {
        match self {
            Lam::Var { .. } => None,
            Lam::Lam { .. } => None,
            Lam::App { e1, e2 } => {
                let e1 = *e1;
                let e2 = *e2;
                match (e1.is_value(), e2.is_value()) {
                    (Some(LamValue::Fun { var, body }), Some(_)) => Some(body.subst(var, e2)),
                    (Some(LamValue::Num(_)), Some(_)) => None,
                    (Some(_), None) => Some(Lam::n_a(e1, e2.step()?)),
                    (None, _) => Some(Lam::n_a(e1.step()?, e2)),
                }
            }
            Lam::Zero => None,
            Lam::Succ { succ } => {
                let succ = *succ;
                if succ.is_value().is_none() {
                    Some(Lam::n_s(succ.step()?))
                } else {
                    None
                }
            }
            Lam::Pred { pred } => {
                let pred = *pred;
                if let Some(v) = pred.is_value() {
                    match v {
                        LamValue::Fun { .. } => None,
                        LamValue::Num(number) => Some(num_to_exp(number.pred())),
                    }
                } else {
                    Some(Lam::n_p(pred.step()?))
                }
            }
            Lam::IfZ { cond, tcase, fcase } => {
                let cond = *cond;
                if let Some(v) = cond.is_value() {
                    match v {
                        LamValue::Fun { .. } => None,
                        LamValue::Num(number) => {
                            if number.is_zero() {
                                Some(*tcase)
                            } else {
                                Some(*fcase)
                            }
                        }
                    }
                } else {
                    Some(
                        Lam::n_i(
                            cond.step()?,
                            *tcase,
                            *fcase,
                        ),
                    )
                }
            }
            Lam::Let { var, bind, body } => Some(Lam::n_a(Lam::n_l(var, *body), *bind)),
            Lam::Rec { fix, var, body } => {
                let body_term = (*body).clone();
                let rec_term = Lam::n_r(fix.clone(), var.clone(), body_term.clone());
                Some(Lam::n_l(var, body_term.subst(fix, rec_term)))
            }
        }
    }
}

pub fn print(t: &Lam) -> String {
    match t {
        Lam::Var { var } => format!("{var}"),
        Lam::Lam { var, body } => format!("fun {var} => {}", print(body)),
        Lam::App { e1, e2 } => format!("({} {})", print(e1), print(e2)),
        Lam::Zero => "0".to_string(),
        Lam::Succ { succ } => format!("S {}", print(succ)),
        Lam::Pred { pred } => format!("P {}", print(pred)),
        Lam::IfZ { cond, tcase, fcase } => {
            format!("if {} then {} else {}", print(cond), print(tcase), print(fcase))
        }
        Lam::Let { var, bind, body } => format!("let {var} = {} in \n {}", print(bind), print(body)),
        Lam::Rec { fix, var, body } => format!("rec {fix} {var} = {}", print(body)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{bapp, blam, bvar, eapp, eif, elam, elet, epred, erec, esucc, evar, ezero};

    #[test]
    fn test_variables() {
        let l: Lam = bvar!("x");
        assert_eq!(l.free_variables(), vec!["x".to_string()].into_iter().collect());
        assert_eq!(l.bound_variables(), Default::default());

        let l: Lam = blam!("x", bvar!("x"));
        assert_eq!(l.free_variables(), Default::default());
        assert_eq!(l.bound_variables(), vec!["x".to_string()].into_iter().collect());

        let l: Lam = blam!("x", bvar!("y"));
        assert_eq!(l.free_variables(), vec!["y".to_string()].into_iter().collect());
        assert_eq!(l.bound_variables(), vec!["x".to_string()].into_iter().collect());

        let l: Lam = bapp!(bvar!("x"), blam!("x", bvar!("z")));
        assert_eq!(
            l.free_variables(),
            vec!["x".to_string(), "z".to_string()]
                .into_iter()
                .collect()
        );
        assert_eq!(l.bound_variables(), vec!["x".to_string()].into_iter().collect());
    }

    #[test]
    fn test_subst_alpha() {
        let l1: Lam = blam!("x", blam!("x", bvar!("x")));
        let l2: Lam = blam!("x", blam!("x", bvar!("y")));
        let l3: Lam = blam!("x", blam!("y", bvar!("x")));
        let l4: Lam = blam!("x", blam!("y", bvar!("y")));
        let l5: Lam = blam!("y", blam!("x", bvar!("x")));
        let l6: Lam = blam!("y", blam!("x", bvar!("y")));
        let l7: Lam = blam!("y", blam!("y", bvar!("x")));
        let l8: Lam = blam!("y", blam!("y", bvar!("y")));

        let set1 = vec![l1, l4, l5, l8];
        let set2 = vec![l3, l6];
        for t1 in &set1 {
            for t2 in &set1 {
                assert!(t1.alpha_eq(t2));
            }
        }
        for t1 in &set2 {
            for t2 in &set2 {
                assert!(t1.alpha_eq(t2));
            }
        }
        for t1 in &set1 {
            for t2 in &set2 {
                assert!(!t1.alpha_eq(t2));
            }
        }
        assert!(!l2.alpha_eq(&l7));
    }

    #[test]
    fn test_value_step() {
        let l: Lam = bvar!("x");
        assert!(l.is_value().is_none());
        let l: Lam = blam!("x", bvar!("x"));
        assert!(l.is_value().is_some());

        let l: Lam = bapp!(blam!("x", bvar!("x")), blam!("y", blam!("z", bvar!("y"))));
        assert!(l.is_value().is_none());
        let l = l.step().unwrap();
        assert!(l.alpha_eq(&blam!("y", blam!("z", bvar!("y")))))
    }

    fn double() -> Lam {
        erec!(
            "f",
            "x",
            eif!(
                evar!("x"),
                ezero!(),
                esucc!(esucc!(eapp!(evar!("f"), epred!(evar!("x")))))
            )
        )
    }

    #[test]
    fn etest_variables() {
        let _: Lam = evar!("x");
        let _: Lam = eapp!(evar!("x"), evar!("x"));
        let _: Lam = ezero!();
        let _: Lam = esucc!(evar!("x"));
        let _: Lam = epred!(esucc!(ezero!()));
        let _: Lam = elet!("x", ezero!(), esucc!(evar!("x")));
        let _: Lam = erec!(
            "f",
            "x",
            eif!(
                evar!("x"),
                ezero!(),
                esucc!(esucc!(eapp!(evar!("f"), epred!(evar!("x")))))
            )
        );
    }

    #[test]
    fn etest_subst_alpha() {
        let l1: Lam = elam!("x", elam!("x", evar!("x")));
        let l2: Lam = elam!("x", elam!("x", evar!("y")));
        let l3: Lam = elam!("x", elam!("y", evar!("x")));
        let l4: Lam = elam!("x", elam!("y", evar!("y")));
        let l5: Lam = elam!("y", elam!("x", evar!("x")));
        let l6: Lam = elam!("y", elam!("x", evar!("y")));
        let l7: Lam = elam!("y", elam!("y", evar!("x")));
        let l8: Lam = elam!("y", elam!("y", evar!("y")));

        let set1 = vec![l1, l4, l5, l8];
        let set2 = vec![l3, l6];
        for t1 in &set1 {
            for t2 in &set1 {
                assert!(t1.alpha_eq(t2));
            }
        }
        for t1 in &set2 {
            for t2 in &set2 {
                assert!(t1.alpha_eq(t2));
            }
        }
        for t1 in &set1 {
            for t2 in &set2 {
                assert!(!t1.alpha_eq(t2));
            }
        }
        assert!(!l2.alpha_eq(&l7));
    }

    #[test]
    fn etest_value_step() {
        let l: Lam = double();
        let mut l: Lam = eapp!(l, esucc!(esucc!(ezero!())));
        while let Some(l1) = l.step() {
            println!("{}", print(&l1));
            l = l1;
        }
    }
}
