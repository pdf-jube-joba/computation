use std::collections::HashSet;

use crate::traits::LambdaExt;
use utils::{number::Number, variable::Var};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var {
        var: Var,
    },
    Lam {
        var: Var,
        body: Box<Lam>,
    },
    App {
        e1: Box<Lam>,
        e2: Box<Lam>,
    },
    Zero,
    Succ {
        succ: Box<Lam>,
    },
    Pred {
        pred: Box<Lam>,
    },
    IfZ {
        cond: Box<Lam>,
        tcase: Box<Lam>,
        fcase: Box<Lam>,
    },
    Let {
        var: Var,
        bind: Box<Lam>,
        body: Box<Lam>,
    },
    Rec {
        fix: Var,
        var: Var,
        body: Box<Lam>,
    },
}

impl Lam {
    pub fn n_v(var: Var) -> Self {
        Lam::Var { var }
    }
    pub fn n_l(var: Var, body: Lam) -> Self {
        Lam::Lam {
            var,
            body: Box::new(body),
        }
    }
    pub fn n_a(e1: Lam, e2: Lam) -> Self {
        Lam::App {
            e1: Box::new(e1),
            e2: Box::new(e2),
        }
    }
    pub fn n_z() -> Self {
        Lam::Zero
    }
    pub fn n_s(succ: Lam) -> Self {
        Lam::Succ {
            succ: Box::new(succ),
        }
    }
    pub fn n_p(pred: Lam) -> Self {
        Lam::Pred {
            pred: Box::new(pred),
        }
    }
    pub fn n_i(cond: Lam, tcase: Lam, fcase: Lam) -> Self {
        Lam::IfZ {
            cond: Box::new(cond),
            tcase: Box::new(tcase),
            fcase: Box::new(fcase),
        }
    }
    pub fn n_d(var: Var, bind: Lam, body: Lam) -> Self {
        Lam::Let {
            var,
            bind: Box::new(bind),
            body: Box::new(body),
        }
    }
    pub fn n_r(fix: Var, var: Var, body: Lam) -> Self {
        Lam::Rec {
            fix,
            var,
            body: Box::new(body),
        }
    }
}

