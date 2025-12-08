use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use crate::{
    lambda::{ext_to_ext_value, num_to_exp, Core, CoreStruct, ExtValue},
    traits::{LamFamily, LamFamilySubst, LambdaExt, Step},
};
use utils::variable::Var;

pub enum Lam<T>
where
    T: LamFamily<Lam<T>>,
{
    Base(Box<T::This>),
}

impl LambdaExt for Lam<CoreStruct> {
    fn free_variables(&self) -> HashSet<Var> {
        match self {
            Lam::Base(b) => b.as_ref().free_variables(),
        }
    }
    fn bound_variables(&self) -> HashSet<Var> {
        match self {
            Lam::Base(b) => b.bound_variables(),
        }
    }
    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Lam::Base(b1), Lam::Base(b2)) => b1.alpha_eq(b2),
        }
    }
    fn subst(self, v: Var, t: Self) -> Self {
        match self {
            Lam::Base(b) => b.subst_t(v, t),
        }
    }
}

impl Step for Lam<CoreStruct> {
    type Value = ExtValue<Lam<CoreStruct>>;
    fn is_value(&self) -> Option<Self::Value> {
        let Lam::Base(b) = self;
        ext_to_ext_value(b.as_ref().clone(), t_to_core)
    }
    fn step(self) -> Option<Self> {
        let Lam::Base(b) = self;
        match *b {
            Core::Var { .. } => None,
            Core::Lam { .. } => None,
            Core::App { e1, e2 } => match (e1.is_value(), e2.is_value()) {
                (Some(ExtValue::Fun { var, body }), Some(_)) => Some(body.subst(var, e2)),
                (Some(ExtValue::Num(_)), Some(_)) => None,
                (Some(_), None) => Some(Core::n_a(e1, e2.step()?).into()),
                (None, _) => Some(Core::n_a(e1.step()?, e2).into()),
            },
            Core::Zero => None,
            Core::Succ { succ } => {
                if succ.is_value().is_none() {
                    Some(Lam::Base(Box::new(Core::Succ { succ: succ.step()? })))
                } else {
                    None
                }
            }
            Core::Pred { pred } => {
                if let Some(v) = pred.is_value() {
                    match v {
                        ExtValue::Fun { .. } => None,
                        ExtValue::Num(number) => {
                            fn wrap(t: Core<Lam<CoreStruct>>) -> Lam<CoreStruct> {
                                t.into()
                            }
                            let e = num_to_exp(number.pred(), wrap);
                            Some(e.into())
                        }
                    }
                } else {
                    Some(Core::Pred { pred: pred.step()? }.into())
                }
            }
            Core::IfZ { cond, tcase, fcase } => {
                if let Some(v) = cond.is_value() {
                    match v {
                        ExtValue::Fun { .. } => None,
                        ExtValue::Num(number) => {
                            if number.is_zero() {
                                Some(tcase)
                            } else {
                                Some(fcase)
                            }
                        }
                    }
                } else {
                    Some(
                        Core::IfZ {
                            cond: cond.step()?,
                            tcase,
                            fcase,
                        }
                        .into(),
                    )
                }
            }
            Core::Let { var, bind, body } => Some(Core::n_a(Core::n_l(var, body).into(), bind).into()),
            Core::Rec { fix, var, body } => Some(
                Core::n_l(
                    var.clone(),
                    body.clone()
                        .subst(fix.clone(), Core::Rec { fix, var, body }.into()),
                )
                .into(),
            ),
        }
    }
}

fn print(t: &Lam<CoreStruct>) -> String {
    let Lam::Base(b) = t;
    match b.as_ref() {
        Core::Var { var } => format!("{var}"),
        Core::Lam { var, body } => format!("fun {var} => {}", print(body)),
        Core::App { e1, e2 } => format!("({} {})", print(e1), print(e2)),
        Core::Zero => "0".to_string(),
        Core::Succ { succ } => format!("S {}", print(succ)),
        Core::Pred { pred } => format!("P {}", print(pred)),
        Core::IfZ { cond, tcase, fcase } => format!(
            "if {} then {} else {}",
            print(cond),
            print(tcase),
            print(fcase)
        ),
        Core::Let { var, bind, body } => format!("let {var} = {} in \n {}", print(bind), print(body)),
        Core::Rec { fix, var, body } => format!("rec {fix} {var} = {}", print(body)),
    }
}

