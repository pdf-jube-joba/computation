use std::collections::HashSet;

use crate::traits::*;
use utils::number::Number;
use utils::variable::Var;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Core<T> {
    Var { var: Var },
    Lam { var: Var, body: T },
    App { e1: T, e2: T },
    Zero,
    Succ { succ: T },
    Pred { pred: T },
    IfZ { cond: T, tcase: T, fcase: T },
    Let { var: Var, bind: T, body: T },
    Rec { fix: Var, var: Var, body: T },
}

impl<T> Core<T> {
    pub fn n_v(var: Var) -> Self {
        Core::Var { var }
    }
    pub fn n_l(var: Var, body: T) -> Self {
        Core::Lam { var, body }
    }
    pub fn n_a(e1: T, e2: T) -> Self {
        Core::App { e1, e2 }
    }
    pub fn n_z() -> Self {
        Core::Zero
    }
    pub fn n_s(succ: T) -> Self {
        Core::Succ { succ }
    }
    pub fn n_p(pred: T) -> Self {
        Core::Pred { pred }
    }
    pub fn n_i(cond: T, tcase: T, fcase: T) -> Self {
        Core::IfZ { cond, tcase, fcase }
    }
    pub fn n_d(var: Var, bind: T, body: T) -> Self {
        Core::Let { var, bind, body }
    }
    pub fn n_r(fix: Var, var: Var, body: T) -> Self {
        Core::Rec { fix, var, body }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtValue<T> {
    Fun { var: Var, body: T },
    Num(Number),
}

impl<T> From<ExtValue<T>> for Core<T>
where
    T: From<Core<T>>,
{
    fn from(value: ExtValue<T>) -> Self {
        fn num_to_exp<T>(n: Number) -> Core<T>
        where
            T: From<Core<T>>,
        {
            if n == 0.into() {
                Core::Zero
            } else {
                let succ: Core<T> = num_to_exp(n.pred());
                let succ: T = succ.into();
                Core::Succ { succ }
            }
        }
        match value {
            ExtValue::Fun { var, body } => Core::Lam { var, body },
            ExtValue::Num(n) => num_to_exp(n),
        }
    }
}

pub fn num_to_exp<T, F>(n: Number, f: F) -> Core<T>
where
    F: Fn(Core<T>) -> T + Clone,
{
    if n.is_zero() {
        Core::Zero
    } else {
        Core::Succ {
            succ: f(num_to_exp(n.pred(), f.clone())),
        }
    }
}

fn exp_to_num<T, F>(t: Core<T>, f: F) -> Option<Number>
where
    F: Fn(T) -> Option<Core<T>>,
{
    if let Core::Zero = t {
        Some(0.into())
    } else if let Core::Succ { succ } = t {
        Some(exp_to_num(f(succ)?, f)?.succ())
    } else {
        None
    }
}

pub fn ext_to_ext_value<T, F>(value: Core<T>, f: F) -> Option<ExtValue<T>>
where
    F: Fn(T) -> Option<Core<T>>,
{
    match value {
        Core::Lam { var, body } => Some(ExtValue::Fun { var, body }),
        _ => Some(ExtValue::Num(exp_to_num(value, f)?)),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtFrame<T> {
    EvalR { left_value: ExtValue<T> },
    EvalL { right_exp: T },
    EvalSucc,
    EvalPred,
    EvalIf { tcase: T, fcase: T },
    EvalLet { var: Var, body: T },
}

impl<T> ExtFrame<T>
where
    T: Clone + From<Core<T>>,
{
    pub fn decomp<F>(t: Core<T>, is_value: F) -> Option<(ExtFrame<T>, T)>
    where
        F: Fn(T) -> Option<ExtValue<T>> + Clone,
    {
        match t {
            Core::Var { var: _ } => None,
            Core::Lam { var: _, body: _ } => None,
            Core::App { e1, e2 } => match is_value(e1.clone()) {
                Some(value) => Some((ExtFrame::EvalR { left_value: value }, e2)),
                None => Some((ExtFrame::EvalL { right_exp: e2 }, e1)),
            },
            Core::Zero => None,
            Core::Succ { succ } => Some((ExtFrame::EvalSucc, succ)),
            Core::Pred { pred } => Some((ExtFrame::EvalPred, pred)),
            Core::IfZ { cond, tcase, fcase } => Some((ExtFrame::EvalIf { tcase, fcase }, cond)),
            Core::Let { var, bind, body } => Some((ExtFrame::EvalLet { var, body }, bind)),
            Core::Rec {
                fix: _,
                var: _,
                body: _,
            } => None,
        }
    }
    pub fn plug(self, t: T) -> Core<T> {
        match self {
            ExtFrame::EvalR { left_value } => {
                let v: Core<T> = left_value.into();
                Core::App {
                    e1: v.into(),
                    e2: t,
                }
            }
            ExtFrame::EvalL { right_exp } => Core::App {
                e1: t,
                e2: right_exp,
            },
            ExtFrame::EvalSucc => Core::Succ { succ: t },
            ExtFrame::EvalPred => Core::Pred { pred: t },
            ExtFrame::EvalIf { tcase, fcase } => Core::IfZ {
                cond: t,
                tcase,
                fcase,
            },
            ExtFrame::EvalLet { var, body } => Core::Let { var, bind: t, body },
        }
    }
}

impl<T> LambdaExt for Core<T>
where
    T: LambdaExt + Clone + PartialEq + From<Core<T>>,
{
    fn free_variables(&self) -> HashSet<Var> {
        let mut set = HashSet::default();
        match self {
            Core::Var { var } => {
                set.insert(var.clone());
            }
            Core::Lam { var, body } => {
                set.extend(body.free_variables());
                set.remove(var);
            }
            Core::App { e1, e2 } => {
                set.extend(e1.free_variables());
                set.extend(e2.free_variables());
            }
            Core::Zero => {}
            Core::Succ { succ } => set.extend(succ.free_variables()),
            Core::Pred { pred } => {
                set.extend(pred.free_variables());
            }
            Core::IfZ { cond, tcase, fcase } => {
                set.extend(cond.free_variables());
                set.extend(tcase.free_variables());
                set.extend(fcase.free_variables());
            }
            Core::Let { var, bind, body } => {
                set.extend(body.free_variables());
                set.remove(var);
                set.extend(bind.free_variables());
            }
            Core::Rec { fix, var, body } => {
                set.extend(body.free_variables());
                set.remove(fix);
                set.remove(var);
            }
        }
        set
    }
    fn bound_variables(&self) -> HashSet<Var> {
        let mut set = HashSet::default();
        match self {
            Core::Var { var: _ } => {}
            Core::Lam { var, body } => {
                set.extend(body.bound_variables());
                set.insert(var.clone());
            }
            Core::App { e1, e2 } => {
                set.extend(e1.bound_variables());
                set.extend(e2.bound_variables());
            }
            Core::Zero => {}
            Core::Succ { succ } => set.extend(succ.bound_variables()),
            Core::Pred { pred } => {
                set.extend(pred.bound_variables());
            }
            Core::IfZ { cond, tcase, fcase } => {
                set.extend(cond.bound_variables());
                set.extend(tcase.bound_variables());
                set.extend(fcase.bound_variables());
            }
            Core::Let { var, bind, body } => {
                set.extend(body.bound_variables());
                set.insert(var.clone());
                set.extend(bind.bound_variables());
            }
            Core::Rec { fix, var, body } => {
                set.extend(body.bound_variables());
                set.insert(fix.clone());
                set.insert(var.clone());
            }
        }
        set
    }
    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Core::Var { var: var1 }, Core::Var { var: var2 }) => var1 == var2,
            (Core::App { e1: m1, e2: m2 }, Core::App { e1: n1, e2: n2 }) => {
                m1.alpha_eq(n1) && m2.alpha_eq(n2)
            }
            (
                Core::Lam {
                    var: var1,
                    body: body1,
                },
                Core::Lam {
                    var: var2,
                    body: body2,
                },
            ) => {
                let new_var = Var::dummy();
                let body1 = body1
                    .clone()
                    .subst(var1.clone(), Core::n_v(new_var.clone()).into());
                let body2 = body2
                    .clone()
                    .subst(var2.clone(), Core::n_v(new_var.clone()).into());
                body1.alpha_eq(&body2)
            }
            (Core::Zero, Core::Zero) => true,
            (Core::Succ { succ: succ1 }, Core::Succ { succ: succ2 }) => succ1.alpha_eq(succ2),
            (Core::Pred { pred: pred1 }, Core::Pred { pred: pred2 }) => pred1.alpha_eq(pred2),
            (
                Core::IfZ {
                    cond: cond1,
                    tcase: tcase1,
                    fcase: fcase1,
                },
                Core::IfZ {
                    cond: cond2,
                    tcase: tcase2,
                    fcase: fcase2,
                },
            ) => cond1.alpha_eq(cond2) && tcase1.alpha_eq(tcase2) && fcase1.alpha_eq(fcase2),
            (
                Core::Let {
                    var: var1,
                    bind: bind1,
                    body: body1,
                },
                Core::Let {
                    var: var2,
                    bind: bind2,
                    body: body2,
                },
            ) => {
                body1.alpha_eq(body2)
                    && Core::Lam {
                        var: var1.clone(),
                        body: bind1.clone(),
                    }
                    .alpha_eq(&Core::Lam {
                        var: var2.clone(),
                        body: bind2.clone(),
                    })
                // {
                //     let mut set: VarSet = bind1.free_variables();
                //     set.extend(bind2.free_variables());
                //     let new_var = Ext::n_v(set.new_var_modify());
                //     let bind1 = bind1.clone().subst(var1.clone(), new_var.clone().into());
                //     let bind2 = bind2.clone().subst(var2.clone(), new_var.into());
                //     bind1.alpha_eq(&bind2)
                // }
            }
            (
                Core::Rec {
                    fix: fix1,
                    var: var1,
                    body: body1,
                },
                Core::Rec {
                    fix: fix2,
                    var: var2,
                    body: body2,
                },
            ) => {
                let new_var = Var::dummy();
                let new_fix = Var::dummy();
                let body1 = body1
                    .clone()
                    .subst(var1.clone(), Core::n_v(new_var.clone()).into())
                    .subst(fix1.clone(), Core::n_v(new_fix.clone()).into());
                let body2 = body2
                    .clone()
                    .subst(var2.clone(), Core::n_v(new_var.clone()).into())
                    .subst(fix2.clone(), Core::n_v(new_fix.clone()).into());
                body1.alpha_eq(&body2)
            }
            _ => false,
        }
    }
    fn subst(self, v: Var, t: Self) -> Self {
        match self {
            Core::Var { var } => {
                if var == v {
                    t
                } else {
                    Core::Var { var }
                }
            }
            Core::Lam { var, body } => {
                let new_var = Var::dummy();
                Core::n_l(
                    new_var.clone(),
                    body.subst(var, Core::n_v(new_var).into())
                        .subst(v, t.into()),
                )
            }
            Core::App { e1, e2 } => {
                Core::n_a(e1.subst(v.clone(), t.clone().into()), e2.subst(v, t.into()))
            }
            Core::Zero => Core::n_z(),
            Core::Succ { succ } => Core::n_s(succ.subst(v, t.into())),
            Core::Pred { pred } => Core::n_p(pred.subst(v, t.into())),
            Core::IfZ { cond, tcase, fcase } => Core::n_i(
                cond.subst(v.clone(), t.clone().into()),
                tcase.subst(v.clone(), t.clone().into()),
                fcase.subst(v, t.into()),
            ),
            Core::Let { var, bind, body } => {
                let new_var = Var::dummy();
                let bind = bind.subst(v.clone(), t.clone().into());
                let body = body
                    .subst(var, Core::n_v(new_var.clone()).into())
                    .subst(v, t.into());
                Core::n_d(new_var, bind, body)
            }
            Core::Rec { fix, var, body } => {
                let new_fix = Var::dummy();
                let new_var = Var::dummy();
                let new_body = body
                    .subst(fix.clone(), Core::n_v(new_fix.clone()).into())
                    .subst(var.clone(), Core::n_v(new_var.clone()).into())
                    .subst(v, t.into());
                Core::n_r(new_fix, new_var, new_body)
            }
        }
    }
}

impl<T> LamFamilySubst<T> for Core<T>
where
    T: LambdaExt + From<Core<T>> + Clone,
{
    /// (Ext<T>, Var, T) -> T
    fn subst_t(self, v: Var, t: T) -> T {
        match self {
            Core::Var { var } => {
                if var == v {
                    t
                } else {
                    Core::Var { var }.into()
                }
            }
            Core::Lam { var, body } => {
                let new_var = Var::dummy();
                Core::n_l(
                    new_var.clone(),
                    body.subst(var, Core::n_v(new_var).into()).subst(v, t),
                )
                .into()
            }
            Core::App { e1, e2 } => {
                Core::n_a(e1.subst(v.clone(), t.clone()), e2.subst(v, t)).into()
            }
            Core::Zero => Core::n_z().into(),
            Core::Succ { succ } => Core::n_s(succ.subst(v, t)).into(),
            Core::Pred { pred } => Core::n_p(pred.subst(v, t)).into(),
            Core::IfZ { cond, tcase, fcase } => Core::n_i(
                cond.subst(v.clone(), t.clone()),
                tcase.subst(v.clone(), t.clone()),
                fcase.subst(v, t),
            )
            .into(),
            Core::Let { var, bind, body } => {
                let new_var = Var::dummy();
                let bind = bind.subst(v.clone(), t.clone());
                let body = body
                    .subst(var, Core::n_v(new_var.clone()).into())
                    .subst(v, t);
                Core::n_d(new_var, bind, body).into()
            }
            Core::Rec { fix, var, body } => {
                let new_fix = Var::dummy();
                let new_var = Var::dummy();
                let new_body = body
                    .subst(fix.clone(), Core::n_v(new_fix.clone()).into())
                    .subst(var.clone(), Core::n_v(new_var.clone()).into())
                    .subst(v, t);
                Core::n_r(new_fix, new_var, new_body).into()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreStruct;

impl<T> LamFamily<T> for CoreStruct {
    type This = Core<T>;
}

#[macro_export]
macro_rules! bvar {
    ($name:literal) => {
        $crate::lambda::Core::n_v(utils::variable::Var::from($name)).into()
    };
}

#[macro_export]
macro_rules! blam {
    ($name:literal, $body:expr) => {
        $crate::lambda::Core::n_l(utils::variable::Var::from($name), $body).into()
    };
}

#[macro_export]
macro_rules! bapp {
    ($e1:expr, $e2:expr) => {
        $crate::lambda::Core::n_a($e1, $e2).into()
    };
}

#[macro_export]
macro_rules! evar {
    ($name:literal) => {
        $crate::lambda::Core::n_v(utils::variable::Var::from($name)).into()
    };
}

#[macro_export]
macro_rules! elam {
    ($name:literal, $body:expr) => {
        $crate::lambda::Core::n_l(utils::variable::Var::from($name), $body).into()
    };
}

#[macro_export]
macro_rules! eapp {
    ($e1:expr, $e2:expr) => {
        $crate::lambda::Core::n_a($e1, $e2).into()
    };
}

#[macro_export]
macro_rules! ezero {
    () => {
        $crate::lambda::Core::n_z().into()
    };
}

#[macro_export]
macro_rules! esucc {
    ($t:expr) => {
        $crate::lambda::Core::n_s($t).into()
    };
}

#[macro_export]
macro_rules! epred {
    ($t:expr) => {
        $crate::lambda::Core::n_p($t).into()
    };
}

#[macro_export]
macro_rules! eif {
    ($cond:expr, $tcase:expr, $fcase:expr) => {
        $crate::lambda::Core::n_i($cond, $tcase, $fcase).into()
    };
}

#[macro_export]
macro_rules! elet {
    ($name:literal, $bind:expr, $body:expr) => {
        $crate::lambda::Core::n_d(utils::variable::Var::from($name), $bind, $body).into()
    };
}

#[macro_export]
macro_rules! erec {
    ($fix:literal, $var:literal, $body:expr) => {
        $crate::lambda::Core::n_r(
            utils::variable::Var::from($fix),
            utils::variable::Var::from($var),
            $body,
        )
        .into()
    };
}

pub use {bapp, blam, bvar, eapp, eif, elam, elet, epred, erec, esucc, evar, ezero};
