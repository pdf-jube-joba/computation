use crate::{LambdaContext, LambdaExt, State};
use std::{collections::HashSet, fmt::Display};
use utils::variable::{self, Var};

use super::*;
use utils::number::Number;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
    Zero,
    Succ(Box<Lam>),
    Pred(Box<Lam>),
    IfZ(Box<Lam>, Box<Lam>, Box<Lam>),
    Let(Var, Box<Lam>, Box<Lam>),
    Rec(Var, Var, Box<Lam>),
}

impl Display for Lam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Lam::Var(x) => format!("{x}"),
            Lam::Lam(x, e) => format!("\\{x}. {e}"),
            Lam::App(e1, e2) => format!("({e1} @ {e2})"),
            Lam::Zero => "0".to_string(),
            Lam::Succ(e) => format!("succ {e}"),
            Lam::Pred(e) => format!("pred {e}"),
            Lam::IfZ(e1, e2, e3) => format!("if {e1} then {e2} else {e3}"),
            Lam::Let(x, e1, e2) => format!("let {x} = {e1} in {e2}"),
            Lam::Rec(f, x, e) => format!("rec {f} {x} = {e}"),
        };
        write!(f, "{string}")
    }
}

fn num_to_exp(e: Number) -> Lam {
    if e.is_zero() {
        Lam::Zero
    } else {
        Lam::Succ(Box::new(num_to_exp(e - 1)))
    }
}

