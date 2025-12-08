use crate::{
    lambda::{
        base::{Base, BaseFrame, BaseStruct, BaseValue},
        ext::{Core, ExtStruct},
    },
    traits::{LamFamily, LamFamilySubst, LambdaExt, Step},
};
use utils::variable::{Var, VarSet};

enum AbCt<T>
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

struct AbCtBaseFrame(BaseFrame<AbCt<BaseStruct>>);

impl LambdaExt for AbCt<BaseStruct> {
    fn free_variables(&self) -> VarSet {
        match self {
            AbCt::Base(b) => b.free_variables(),
            AbCt::Abort(t) => t.free_variables(),
            AbCt::Control(t) => t.free_variables(),
        }
    }
    fn bound_variables(&self) -> VarSet {
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

impl Step for AbCt<BaseStruct> {
    type Value = BaseValue<AbCt<BaseStruct>>;
    fn is_value(&self) -> Option<Self::Value> {
        match self {
            AbCt::Base(b) => {
                if let Ok(b) = b.as_ref().clone().try_into() {
                    Some(b)
                } else {
                    None
                }
            }
            AbCt::Abort(e) => None,
            AbCt::Control(e) => None,
        }
    }
    fn step(self) -> Option<Self> {
        let mut stack: Vec<BaseFrame<AbCt<BaseStruct>>> = vec![];
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
                        let new_var: Var = {
                            let mut setvar = VarSet::default();
                            for frame in &stack {
                                setvar.extend(frame.free_variables());
                                setvar.extend(frame.bound_variables());
                            }
                            setvar.new_var_modify()
                        };
                        let mut t = Base::n_v(new_var.clone());
                        while let Some(frame) = stack.pop() {
                            t = frame.plug(t.into());
                        }
                        Base::n_l(new_var, AbCt::n(t)).into()
                    };
                    return Some(Base::n_a(*e, cont).into());
                }
            }
        }
    }
}

impl LambdaExt for AbCt<ExtStruct> {
    fn free_variables(&self) -> VarSet {
        match self {
            AbCt::Base(b) => b.free_variables(),
            AbCt::Abort(t) => t.free_variables(),
            AbCt::Control(t) => t.free_variables(),
        }
    }
    fn bound_variables(&self) -> VarSet {
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
    use crate::lambda::{
        base::{bapp, blam, bvar},
        ext::{eapp, eif, elam, elet, epred, erec, esucc, evar, ezero},
    };

    #[test]
    fn t() {
        let l1: AbCt<_> = bvar!("x");
        let l2: AbCt<_> = evar!("x");
        let s1 = l1.free_variables();
        let s2 = l2.free_variables();
    }
}

mod traits {
    use crate::lambda::{
        base::{Base, BaseStruct},
        ext::{Core, ExtStruct},
    };

    use super::AbCt;

    impl Clone for AbCt<BaseStruct> {
        fn clone(&self) -> Self {
            match self {
                AbCt::Base(b) => AbCt::Base(b.clone()),
                AbCt::Abort(e) => AbCt::Abort(e.clone()),
                AbCt::Control(e) => AbCt::Control(e.clone()),
            }
        }
    }

    impl Clone for AbCt<ExtStruct> {
        fn clone(&self) -> Self {
            match self {
                AbCt::Base(b) => AbCt::Base(b.clone()),
                AbCt::Abort(e) => AbCt::Abort(e.clone()),
                AbCt::Control(e) => AbCt::Control(e.clone()),
            }
        }
    }

    impl PartialEq for AbCt<BaseStruct> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (AbCt::Base(b1), AbCt::Base(b2)) => b1 == b2,
                (AbCt::Abort(e1), AbCt::Abort(e2)) => e1 == e2,
                (AbCt::Control(e1), AbCt::Control(e2)) => e1 == e2,
                _ => false,
            }
        }
    }

    impl PartialEq for AbCt<ExtStruct> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (AbCt::Base(b1), AbCt::Base(b2)) => b1 == b2,
                (AbCt::Abort(e1), AbCt::Abort(e2)) => e1 == e2,
                (AbCt::Control(e1), AbCt::Control(e2)) => e1 == e2,
                _ => false,
            }
        }
    }

    impl From<Base<AbCt<BaseStruct>>> for AbCt<BaseStruct> {
        fn from(value: Base<AbCt<BaseStruct>>) -> Self {
            AbCt::Base(Box::new(value))
        }
    }

    impl From<Core<AbCt<ExtStruct>>> for AbCt<ExtStruct> {
        fn from(value: Core<AbCt<ExtStruct>>) -> Self {
            AbCt::Base(Box::new(value))
        }
    }
}
