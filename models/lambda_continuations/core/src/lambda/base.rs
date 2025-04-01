use crate::traits::*;
use utils::variable::{Var, VarSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Base<T> {
    Var { var: Var },
    Lam { var: Var, body: T },
    App { e1: T, e2: T },
}

impl<T> Base<T> {
    pub fn n_v(var: Var) -> Self {
        Base::Var { var }
    }
    pub fn n_l(var: Var, body: T) -> Self {
        Base::Lam { var, body }
    }
    pub fn n_a(e1: T, e2: T) -> Self {
        Base::App { e1, e2 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaseValue<T> {
    Fun { var: Var, body: T },
}

impl<T> BaseValue<T>
where
    T: LambdaExt,
{
    pub fn free_variables(&self) -> VarSet {
        match self {
            BaseValue::Fun { var, body } => {
                let mut set = body.free_variables();
                set.remove(var);
                set
            }
        }
    }
    pub fn bound_variables(&self) -> VarSet {
        match self {
            BaseValue::Fun { var, body } => {
                let mut set = body.bound_variables();
                set.insert(var);
                set
            }
        }
    }
}

impl<T> From<BaseValue<T>> for Base<T> {
    fn from(value: BaseValue<T>) -> Self {
        match value {
            BaseValue::Fun { var, body } => Base::Lam { var, body },
        }
    }
}

impl<T> TryFrom<Base<T>> for BaseValue<T> {
    type Error = ();
    fn try_from(value: Base<T>) -> Result<Self, Self::Error> {
        match value {
            Base::Var { var: _ } => Err(()),
            Base::Lam { var, body } => Ok(BaseValue::Fun { var, body }),
            Base::App { e1: _, e2: _ } => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseFrame<T> {
    EvalR { left_value: BaseValue<T> },
    EvalL { right_exp: T },
}

impl<T> BaseFrame<T>
where
    T: Clone + From<Base<T>>,
{
    pub fn decomp<F>(t: Base<T>, f: F) -> Option<(BaseFrame<T>, T)>
    where
        F: Fn(T) -> Option<BaseValue<T>>,
    {
        let Base::App { e1, e2 } = t else {
            return None;
        };
        match f(e1.clone()) {
            Some(value) => Some((BaseFrame::EvalR { left_value: value }, e2)),
            None => Some((BaseFrame::EvalL { right_exp: e2 }, e1)),
        }
    }
    pub fn plug(self, t: T) -> Base<T> {
        match self {
            BaseFrame::EvalR { left_value } => {
                let value: Base<T> = left_value.into();
                let value: T = value.into();
                Base::App { e1: value, e2: t }
            }
            BaseFrame::EvalL { right_exp } => Base::App {
                e1: right_exp,
                e2: t,
            },
        }
    }
}

impl<T> BaseFrame<T>
where
    T: LambdaExt,
{
    pub fn free_variables(&self) -> VarSet {
        match self {
            BaseFrame::EvalR { left_value } => left_value.free_variables(),
            BaseFrame::EvalL { right_exp } => right_exp.free_variables(),
        }
    }
    pub fn bound_variables(&self) -> VarSet {
        match self {
            BaseFrame::EvalR { left_value } => left_value.bound_variables(),
            BaseFrame::EvalL { right_exp } => right_exp.bound_variables(),
        }
    }
}

impl<T> LamFamilySubst<T> for Base<T>
where
    T: LambdaExt + From<Base<T>> + Clone + PartialEq,
{
    /// (Base<T>, Var, T) -> T
    fn subst_t(self, v: Var, t: T) -> T {
        match self {
            Base::Var { var } => {
                if var == v {
                    t
                } else {
                    Base::n_v(var).into()
                }
            }
            Base::Lam { var, body } => {
                let mut set: VarSet = t.free_variables();
                let new_var = set.new_var_modify();
                Base::n_l(
                    new_var.clone(),
                    body.subst(var, Base::n_v(new_var).into()).subst(v, t),
                )
                .into()
            }
            Base::App { e1, e2 } => {
                Base::n_a(e1.subst(v.clone(), t.clone()), e2.subst(v, t)).into()
            }
        }
    }
}

impl<T> LambdaExt for Base<T>
where
    T: LambdaExt + Clone + PartialEq + From<Base<T>>,
{
    fn free_variables(&self) -> VarSet {
        let mut set = VarSet::default();
        match self {
            Base::Var { var } => {
                set.insert(var.clone());
            }
            Base::Lam { var, body } => {
                set.extend(body.free_variables());
                set.remove(var);
            }
            Base::App { e1, e2 } => {
                set.extend(e1.free_variables());
                set.extend(e2.free_variables());
            }
        }
        set
    }
    fn bound_variables(&self) -> VarSet {
        let mut set = VarSet::default();
        match self {
            Base::Var { var: _ } => {}
            Base::Lam { var, body } => {
                set.extend(body.bound_variables());
                set.insert(var.clone());
            }
            Base::App { e1, e2 } => {
                set.extend(e1.bound_variables());
                set.extend(e2.bound_variables());
            }
        }
        set
    }
    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Base::Var { var: var1 }, Base::Var { var: var2 }) => var1 == var2,
            (Base::App { e1: m1, e2: m2 }, Base::App { e1: n1, e2: n2 }) => {
                m1.alpha_eq(n1) && m2.alpha_eq(n2)
            }
            (
                Base::Lam {
                    var: var1,
                    body: body1,
                },
                Base::Lam {
                    var: var2,
                    body: body2,
                },
            ) => {
                let mut set: VarSet = body1.free_variables();
                set.union(&body2.free_variables());
                let new_var = Base::n_v(set.new_var_modify());
                let body1 = body1.clone().subst(var1.clone(), new_var.clone().into());
                let body2 = body2.clone().subst(var2.clone(), new_var.into());
                body1.alpha_eq(&body2)
            }
            _ => false,
        }
    }
    fn subst(self, v: Var, t: Self) -> Self {
        match self {
            Base::Var { var } => {
                if var == v {
                    t
                } else {
                    Base::Var { var }
                }
            }
            Base::Lam { var, body } => {
                let mut set: VarSet = t.free_variables();
                let new_var = set.new_var_default(var.clone());
                Base::n_l(
                    new_var.clone(),
                    body.subst(var, Base::n_v(new_var).into())
                        .subst(v, t.into()),
                )
            }
            Base::App { e1, e2 } => {
                Base::n_a(e1.subst(v.clone(), t.clone().into()), e2.subst(v, t.into()))
            }
        }
    }
}

// higher kinded type のための型
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseStruct;
impl<T> LamFamily<T> for BaseStruct {
    type This = Base<T>;
}

#[macro_export]
macro_rules! bvar {
    ($str: literal) => {
        Base::n_v($str.into()).into()
    };
}

#[macro_export]
macro_rules! blam {
    ($str: literal, $t: expr) => {
        Base::n_l($str.into(), $t).into()
    };
}

#[macro_export]
macro_rules! bapp {
    ($t1: expr, $t2: expr) => {
        Base::n_a($t1, $t2).into()
    };
}

pub use {bapp, blam, bvar};