fn exp_to_num(e: &Lam) -> Option<Number> {
    match e {
        Lam::Zero => Some(0.into()),
        Lam::Succ(e) => Some(exp_to_num(e)? + 1),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Num(Number),
    Function(Var, Lam),
}

impl SubSet for Value {
    type Super = Lam;
    fn from_super(s: &Self::Super) -> Option<Self> {
        if let Some(n) = exp_to_num(s) {
            Some(Value::Num(n))
        } else if let Lam::Lam(x, e) = s {
            Some(Value::Function(x.clone(), e.as_ref().clone()))
        } else {
            None
        }
    }
    fn into_super(self) -> Self::Super {
        match self {
            Value::Num(n) => num_to_exp(n),
            Value::Function(x, e) => Lam::Lam(x, Box::new(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RedexInfo {
    App {
        var: Var,
        exp: Lam,
        value: Value,
    },
    Let {
        var: Var,
        value_of_var: Value,
        exp: Lam,
    },
    Rec {
        fix_var: Var,
        var: Var,
        exp: Lam,
    },
    Pred {
        n: Number,
    },
    IfZ {
        n: Number,
        e1: Lam,
        e2: Lam,
    },
}

impl SubSet for RedexInfo {
    type Super = Lam;
    fn from_super(s: &Self::Super) -> Option<Self> {
        match s {
            Lam::App(e1, e2) => {
                let Value::Function(x, e) = Value::from_super(e1)? else {
                    return None;
                };
                let e2 = Value::from_super(e2)?;
                Some(RedexInfo::App {
                    var: x,
                    exp: e,
                    value: e2,
                })
            }
            Lam::Let(x, e1, e2) => {
                let e1 = Value::from_super(e1)?;
                Some(RedexInfo::Let {
                    var: x.clone(),
                    value_of_var: e1,
                    exp: e2.as_ref().clone(),
                })
            }
            Lam::Rec(f, x, e) => Some(RedexInfo::Rec {
                fix_var: f.clone(),
                var: x.clone(),
                exp: e.as_ref().clone(),
            }),
            Lam::Pred(e) => {
                let n = exp_to_num(e)?;
                Some(RedexInfo::Pred { n })
            }
            Lam::Var(_) => todo!(),
            Lam::Lam(_, _) => todo!(),
            Lam::Zero => todo!(),
            Lam::Succ(_) => todo!(),
            Lam::IfZ(e, e1, e2) => Some(RedexInfo::IfZ {
                n: exp_to_num(e)?,
                e1: e1.as_ref().clone(),
                e2: e2.as_ref().clone(),
            }),
        }
    }
    fn into_super(self) -> Self::Super {
        match self {
            RedexInfo::App { var, exp, value } => Lam::App(
                Box::new(Lam::Lam(var, Box::new(exp))),
                Box::new(value.into_super()),
            ),
            RedexInfo::Let {
                var,
                value_of_var,
                exp,
            } => Lam::Let(var, Box::new(value_of_var.into_super()), Box::new(exp)),
            RedexInfo::Rec { fix_var, var, exp } => Lam::Rec(fix_var, var, Box::new(exp)),
            RedexInfo::Pred { n } => Lam::Pred(Box::new(num_to_exp(n))),
            RedexInfo::IfZ { n, e1, e2 } => {
                Lam::IfZ(Box::new(num_to_exp(n)), Box::new(e1), Box::new(e2))
            }
        }
    }
}

impl LambdaExt for Lam {
    type Value = Value;
    type RedexInfo = RedexInfo;
    fn free_variables(&self) -> HashSet<Var> {
        let mut set = HashSet::new();
        match self {
            Lam::Var(x) => {
                set.insert(x.clone());
            }
            Lam::Lam(x, e) => {
                set.extend(e.free_variables());
                set.remove(x);
            }
            Lam::App(e1, e2) => {
                set.extend(e1.free_variables());
                set.extend(e2.free_variables());
            }
            Lam::Zero => {}
            Lam::Succ(e1) => {
                set.extend(e1.free_variables());
            }
            Lam::Pred(e1) => {
                set.extend(e1.free_variables());
            }
            Lam::IfZ(e1, e2, e3) => {
                set.extend(e1.free_variables());
                set.extend(e2.free_variables());
                set.extend(e3.free_variables());
            }
            Lam::Let(x, e1, e2) => {
                set.extend(e1.free_variables());
                set.extend(e2.free_variables());
                set.remove(x);
            }
            Lam::Rec(f, x, e) => {
                set.extend(e.free_variables());
                set.remove(x);
                set.remove(f);
            }
        }
        set
    }
    fn bound_variables(&self) -> HashSet<Var> {
        let mut set = HashSet::new();
        match self {
            Lam::Var(_) => {}
            Lam::Lam(x, e) => {
                set.extend(e.bound_variables());
                set.insert(x.clone());
            }
            Lam::App(e1, e2) => {
                set.extend(e1.bound_variables());
                set.extend(e2.bound_variables());
            }
            Lam::Zero => {}
            Lam::Succ(e1) => {
                set.extend(e1.bound_variables());
            }
            Lam::Pred(e1) => {
                set.extend(e1.bound_variables());
            }
            Lam::IfZ(e1, e2, e3) => {
                set.extend(e1.bound_variables());
                set.extend(e2.bound_variables());
                set.extend(e3.bound_variables());
            }
            Lam::Let(x, e1, e2) => {
                set.extend(e1.bound_variables());
                set.extend(e2.bound_variables());
                set.insert(x.clone());
            }
            Lam::Rec(f, x, e) => {
                set.extend(e.bound_variables());
                set.insert(x.clone());
                set.insert(f.clone());
            }
        }
        set
    }
    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
        fn alpha_conversion_canonical_rec(e: Lam, mut vs: variable::VarMap) -> Lam {
            match e {
                Lam::Var(x) => Lam::Var(vs.get_table(&x)),
                Lam::Lam(x, e) => {
                    vs.push_var(&x);
                    let new_x = vs.get_table(&x);
                    Lam::Lam(new_x, Box::new(alpha_conversion_canonical_rec(*e, vs)))
                }
                Lam::App(e1, e2) => Lam::App(
                    Box::new(alpha_conversion_canonical_rec(*e1, vs.clone())),
                    Box::new(alpha_conversion_canonical_rec(*e2, vs)),
                ),
                Lam::Zero => Lam::Zero,
                Lam::Succ(e) => Lam::Succ(Box::new(alpha_conversion_canonical_rec(*e, vs))),
                Lam::Pred(e) => Lam::Pred(Box::new(alpha_conversion_canonical_rec(*e, vs))),
                Lam::IfZ(e1, e2, e3) => Lam::IfZ(
                    Box::new(alpha_conversion_canonical_rec(*e1, vs.clone())),
                    Box::new(alpha_conversion_canonical_rec(*e2, vs.clone())),
                    Box::new(alpha_conversion_canonical_rec(*e3, vs)),
                ),
                Lam::Let(x, e1, e2) => {
                    let new_e1 = alpha_conversion_canonical_rec(*e1, vs.clone());
                    let new_x = vs.get_table(&x);
                    let new_e2 = alpha_conversion_canonical_rec(*e2, vs);
                    Lam::Let(new_x, Box::new(new_e1), Box::new(new_e2))
                }
                Lam::Rec(f, x, e) => {
                    vs.push_var(&f);
                    vs.push_var(&x);
                    let new_f = vs.get_table(&f);
                    let new_x = vs.get_table(&x);
                    let new_e = alpha_conversion_canonical_rec(*e, vs);
                    Lam::Rec(new_f, new_x, Box::new(new_e))
                }
            }
        }
        let vs = variable::VarMap::new_iter(self.free_variables().into_iter().chain(vs));
        alpha_conversion_canonical_rec(self, vs)
    }

    fn subst(self, x: Var, t: Self) -> Self {
        pub fn simple_subst(e: Lam, x: Var, t: Lam) -> Lam {
            match e {
                Lam::Var(y) => {
                    if x == y {
                        t
                    } else {
                        Lam::Var(y)
                    }
                }
                Lam::Lam(y, e) => {
                    if x == y {
                        Lam::Lam(y, e)
                    } else {
                        Lam::Lam(y, Box::new(simple_subst(*e, x, t)))
                    }
                }
                Lam::App(e1, e2) => Lam::App(
                    Box::new(simple_subst(*e1, x.clone(), t.clone())),
                    Box::new(simple_subst(*e2, x, t)),
                ),
                Lam::Zero => todo!(),
                Lam::Succ(_) => todo!(),
                Lam::Pred(_) => todo!(),
                Lam::IfZ(_, _, _) => todo!(),
                Lam::Let(_, _, _) => todo!(),
                Lam::Rec(_, _, _) => todo!(),
            }
        }
        let free_t = t.free_variables();
        let e = self.alpha_conversion_canonical(free_t);
        simple_subst(e, x, t)
    }

    fn redex_step(r: Self::RedexInfo) -> Self {
        match r {
            RedexInfo::App { var, exp, value } => exp.subst(var, value.into_super()),
            RedexInfo::Let {
                var,
                value_of_var,
                exp,
            } => exp.subst(var, value_of_var.into_super()),
            RedexInfo::Rec { fix_var, var, exp } => {
                let same_exp = RedexInfo::Rec {
                    fix_var: fix_var.clone(),
                    var: var.clone(),
                    exp: exp.clone(),
                }
                .into_super();
                let rec_m = exp.subst(fix_var.clone(), same_exp);
                Lam::Lam(var, Box::new(rec_m))
            }
            RedexInfo::Pred { n } => num_to_exp(n.pred()),
            RedexInfo::IfZ { n, e1, e2 } => {
                if n.is_zero() {
                    e1
                } else {
                    e2
                }
            }
        }
    }

    fn step(self) -> Option<Self> {
        if let Some(redex) = RedexInfo::from_super(&self) {
            return Some(Lam::redex_step(redex));
        }
        let lam: Lam = match self {
            Lam::Var(_) | Lam::Lam(_, _) | Lam::Zero | Lam::Rec(_, _, _) => todo!(),
            Lam::App(e1, e2) => {
                if Value::from_super(&e1).is_some() {
                    Lam::App(Box::new(e1.step()?), e2)
                } else {
                    Lam::App(e1, Box::new(e2.step()?))
                }
            }
            Lam::Succ(e) => Lam::Succ(Box::new(e.step()?)),
            Lam::Pred(e) => Lam::Pred(Box::new(e.step()?)),
            Lam::IfZ(e, e1, e2) => Lam::IfZ(Box::new(e.step()?), e1, e2),
            Lam::Let(x, e1, e2) => Lam::Let(x, Box::new(e1.step()?), e2),
        };
        Some(lam)
    }
}

// Var(Var),
// Lam(Var, Box<Lam>),
// App(Box<Lam>, Box<Lam>),
// Zero,
// Succ(Box<Lam>),
// Pred(Box<Lam>),
// IfZ(Box<Lam>, Box<Lam>, Box<Lam>),
// Let(Var, Box<Lam>, Box<Lam>),
// Rec(Var, Var, Box<Lam>),

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Frame {
    EvalL(Lam),
    EvalR(Value),
    Succ,
    Pred,
    Ifz(Lam, Lam),
    Let(Var, Lam),
}

impl LambdaContext for Lam {
    type Frame = Frame;
    fn decomp(e: Self) -> Option<(Self::Frame, Self)> {
        match e {
            Lam::Var(_) | Lam::Lam(_, _) | Lam::Zero | Lam::Rec(_, _, _) => None,
            Lam::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    Some((Frame::EvalR(v), *e2))
                } else {
                    Some((Frame::EvalL(*e2), *e1))
                }
            }
            Lam::Succ(e) => Some((Frame::Succ, *e)),
            Lam::Pred(e) => Some((Frame::Pred, *e)),
            Lam::IfZ(e, e1, e2) => Some((Frame::Ifz(*e1, *e2), *e)),
            Lam::Let(x, e, e1) => Some((Frame::Let(x, *e1), *e)),
        }
    }
    fn plug(frame: Self::Frame, e: Self) -> Self {
        match frame {
            Frame::EvalR(v) => Lam::App(Box::new(v.into_super()), Box::new(e)),
            Frame::EvalL(e1) => Lam::App(Box::new(e), Box::new(e1)),
            Frame::Ifz(e1, e2) => Lam::IfZ(Box::new(e), Box::new(e1), Box::new(e2)),
            Frame::Let(x, e1) => Lam::Let(x, Box::new(e), Box::new(e1)),
            Frame::Succ => Lam::Succ(Box::new(e)),
            Frame::Pred => Lam::Pred(Box::new(e)),
        }
    }
    fn step_state(State { mut stack, top }: State<Self>) -> Option<State<Self>> {
        if Value::from_super(&top).is_some() {
            if let Some(frame) = stack.pop() {
                let new_lam = Lam::plug(frame, top);
                Some(State {
                    stack,
                    top: new_lam,
                })
            } else {
                None
            }
        } else if let Some(redexinfo) = RedexInfo::from_super(&top) {
            Some(State {
                stack,
                top: Lam::redex_step(redexinfo),
            })
        } else {
            let (frame, e) = Lam::decomp(top)?;
            stack.push(frame);
            Some(State { stack, top: e })
        }
    }
}
