use std::fmt::Debug;

use crate::{
    lambda::{
        base::{Base, BaseStruct},
        ext::{Ext, ExtStruct},
    },
    traits::{LamFamily, LamFamilySubst, LambdaExt},
};
use utils::variable::Var;

pub enum Lam<T>
where
    T: LamFamily<Lam<T>>,
{
    Base(Box<T::This>),
}

// 以下、個別に実装しないと認識してくれなかった部分（#[derive(Clone, PartialEq)] が通用しない。）
impl Clone for Lam<BaseStruct> {
    fn clone(&self) -> Self {
        let Lam::Base(b) = self;
        Lam::Base(b.clone())
    }
}

// 以下、個別に実装しないと認識してくれなかった部分（#[derive(Clone, PartialEq)] が通用しない。）
impl Clone for Lam<ExtStruct> {
    fn clone(&self) -> Self {
        let Lam::Base(b) = self;
        Lam::Base(b.clone())
    }
}

impl PartialEq for Lam<BaseStruct> {
    fn eq(&self, other: &Self) -> bool {
        let Lam::Base(b1) = self;
        let Lam::Base(b2) = other;
        b1 == b2
    }
}

impl PartialEq for Lam<ExtStruct> {
    fn eq(&self, other: &Self) -> bool {
        let Lam::Base(b1) = self;
        let Lam::Base(b2) = other;
        b1 == b2
    }
}

impl<T> Debug for Lam<T>
where
    T: LamFamily<Lam<T>>,
    T::This: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Lam::Base(b) = self;
        write!(f, "{:?}", b)
    }
}

// impl Debug for Lam<BaseStruct> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     }
// }

// impl Debug for Lam<ExtStruct> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let Lam::Base(b) = self;
//         write!(f, "{:?}", b)
//     }
// }

impl From<Base<Lam<BaseStruct>>> for Lam<BaseStruct> {
    fn from(value: Base<Lam<BaseStruct>>) -> Self {
        Lam::Base(Box::new(value))
    }
}

impl From<Ext<Lam<ExtStruct>>> for Lam<ExtStruct> {
    fn from(value: Ext<Lam<ExtStruct>>) -> Self {
        Lam::Base(Box::new(value))
    }
}

impl LambdaExt for Lam<BaseStruct> {
    fn free_variables(&self) -> std::collections::HashSet<Var> {
        match self {
            Lam::Base(b) => b.as_ref().free_variables(),
        }
    }
    fn bound_variables(&self) -> std::collections::HashSet<Var> {
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

impl LambdaExt for Lam<ExtStruct> {
    fn free_variables(&self) -> std::collections::HashSet<Var> {
        match self {
            Lam::Base(b) => b.as_ref().free_variables(),
        }
    }
    fn bound_variables(&self) -> std::collections::HashSet<Var> {
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

#[cfg(test)]
mod tests {
    use crate::lambda::base::{bapp, blam, bvar, Base, BaseStruct};

    use super::*;
    #[test]
    fn test1() {
        let l: Lam<BaseStruct> = bvar!("x");
        assert_eq!(l.free_variables(), vec!["x".into()].into_iter().collect());
        assert_eq!(l.bound_variables(), vec![].into_iter().collect());

        let l: Lam<BaseStruct> = blam!("x", bvar!("x"));
        assert_eq!(l.free_variables(), vec![].into_iter().collect());
        assert_eq!(l.bound_variables(), vec!["x".into()].into_iter().collect());

        let l: Lam<BaseStruct> = blam!("x", bvar!("y"));
        assert_eq!(l.free_variables(), vec!["y".into()].into_iter().collect());
        assert_eq!(l.bound_variables(), vec!["x".into()].into_iter().collect());

        let l: Lam<BaseStruct> = bapp!(bvar!("x"), blam!("x", bvar!("z")));
        assert_eq!(
            l.free_variables(),
            vec!["x".into(), "z".into()].into_iter().collect()
        );
        assert_eq!(l.bound_variables(), vec!["x".into()].into_iter().collect());

        let l1: Lam<BaseStruct> = blam!("x", blam!("x", bvar!("x")));
        let l2: Lam<BaseStruct> = blam!("x", blam!("x", bvar!("y")));
        let l3: Lam<BaseStruct> = blam!("x", blam!("y", bvar!("x")));
        let l4: Lam<BaseStruct> = blam!("x", blam!("y", bvar!("y")));
        let l5: Lam<BaseStruct> = blam!("y", blam!("x", bvar!("x")));
        let l6: Lam<BaseStruct> = blam!("y", blam!("x", bvar!("y")));
        let l7: Lam<BaseStruct> = blam!("y", blam!("y", bvar!("x")));
        let l8: Lam<BaseStruct> = blam!("y", blam!("y", bvar!("y")));

        let set1 = vec![l1, l4, l5, l8]; // \x.\x.x = \x.\y.y = \y.\x.x = \y.\y.y
        let set2 = vec![l3, l6]; // \x.y.x = \y.\x.y
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
                println!("{t1:?} {t2:?}");
                assert!(!t1.alpha_eq(t2));
            }
        }

        assert!(!l2.alpha_eq(&l7));

        for t in &set1 {
            assert!(!l2.alpha_eq(t))
        }

        for t in &set2 {
            assert!(!l2.alpha_eq(t))
        }

        for t in &set1 {
            assert!(!l7.alpha_eq(t))
        }

        for t in &set2 {
            assert!(!l7.alpha_eq(t))
        }
    }
}
