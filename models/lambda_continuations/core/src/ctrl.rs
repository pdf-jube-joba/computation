use std::collections::HashSet;

use crate::{
    traits::{LambdaExt, Step},
};
use utils::{number::Number, variable::Var};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbCt {
    Var { var: Var },
    Lam { var: Var, body: Box<AbCt> },
    App { e1: Box<AbCt>, e2: Box<AbCt> },
    Zero,
    Succ { succ: Box<AbCt> },
    Pred { pred: Box<AbCt> },
    IfZ { cond: Box<AbCt>, tcase: Box<AbCt>, fcase: Box<AbCt> },
    Let { var: Var, bind: Box<AbCt>, body: Box<AbCt> },
    Rec { fix: Var, var: Var, body: Box<AbCt> },
    Abort(Box<AbCt>),
    Control(Box<AbCt>),
}

impl AbCt {
    pub fn n_v(var: Var) -> Self {
        AbCt::Var { var }
    }
    pub fn n_l(var: Var, body: AbCt) -> Self {
        AbCt::Lam {
            var,
            body: Box::new(body),
        }
    }
    pub fn n_a(e1: AbCt, e2: AbCt) -> Self {
        AbCt::App {
            e1: Box::new(e1),
            e2: Box::new(e2),
        }
    }
    pub fn n_z() -> Self {
        AbCt::Zero
    }
    pub fn n_s(succ: AbCt) -> Self {
        AbCt::Succ {
            succ: Box::new(succ),
        }
    }
    pub fn n_p(pred: AbCt) -> Self {
        AbCt::Pred {
            pred: Box::new(pred),
        }
    }
    pub fn n_i(cond: AbCt, tcase: AbCt, fcase: AbCt) -> Self {
        AbCt::IfZ {
            cond: Box::new(cond),
            tcase: Box::new(tcase),
            fcase: Box::new(fcase),
        }
    }
    pub fn n_d(var: Var, bind: AbCt, body: AbCt) -> Self {
        AbCt::Let {
            var,
            bind: Box::new(bind),
            body: Box::new(body),
        }
    }
    pub fn n_r(fix: Var, var: Var, body: AbCt) -> Self {
        AbCt::Rec {
            fix,
            var,
            body: Box::new(body),
        }
    }
    pub fn abort(term: AbCt) -> Self {
        AbCt::Abort(Box::new(term))
    }
    pub fn control(term: AbCt) -> Self {
        AbCt::Control(Box::new(term))
    }
}