fn t_to_core(value: Lam<CoreStruct>) -> Option<Core<Lam<CoreStruct>>> {
    match value {
        Lam::Base(b) => Some(*b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{bapp, blam, bvar, eapp, eif, elam, elet, epred, erec, esucc, evar, ezero};

    #[test]
    fn test_variables() {
        let l: Lam<CoreStruct> = bvar!("x");
        assert_eq!(l.free_variables(), vec!["x".into()].into_iter().collect());
        assert_eq!(l.bound_variables(), HashSet::new());

        let l: Lam<CoreStruct> = blam!("x", bvar!("x"));
        assert_eq!(l.free_variables(), HashSet::new());
        assert_eq!(l.bound_variables(), vec!["x".into()].into_iter().collect());

        let l: Lam<CoreStruct> = blam!("x", bvar!("y"));
        assert_eq!(l.free_variables(), vec!["y".into()].into_iter().collect());
        assert_eq!(l.bound_variables(), vec!["x".into()].into_iter().collect());

        let l: Lam<CoreStruct> = bapp!(bvar!("x"), blam!("x", bvar!("z")));
        assert_eq!(
            l.free_variables(),
            vec!["x".into(), "z".into()].into_iter().collect()
        );
        assert_eq!(l.bound_variables(), vec!["x".into()].into_iter().collect());
    }

    #[test]
    fn test_subst_alpha() {
        let l1: Lam<CoreStruct> = blam!("x", blam!("x", bvar!("x")));
        let l2: Lam<CoreStruct> = blam!("x", blam!("x", bvar!("y")));
        let l3: Lam<CoreStruct> = blam!("x", blam!("y", bvar!("x")));
        let l4: Lam<CoreStruct> = blam!("x", blam!("y", bvar!("y")));
        let l5: Lam<CoreStruct> = blam!("y", blam!("x", bvar!("x")));
        let l6: Lam<CoreStruct> = blam!("y", blam!("x", bvar!("y")));
        let l7: Lam<CoreStruct> = blam!("y", blam!("y", bvar!("x")));
        let l8: Lam<CoreStruct> = blam!("y", blam!("y", bvar!("y")));

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
        let l: Lam<_> = bvar!("x");
        assert!(l.is_value().is_none());
        let l: Lam<_> = blam!("x", bvar!("x"));
        assert!(l.is_value().is_some());

        let l: Lam<_> = bapp!(blam!("x", bvar!("x")), blam!("y", blam!("z", bvar!("y"))));
        assert!(l.is_value().is_none());
        let l = l.step().unwrap();
        assert!(l.alpha_eq(&blam!("y", blam!("z", bvar!("y")))))
    }

    fn double() -> Lam<CoreStruct> {
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
        let _: Lam<_> = evar!("x");
        let _: Lam<_> = eapp!(evar!("x"), evar!("x"));
        let _: Lam<_> = ezero!();
        let _: Lam<_> = esucc!(evar!("x"));
        let _: Lam<_> = epred!(esucc!(ezero!()));
        let _: Lam<_> = elet!("x", ezero!(), esucc!(evar!("x")));
        let _: Lam<_> = erec!(
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
        let l1: Lam<CoreStruct> = elam!("x", elam!("x", evar!("x")));
        let l2: Lam<CoreStruct> = elam!("x", elam!("x", evar!("y")));
        let l3: Lam<CoreStruct> = elam!("x", elam!("y", evar!("x")));
        let l4: Lam<CoreStruct> = elam!("x", elam!("y", evar!("y")));
        let l5: Lam<CoreStruct> = elam!("y", elam!("x", evar!("x")));
        let l6: Lam<CoreStruct> = elam!("y", elam!("x", evar!("y")));
        let l7: Lam<CoreStruct> = elam!("y", elam!("y", evar!("x")));
        let l8: Lam<CoreStruct> = elam!("y", elam!("y", evar!("y")));

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
        let l: Lam<_> = double();
        let mut l: Lam<_> = eapp!(l, esucc!(esucc!(ezero!())));
        while let Some(l1) = l.step() {
            println!("{}", print(&l1));
            l = l1;
        }
    }
}

mod traits {
    use super::*;

    impl Clone for Lam<CoreStruct> {
        fn clone(&self) -> Self {
            let Lam::Base(b) = self;
            Lam::Base(b.clone())
        }
    }

    impl PartialEq for Lam<CoreStruct> {
        fn eq(&self, other: &Self) -> bool {
            let Lam::Base(b1) = self;
            let Lam::Base(b2) = other;
            b1 == b2
        }
    }

    impl Debug for Lam<CoreStruct> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let Lam::Base(b) = self;
            write!(f, "{:?}", b)
        }
    }

    impl Display for Lam<CoreStruct> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let Lam::Base(b) = self;
            write!(f, "{:?}", b)
        }
    }

    impl From<Core<Lam<CoreStruct>>> for Lam<CoreStruct> {
        fn from(value: Core<Lam<CoreStruct>>) -> Self {
            Lam::Base(Box::new(value))
        }
    }
}
