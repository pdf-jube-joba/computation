use std::collections::HashSet;

use crate::{
    lambda::{ext_to_ext_value, Core, CoreStruct, ExtFrame, ExtValue},
    traits::{LamFamily, LamFamilySubst, LambdaExt, Step},
};
use utils::variable::Var;

pub enum AbCt<T>
where
    T: LamFamily<AbCt<T>>,
{
    Base(Box<T::This>),
    Abort(Box<AbCt<T>>),
    Control(Box<AbCt<T>>),
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

impl LambdaExt for AbCt<CoreStruct> {
    fn free_variables(&self) -> HashSet<Var> {
        match self {
            AbCt::Base(b) => b.free_variables(),
            AbCt::Abort(t) => t.free_variables(),
            AbCt::Control(t) => t.free_variables(),
        }
    }
    fn bound_variables(&self) -> HashSet<Var> {
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

impl Step for AbCt<CoreStruct> {
    type Value = ExtValue<AbCt<CoreStruct>>;
    fn is_value(&self) -> Option<Self::Value> {
        match self {
            AbCt::Base(b) => ext_to_ext_value(b.as_ref().clone(), t_to_core),
            AbCt::Abort(e) => None,
            AbCt::Control(e) => None,
        }
    }
    fn step(self) -> Option<Self> {
        let mut stack: Vec<ExtFrame<AbCt<CoreStruct>>> = vec![];
        let mut t = self;
        loop {
            match t {
                AbCt::Base(b) => {
                    todo!()
                }
                AbCt::Abort(e) => {
                    return Some(*e);
                }
                AbCt::Control(e) => {
                    let cont = {
                        let new_var = Var::dummy();
                        let mut t = Core::n_v(new_var.clone());
                        while let Some(frame) = stack.pop() {
                            t = frame.plug(t.into());
                        }
                        Core::n_l(new_var, AbCt::n(t)).into()
                    };
                    return Some(Core::n_a(*e, cont).into());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{bapp, blam, bvar, eapp, eif, elam, elet, epred, erec, esucc, evar, ezero};

    #[test]
    fn t() {
        let l1: AbCt<_> = bvar!("x");
        let l2: AbCt<_> = evar!("x");
        let s1 = l1.free_variables();
        let s2 = l2.free_variables();
    }
}

mod traits {
    use crate::lambda::{Core, CoreStruct};

    use super::AbCt;

    impl Clone for AbCt<CoreStruct> {
        fn clone(&self) -> Self {
            match self {
                AbCt::Base(b) => AbCt::Base(b.clone()),
                AbCt::Abort(e) => AbCt::Abort(e.clone()),
                AbCt::Control(e) => AbCt::Control(e.clone()),
            }
        }
    }

    impl PartialEq for AbCt<CoreStruct> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (AbCt::Base(b1), AbCt::Base(b2)) => b1 == b2,
                (AbCt::Abort(e1), AbCt::Abort(e2)) => e1 == e2,
                (AbCt::Control(e1), AbCt::Control(e2)) => e1 == e2,
                _ => false,
            }
        }
    }

    impl From<Core<AbCt<CoreStruct>>> for AbCt<CoreStruct> {
        fn from(value: Core<AbCt<CoreStruct>>) -> Self {
            AbCt::Base(Box::new(value))
        }
    }
}

fn t_to_core(value: AbCt<CoreStruct>) -> Option<Core<AbCt<CoreStruct>>> {
    match value {
        AbCt::Base(b) => Some(*b),
        _ => None,
    }
}