fn same_var(a: &Var, b: &Var) -> bool {
    a.as_str() == b.as_str()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LamValue {
    Fun { var: Var, body: Lam },
    Num(Number),
}

impl From<LamValue> for Lam {
    fn from(value: LamValue) -> Self {
        match value {
            LamValue::Fun { var, body } => Lam::n_l(var, body),
            LamValue::Num(n) => num_to_exp(n),
        }
    }
}

pub fn num_to_exp(n: Number) -> Lam {
    if n.is_zero() {
        Lam::n_z()
    } else {
        Lam::n_s(num_to_exp(n.pred()))
    }
}

fn exp_to_num(t: Lam) -> Option<Number> {
    match t {
        Lam::Zero => Some(0.into()),
        Lam::Succ { succ } => exp_to_num(*succ).map(|n| n.succ()),
        _ => None,
    }
}

pub fn lam_to_value(value: Lam) -> Option<LamValue> {
    match value {
        Lam::Lam { var, body } => Some(LamValue::Fun { var, body: *body }),
        other => Some(LamValue::Num(exp_to_num(other)?)),
    }
}

impl LambdaExt for Lam {
    fn free_variables(&self) -> HashSet<String> {
        match self {
            Lam::Var { var } => {
                let mut set = HashSet::new();
                set.insert(var.as_str().to_string());
                set
            }
            Lam::Lam { var, body } => {
                let mut set = body.free_variables();
                set.remove(var.as_str());
                set
            }
            Lam::App { e1, e2 } => {
                let mut set = e1.free_variables();
                set.extend(e2.free_variables());
                set
            }
            Lam::Zero => HashSet::new(),
            Lam::Succ { succ } => succ.free_variables(),
            Lam::Pred { pred } => pred.free_variables(),
            Lam::IfZ { cond, tcase, fcase } => {
                let mut set = cond.free_variables();
                set.extend(tcase.free_variables());
                set.extend(fcase.free_variables());
                set
            }
            Lam::Let { var, bind, body } => {
                let mut set = body.free_variables();
                set.remove(var.as_str());
                set.extend(bind.free_variables());
                set
            }
            Lam::Rec { fix, var, body } => {
                let mut set = body.free_variables();
                set.remove(fix.as_str());
                set.remove(var.as_str());
                set
            }
        }
    }

    fn bound_variables(&self) -> HashSet<String> {
        match self {
            Lam::Var { .. } => HashSet::new(),
            Lam::Lam { var, body } => {
                let mut set = body.bound_variables();
                set.insert(var.as_str().to_string());
                set
            }
            Lam::App { e1, e2 } => {
                let mut set = e1.bound_variables();
                set.extend(e2.bound_variables());
                set
            }
            Lam::Zero => HashSet::new(),
            Lam::Succ { succ } => succ.bound_variables(),
            Lam::Pred { pred } => pred.bound_variables(),
            Lam::IfZ { cond, tcase, fcase } => {
                let mut set = cond.bound_variables();
                set.extend(tcase.bound_variables());
                set.extend(fcase.bound_variables());
                set
            }
            Lam::Let { var, bind, body } => {
                let mut set = body.bound_variables();
                set.insert(var.as_str().to_string());
                set.extend(bind.bound_variables());
                set
            }
            Lam::Rec { fix, var, body } => {
                let mut set = body.bound_variables();
                set.insert(fix.as_str().to_string());
                set.insert(var.as_str().to_string());
                set
            }
        }
    }

    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Lam::Var { var: v1 }, Lam::Var { var: v2 }) => same_var(v1, v2),
            (Lam::App { e1: l1, e2: l2 }, Lam::App { e1: r1, e2: r2 }) => {
                l1.alpha_eq(r1) && l2.alpha_eq(r2)
            }
            (Lam::Lam { var: v1, body: b1 }, Lam::Lam { var: v2, body: b2 }) => {
                let new_var = Var::dummy();
                let body1 = b1
                    .as_ref()
                    .clone()
                    .subst(v1.clone(), Lam::n_v(new_var.clone()));
                let body2 = b2.as_ref().clone().subst(v2.clone(), Lam::n_v(new_var));
                body1.alpha_eq(&body2)
            }
            (Lam::Zero, Lam::Zero) => true,
            (Lam::Succ { succ: l }, Lam::Succ { succ: r }) => l.alpha_eq(r),
            (Lam::Pred { pred: l }, Lam::Pred { pred: r }) => l.alpha_eq(r),
            (
                Lam::IfZ {
                    cond: c1,
                    tcase: t1,
                    fcase: f1,
                },
                Lam::IfZ {
                    cond: c2,
                    tcase: t2,
                    fcase: f2,
                },
            ) => c1.alpha_eq(c2) && t1.alpha_eq(t2) && f1.alpha_eq(f2),
            (
                Lam::Let {
                    var: v1,
                    bind: b1,
                    body: body1,
                },
                Lam::Let {
                    var: v2,
                    bind: b2,
                    body: body2,
                },
            ) => {
                body1.alpha_eq(body2)
                    && Lam::n_l(v1.clone(), b1.as_ref().clone())
                        .alpha_eq(&Lam::n_l(v2.clone(), b2.as_ref().clone()))
            }
            (
                Lam::Rec {
                    fix: f1,
                    var: v1,
                    body: b1,
                },
                Lam::Rec {
                    fix: f2,
                    var: v2,
                    body: b2,
                },
            ) => {
                let new_var = Var::dummy();
                let new_fix = Var::dummy();
                let body1 = b1
                    .clone()
                    .subst(v1.clone(), Lam::n_v(new_var.clone()))
                    .subst(f1.clone(), Lam::n_v(new_fix.clone()));
                let body2 = b2
                    .clone()
                    .subst(v2.clone(), Lam::n_v(new_var))
                    .subst(f2.clone(), Lam::n_v(new_fix));
                body1.alpha_eq(&body2)
            }
            _ => false,
        }
    }

    fn subst(self, v: Var, t: Self) -> Self {
        match self {
            Lam::Var { var } => {
                if same_var(&var, &v) {
                    t
                } else {
                    Lam::Var { var }
                }
            }
            Lam::Lam { var, body } => {
                let new_var = Var::dummy();
                let body = *body;
                Lam::n_l(
                    new_var.clone(),
                    body.subst(var, Lam::n_v(new_var)).subst(v, t),
                )
            }
            Lam::App { e1, e2 } => Lam::n_a((*e1).subst(v.clone(), t.clone()), (*e2).subst(v, t)),
            Lam::Zero => Lam::n_z(),
            Lam::Succ { succ } => Lam::n_s((*succ).subst(v, t)),
            Lam::Pred { pred } => Lam::n_p((*pred).subst(v, t)),
            Lam::IfZ { cond, tcase, fcase } => Lam::n_i(
                (*cond).subst(v.clone(), t.clone()),
                (*tcase).subst(v.clone(), t.clone()),
                (*fcase).subst(v, t),
            ),
            Lam::Let { var, bind, body } => {
                let new_var = Var::dummy();
                let bind = (*bind).subst(v.clone(), t.clone());
                let body = body.subst(var, Lam::n_v(new_var.clone())).subst(v, t);
                Lam::n_d(new_var, bind, body)
            }
            Lam::Rec { fix, var, body } => {
                let new_fix = Var::dummy();
                let new_var = Var::dummy();
                let new_body = body
                    .subst(fix.clone(), Lam::n_v(new_fix.clone()))
                    .subst(var.clone(), Lam::n_v(new_var.clone()))
                    .subst(v, t);
                Lam::n_r(new_fix, new_var, new_body)
            }
        }
    }
}

