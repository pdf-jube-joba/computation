use crate::{
    lambda::{
        base::{Base, BaseStruct},
        ext::{Ext, ExtStruct},
    },
    traits::{LamFamily, LamFamilySubst, LambdaExt},
};
use utils::variable::Var;

#[derive(Debug, Clone, PartialEq)]
enum AbCt<T>
where
    T: LamFamily<AbCt<T>>,
{
    Base(Box<T::This>),
    Abort(Box<AbCt<T>>),
    Control(Box<AbCt<T>>),
}

impl From<Base<AbCt<BaseStruct>>> for AbCt<BaseStruct> {
    fn from(value: Base<AbCt<BaseStruct>>) -> Self {
        AbCt::Base(Box::new(value))
    }
}

impl From<Ext<AbCt<ExtStruct>>> for AbCt<ExtStruct> {
    fn from(value: Ext<AbCt<ExtStruct>>) -> Self {
        AbCt::Base(Box::new(value))
    }
}

impl<T> AbCt<T>
where
    T: LamFamily<AbCt<T>>,
{
    pub fn n(t: T::This) -> Self {
        AbCt::Base(Box::new(t))
    }
    pub fn ab(t: AbCt<T>) -> Self {
        AbCt::Abort(Box::new(t))
    }
    pub fn ct(t: AbCt<T>) -> Self {
        AbCt::Control(Box::new(t))
    }
}

impl<T> LambdaExt for AbCt<T>
where
    T: LamFamily<AbCt<T>>,
    T::This: LambdaExt + LamFamilySubst<AbCt<T>>,
{
    fn free_variables(&self) -> std::collections::HashSet<Var> {
        match self {
            AbCt::Base(b) => b.free_variables(),
            AbCt::Abort(t) => t.free_variables(),
            AbCt::Control(t) => t.free_variables(),
        }
    }
    fn bound_variables(&self) -> std::collections::HashSet<Var> {
        match self {
            AbCt::Base(b) => b.bound_variables(),
            AbCt::Abort(t) => t.bound_variables(),
            AbCt::Control(t) => t.bound_variables(),
        }
    }
    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AbCt::Base(b1), AbCt::Base(b2)) => b1.alpha_eq(b2),
            (AbCt::Abort(t1), AbCt::Abort(t2)) => t1.alpha_eq(t2),
            (AbCt::Control(t1), AbCt::Control(t2)) => t1.alpha_eq(t2),
            _ => false,
        }
    }
    fn subst(self, v: Var, t: Self) -> Self {
        match self {
            AbCt::Base(b) => AbCt::ab(b.subst_t(v, t)),
            AbCt::Abort(e) => AbCt::ab(e.subst(v, t)),
            AbCt::Control(e) => AbCt::ct(e.subst(v, t)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{base, ext};

    #[test]
    fn t() {
        let v: Var = "test".into();
        let l: AbCt<base::BaseStruct> = AbCt::Base(Box::new(base::Base::n_v(v.clone())));
        let l: AbCt<ext::ExtStruct> = AbCt::n(ext::Ext::n_v(v));
    }
}