fn same_var(a: &Var, b: &Var) -> bool {
    a.as_str() == b.as_str()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbCtValue {
    Fun { var: Var, body: AbCt },
    Num(Number),
}

impl From<AbCtValue> for AbCt {
    fn from(value: AbCtValue) -> Self {
        match value {
            AbCtValue::Fun { var, body } => AbCt::n_l(var, body),
            AbCtValue::Num(n) => num_to_abct(n),
        }
    }
}

fn num_to_abct(n: Number) -> AbCt {
    if n.is_zero() {
        AbCt::n_z()
    } else {
        AbCt::n_s(num_to_abct(n.pred()))
    }
}

fn abct_to_value(term: AbCt) -> Option<AbCtValue> {
    match term {
        AbCt::Lam { var, body } => Some(AbCtValue::Fun {
            var,
            body: *body,
        }),
        other => Some(AbCtValue::Num(exp_to_num(other)?)),
    }
}

fn exp_to_num(term: AbCt) -> Option<Number> {
    match term {
        AbCt::Zero => Some(0.into()),
        AbCt::Succ { succ } => exp_to_num(*succ).map(|n| n.succ()),
        _ => None,
    }
}

impl LambdaExt for AbCt {
    fn free_variables(&self) -> HashSet<String> {
        match self {
            AbCt::Var { var } => {
                let mut set = HashSet::new();
                set.insert(var.as_str().to_string());
                set
            }
            AbCt::Lam { var, body } => {
                let mut set = body.free_variables();
                set.remove(var.as_str());
                set
            }
            AbCt::App { e1, e2 } => {
                let mut set = e1.free_variables();
                set.extend(e2.free_variables());
                set
            }
            AbCt::Zero => HashSet::new(),
            AbCt::Succ { succ } => succ.free_variables(),
            AbCt::Pred { pred } => pred.free_variables(),
            AbCt::IfZ { cond, tcase, fcase } => {
                let mut set = cond.free_variables();
                set.extend(tcase.free_variables());
                set.extend(fcase.free_variables());
                set
            }
            AbCt::Let { var, bind, body } => {
                let mut set = body.free_variables();
                set.remove(var.as_str());
                set.extend(bind.free_variables());
                set
            }
            AbCt::Rec { fix, var, body } => {
                let mut set = body.free_variables();
                set.remove(fix.as_str());
                set.remove(var.as_str());
                set
            }
            AbCt::Abort(term) => term.free_variables(),
            AbCt::Control(term) => term.free_variables(),
        }
    }

    fn bound_variables(&self) -> HashSet<String> {
        match self {
            AbCt::Var { .. } => HashSet::new(),
            AbCt::Lam { var, body } => {
                let mut set = body.bound_variables();
                set.insert(var.as_str().to_string());
                set
            }
            AbCt::App { e1, e2 } => {
                let mut set = e1.bound_variables();
                set.extend(e2.bound_variables());
                set
            }
            AbCt::Zero => HashSet::new(),
            AbCt::Succ { succ } => succ.bound_variables(),
            AbCt::Pred { pred } => pred.bound_variables(),
            AbCt::IfZ { cond, tcase, fcase } => {
                let mut set = cond.bound_variables();
                set.extend(tcase.bound_variables());
                set.extend(fcase.bound_variables());
                set
            }
            AbCt::Let { var, bind, body } => {
                let mut set = body.bound_variables();
                set.insert(var.as_str().to_string());
                set.extend(bind.bound_variables());
                set
            }
            AbCt::Rec { fix, var, body } => {
                let mut set = body.bound_variables();
                set.insert(fix.as_str().to_string());
                set.insert(var.as_str().to_string());
                set
            }
            AbCt::Abort(term) => term.bound_variables(),
            AbCt::Control(term) => term.bound_variables(),
        }
    }

    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AbCt::Var { var: v1 }, AbCt::Var { var: v2 }) => same_var(v1, v2),
            (AbCt::App { e1: l1, e2: l2 }, AbCt::App { e1: r1, e2: r2 }) => {
                l1.alpha_eq(r1) && l2.alpha_eq(r2)
            }
            (AbCt::Lam { var: v1, body: b1 }, AbCt::Lam { var: v2, body: b2 }) => {
                let new_var = Var::dummy();
                let body1 = b1.as_ref().clone().subst(v1.clone(), AbCt::n_v(new_var.clone()));
                let body2 = b2.as_ref().clone().subst(v2.clone(), AbCt::n_v(new_var));
                body1.alpha_eq(&body2)
            }
            (AbCt::Zero, AbCt::Zero) => true,
            (AbCt::Succ { succ: l }, AbCt::Succ { succ: r }) => l.alpha_eq(r),
            (AbCt::Pred { pred: l }, AbCt::Pred { pred: r }) => l.alpha_eq(r),
            (
                AbCt::IfZ {
                    cond: c1,
                    tcase: t1,
                    fcase: f1,
                },
                AbCt::IfZ {
                    cond: c2,
                    tcase: t2,
                    fcase: f2,
                },
            ) => c1.alpha_eq(c2) && t1.alpha_eq(t2) && f1.alpha_eq(f2),
            (
                AbCt::Let {
                    var: v1,
                    bind: b1,
                    body: body1,
                },
                AbCt::Let {
                    var: v2,
                    bind: b2,
                    body: body2,
                },
            ) => {
                body1.alpha_eq(body2)
                    && AbCt::n_l(v1.clone(), b1.as_ref().clone())
                        .alpha_eq(&AbCt::n_l(v2.clone(), b2.as_ref().clone()))
            }
            (
                AbCt::Rec {
                    fix: f1,
                    var: v1,
                    body: b1,
                },
                AbCt::Rec {
                    fix: f2,
                    var: v2,
                    body: b2,
                },
            ) => {
                let new_var = Var::dummy();
                let new_fix = Var::dummy();
                let body1 = b1
                    .as_ref()
                    .clone()
                    .subst(v1.clone(), AbCt::n_v(new_var.clone()))
                    .subst(f1.clone(), AbCt::n_v(new_fix.clone()));
                let body2 = b2
                    .as_ref()
                    .clone()
                    .subst(v2.clone(), AbCt::n_v(new_var))
                    .subst(f2.clone(), AbCt::n_v(new_fix));
                body1.alpha_eq(&body2)
            }
            (AbCt::Abort(t1), AbCt::Abort(t2)) => t1.alpha_eq(t2),
            (AbCt::Control(t1), AbCt::Control(t2)) => t1.alpha_eq(t2),
            _ => false,
        }
    }

    fn subst(self, v: Var, t: Self) -> Self {
        match self {
            AbCt::Var { var } => {
                if same_var(&var, &v) {
                    t
                } else {
                    AbCt::Var { var }
                }
            }
            AbCt::Lam { var, body } => {
                let new_var = Var::dummy();
                AbCt::n_l(
                    new_var.clone(),
                    body.subst(var, AbCt::n_v(new_var)).subst(v, t),
                )
            }
            AbCt::App { e1, e2 } => AbCt::n_a(
                e1.subst(v.clone(), t.clone()),
                e2.subst(v, t),
            ),
            AbCt::Zero => AbCt::n_z(),
            AbCt::Succ { succ } => AbCt::n_s(succ.subst(v, t)),
            AbCt::Pred { pred } => AbCt::n_p(pred.subst(v, t)),
            AbCt::IfZ { cond, tcase, fcase } => AbCt::n_i(
                cond.subst(v.clone(), t.clone()),
                tcase.subst(v.clone(), t.clone()),
                fcase.subst(v, t),
            ),
            AbCt::Let { var, bind, body } => {
                let new_var = Var::dummy();
                let bind = bind.subst(v.clone(), t.clone());
                let body = body
                    .subst(var, AbCt::n_v(new_var.clone()))
                    .subst(v, t);
                AbCt::n_d(new_var, bind, body)
            }
            AbCt::Rec { fix, var, body } => {
                let new_fix = Var::dummy();
                let new_var = Var::dummy();
                let new_body = body
                    .subst(fix.clone(), AbCt::n_v(new_fix.clone()))
                    .subst(var.clone(), AbCt::n_v(new_var.clone()))
                    .subst(v, t);
                AbCt::n_r(new_fix, new_var, new_body)
            }
            AbCt::Abort(term) => AbCt::abort(term.subst(v, t)),
            AbCt::Control(term) => AbCt::control(term.subst(v, t)),
        }
    }
}

impl Step for AbCt {
    type Value = AbCtValue;
    fn is_value(&self) -> Option<Self::Value> {
        abct_to_value(self.clone())
    }
    fn step(self) -> Option<Self> {
        // To be implemented: call/cc style semantics.
        let _ = self;
        todo!("abort/control semantics are not implemented yet");
    }
}