#[macro_export]
macro_rules! bvar {
    ($name:literal) => {
        $crate::lambda::Lam::n_v(::utils::variable::Var::from($name))
    };
}

#[macro_export]
macro_rules! blam {
    ($name:literal, $body:expr) => {
        $crate::lambda::Lam::n_l(::utils::variable::Var::from($name), $body)
    };
}

#[macro_export]
macro_rules! bapp {
    ($e1:expr, $e2:expr) => {
        $crate::lambda::Lam::n_a($e1, $e2)
    };
}

#[macro_export]
macro_rules! ezero {
    () => {
        $crate::lambda::Lam::n_z()
    };
}

#[macro_export]
macro_rules! evar {
    ($name:literal) => {
        $crate::lambda::Lam::n_v(::utils::variable::Var::from($name))
    };
}

#[macro_export]
macro_rules! elam {
    ($name:literal, $body:expr) => {
        $crate::lambda::Lam::n_l(::utils::variable::Var::from($name), $body)
    };
}

#[macro_export]
macro_rules! eapp {
    ($e1:expr, $e2:expr) => {
        $crate::lambda::Lam::n_a($e1, $e2)
    };
}

#[macro_export]
macro_rules! esucc {
    ($t:expr) => {
        $crate::lambda::Lam::n_s($t)
    };
}

#[macro_export]
macro_rules! epred {
    ($t:expr) => {
        $crate::lambda::Lam::n_p($t)
    };
}

#[macro_export]
macro_rules! eif {
    ($cond:expr, $tcase:expr, $fcase:expr) => {
        $crate::lambda::Lam::n_i($cond, $tcase, $fcase)
    };
}

#[macro_export]
macro_rules! elet {
    ($name:literal, $bind:expr, $body:expr) => {
        $crate::lambda::Lam::n_d(::utils::variable::Var::from($name), $bind, $body)
    };
}

#[macro_export]
macro_rules! erec {
    ($fix:literal, $var:literal, $body:expr) => {
        $crate::lambda::Lam::n_r(
            ::utils::variable::Var::from($fix),
            ::utils::variable::Var::from($var),
            $body,
        )
    };
}

pub use {bapp, blam, bvar, eapp, eif, elam, elet, epred, erec, esucc, evar, ezero};
