use std::{collections::HashSet, fmt::Display};

use utils::{
    set::SubSet,
    variable::{self, Var},
};

use crate::LambdaExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LamGrabDelim {
    Var(Var),
    Lam(Var, Box<LamGrabDelim>),
    App(Box<LamGrabDelim>, Box<LamGrabDelim>),
    Delim(Box<LamGrabDelim>),
    Grab(Var, Box<LamGrabDelim>),
}

impl LamGrabDelim {
    pub fn v<T>(n: T) -> LamGrabDelim
    where
        T: Into<Var>,
    {
        LamGrabDelim::Var(n.into())
    }
    pub fn l<T>(n: T, e: LamGrabDelim) -> LamGrabDelim
    where
        T: Into<Var>,
    {
        LamGrabDelim::Lam(n.into(), Box::new(e))
    }
    pub fn a(e1: LamGrabDelim, e2: LamGrabDelim) -> LamGrabDelim {
        LamGrabDelim::App(Box::new(e1), Box::new(e2))
    }
    pub fn d(e: LamGrabDelim) -> LamGrabDelim {
        LamGrabDelim::Delim(Box::new(e))
    }
    pub fn g<T>(k: T, e: LamGrabDelim) -> LamGrabDelim
    where
        T: Into<Var>,
    {
        LamGrabDelim::Grab(k.into(), Box::new(e))
    }
}

impl Display for LamGrabDelim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            LamGrabDelim::Var(var) => format!("{var}"),
            LamGrabDelim::Lam(var, term) => format!("\\{var}.{term}"),
            LamGrabDelim::App(term1, term2) => format!("({term1} @ {term2})"),
            LamGrabDelim::Delim(term) => format!("delim {term}"),
            LamGrabDelim::Grab(k, term) => format!("grab {k}. {term}"),
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Function(Var, Box<LamGrabDelim>),
}

impl SubSet for Value {
    type Super = LamGrabDelim;
    fn from_super(s: &Self::Super) -> Option<Self> {
        match s {
            LamGrabDelim::Lam(v, e) => Some(Value::Function(v.clone(), e.clone())),
            _ => None,
        }
    }
    fn into_super(self) -> Self::Super {
        let Value::Function(x, e) = self;
        LamGrabDelim::Lam(x, e)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Value::Function(x, e) = self;
        write!(f, "{{\\ {}. {}}}", x, e)
    }
}

pub enum GrabPureCxt {
    Hole,                                  // []
    EvalL(LamGrabDelim, Box<GrabPureCxt>), // E[[] e]
    EvalR(Value, Box<GrabPureCxt>),        // E[v []]
}

impl GrabPureCxt {
    pub fn plug(self, t: LamGrabDelim) -> LamGrabDelim {
        match self {
            GrabPureCxt::Hole => t,
            GrabPureCxt::EvalL(e, cxt) => cxt.plug(LamGrabDelim::a(t, e)),
            GrabPureCxt::EvalR(v, cxt) => cxt.plug(LamGrabDelim::a(v.into_super(), t)),
        }
    }
    pub fn extend_r(self, v: Value) -> Self {
        match self {
            GrabPureCxt::Hole => GrabPureCxt::EvalR(v, Box::new(GrabPureCxt::Hole)),
            GrabPureCxt::EvalL(e1, c) => GrabPureCxt::EvalL(e1, Box::new(c.extend_r(v))),
            GrabPureCxt::EvalR(e1, c) => GrabPureCxt::EvalR(e1, Box::new(c.extend_r(v))),
        }
    }
    pub fn extend_l(self, e: LamGrabDelim) -> Self {
        match self {
            GrabPureCxt::Hole => GrabPureCxt::EvalL(e, Box::new(GrabPureCxt::Hole)),
            GrabPureCxt::EvalL(e1, c) => GrabPureCxt::EvalL(e1, Box::new(c.extend_l(e))),
            GrabPureCxt::EvalR(e1, c) => GrabPureCxt::EvalR(e1, Box::new(c.extend_l(e))),
        }
    }
}

pub enum GrabCxt {
    Hole,
    EvalL(LamGrabDelim, Box<GrabCxt>), // E[[] e]
    EvalR(Value, Box<GrabCxt>),        // E[v []]
    Del(Box<GrabCxt>),                 // E[delimit []] ,
}

impl GrabCxt {
    pub fn plug(self, t: LamGrabDelim) -> LamGrabDelim {
        match self {
            GrabCxt::Hole => t,
            GrabCxt::EvalL(e, cxt) => cxt.plug(LamGrabDelim::a(t, e)),
            GrabCxt::EvalR(v, cxt) => cxt.plug(LamGrabDelim::a(v.into_super(), t)),
            GrabCxt::Del(cxt) => cxt.plug(LamGrabDelim::d(t)),
        }
    }
    pub fn extend_r(self, v: Value) -> Self {
        match self {
            GrabCxt::Hole => GrabCxt::EvalR(v, Box::new(GrabCxt::Hole)),
            GrabCxt::EvalL(e1, c) => GrabCxt::EvalL(e1, Box::new(c.extend_r(v))),
            GrabCxt::EvalR(e1, c) => GrabCxt::EvalR(e1, Box::new(c.extend_r(v))),
            GrabCxt::Del(c) => GrabCxt::Del(Box::new(c.extend_r(v))),
        }
    }
    pub fn extend_l(self, e: LamGrabDelim) -> Self {
        match self {
            GrabCxt::Hole => GrabCxt::EvalL(e, Box::new(GrabCxt::Hole)),
            GrabCxt::EvalL(e1, c) => GrabCxt::EvalL(e1, Box::new(c.extend_l(e))),
            GrabCxt::EvalR(e1, c) => GrabCxt::EvalR(e1, Box::new(c.extend_l(e))),
            GrabCxt::Del(c) => GrabCxt::Del(Box::new(c.extend_l(e))),
        }
    }
    pub fn extend_d(self) -> Self {
        match self {
            GrabCxt::Hole => GrabCxt::Del(Box::new(GrabCxt::Hole)),
            GrabCxt::EvalL(e1, c) => GrabCxt::EvalL(e1, Box::new(c.extend_d())),
            GrabCxt::EvalR(e1, c) => GrabCxt::EvalR(e1, Box::new(c.extend_d())),
            GrabCxt::Del(c) => GrabCxt::Del(Box::new(c.extend_d())),
        }
    }
    pub fn purify(self) -> Option<GrabPureCxt> {
        match self {
            GrabCxt::Hole => Some(GrabPureCxt::Hole),
            GrabCxt::EvalL(e, f) => Some(GrabPureCxt::EvalL(e, Box::new(f.purify()?))),
            GrabCxt::EvalR(v, f) => Some(GrabPureCxt::EvalR(v, Box::new(f.purify()?))),
            GrabCxt::Del(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Frame {
    EvalL(LamGrabDelim), // [[] e]
    EvalR(Value),        // [v []]
    Del,
}

impl Frame {
    fn plug(self, t: LamGrabDelim) -> LamGrabDelim {
        match self {
            Frame::EvalL(e) => LamGrabDelim::a(t, e),
            Frame::EvalR(v) => LamGrabDelim::a(v.into_super(), t),
            Frame::Del => LamGrabDelim::d(t),
        }
    }
}

pub fn frames_to_cxt(frames: Vec<Frame>) -> GrabCxt {
    frames
        .into_iter()
        .fold(GrabCxt::Hole, |cxt, frame| match frame {
            Frame::EvalL(e) => GrabCxt::EvalL(e, Box::new(cxt)),
            Frame::EvalR(v) => GrabCxt::EvalR(v, Box::new(cxt)),
            Frame::Del => GrabCxt::Del(Box::new(cxt)),
        })
}

pub enum RedexInfo {
    // (\x. e) v
    AbsApp {
        x: Var,
        e: LamGrabDelim,
        v: LamGrabDelim, // e2.is_value()
    },
    // delim v
    DelimVal {
        v: LamGrabDelim, // v.is_value()
    },
    // delim F[grab k. e]
    DelimGrab {
        cxt: GrabPureCxt,
        k: Var,
        e: LamGrabDelim,
    },
}

impl SubSet for RedexInfo {
    type Super = LamGrabDelim;
    fn from_super(s: &Self::Super) -> Option<Self> {
        // t = delim F[grab k. e] と書けるか
        fn cxt_grab(mut e: LamGrabDelim) -> Option<(Var, LamGrabDelim, GrabPureCxt)> {
            let mut cxt = GrabPureCxt::Hole;
            let (k, e) = loop {
                let ne = match e {
                    LamGrabDelim::Var(_) | LamGrabDelim::Lam(_, _) | LamGrabDelim::Delim(_) => {
                        return None;
                    }
                    LamGrabDelim::App(ref e1, ref e2)
                        if Value::from_super(e1).is_some() && Value::from_super(e2).is_some() =>
                    {
                        return None;
                    }
                    LamGrabDelim::App(e1, e2) => {
                        if let Some(v) = Value::from_super(&e1) {
                            cxt = cxt.extend_r(v);
                            *e2
                        } else {
                            cxt = cxt.extend_l(*e2);
                            *e1
                        }
                    }
                    LamGrabDelim::Grab(k, e1) => {
                        break (k, e1);
                    }
                };
                e = ne;
            };
            Some((k, *e, cxt))
        }
        match s {
            LamGrabDelim::Var(_) => None,
            LamGrabDelim::Lam(_, _) => None,
            LamGrabDelim::App(e1, e2) => {
                let Value::Function(x, e) = Value::from_super(e1)?;
                Value::from_super(e2)?;
                Some(RedexInfo::AbsApp {
                    x,
                    e: *e,
                    v: *(*e2).clone(),
                })
            }
            LamGrabDelim::Delim(e) => {
                if Value::from_super(e).is_some() {
                    Some(RedexInfo::DelimVal {
                        v: e.as_ref().clone(),
                    })
                } else {
                    let e = e.as_ref().clone();
                    let (k, e, cxt) = cxt_grab(e)?;
                    Some(RedexInfo::DelimGrab { cxt, k, e })
                }
            }
            _ => None,
        }
    }

    fn into_super(self) -> Self::Super {
        match self {
            RedexInfo::AbsApp { x, e, v } => {
                LamGrabDelim::App(Box::new(LamGrabDelim::Lam(x, Box::new(e))), Box::new(v))
            }
            RedexInfo::DelimVal { v } => LamGrabDelim::Delim(Box::new(v)),
            RedexInfo::DelimGrab { cxt, k, e } => {
                LamGrabDelim::Delim(Box::new(cxt.plug(LamGrabDelim::Grab(k, Box::new(e)))))
            }
        }
    }
}

// incorrect
// pub fn incorrect_natural_l2rcbv(t: LamGrabDelim) -> Option<LamGrabDelim> {
//     if let Some(rdxinfo) = RedexInfo::from_super(&t) {
//         return Some(LamGrabDelim::redex_step(rdxinfo));
//     }
//     match t {
//         LamGrabDelim::Var(_) => None,
//         LamGrabDelim::Lam(_, _) => None,
//         LamGrabDelim::App(e1, e2) => {
//             if Value::from_super(&e1).is_some() {
//                 Some(LamGrabDelim::a(*e1, incorrect_natural_l2rcbv(*e2)?))
//             } else {
//                 Some(LamGrabDelim::a(incorrect_natural_l2rcbv(*e1)?, *e2))
//             }
//         }
//         LamGrabDelim::Delim(e) => Some(LamGrabDelim::d(incorrect_natural_l2rcbv(*e)?)),
//         LamGrabDelim::Grab(_, _) => None,
//     }
// }

impl LambdaExt for LamGrabDelim {
    type Value = Value;
    type RedexInfo = RedexInfo;
    fn free_variables(&self) -> HashSet<Var> {
        match self {
            LamGrabDelim::Var(x) => HashSet::from_iter(vec![x.clone()]),
            LamGrabDelim::Lam(x, e) => {
                let mut s = e.free_variables();
                s.remove(x);
                s
            }
            LamGrabDelim::App(e1, e2) => {
                let mut s = HashSet::new();
                s.extend(e1.free_variables());
                s.extend(e2.free_variables());
                s
            }
            LamGrabDelim::Delim(e) => e.free_variables(),
            LamGrabDelim::Grab(k, e) => {
                let mut s = e.free_variables();
                s.remove(k);
                s
            }
        }
    }

    fn bound_variables(&self) -> HashSet<Var> {
        match self {
            LamGrabDelim::Var(_) => HashSet::new(),
            LamGrabDelim::Lam(x, e) => {
                let mut s = e.bound_variables();
                s.insert(x.clone());
                s
            }
            LamGrabDelim::App(e1, e2) => {
                let mut s = HashSet::new();
                s.extend(e1.bound_variables());
                s.extend(e2.bound_variables());
                s
            }
            LamGrabDelim::Grab(k, e) => {
                let mut s = e.bound_variables();
                s.insert(k.clone());
                s
            }
            LamGrabDelim::Delim(e) => e.free_variables(),
        }
    }

    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
        fn alpha_conversion_canonical_rec(
            e: LamGrabDelim,
            mut v: variable::VarMap,
        ) -> LamGrabDelim {
            match e {
                LamGrabDelim::Var(x) => LamGrabDelim::Var(v.get_table(&x)),
                LamGrabDelim::Lam(x, e) => {
                    v.push_var(&x);
                    LamGrabDelim::Lam(
                        v.get_table(&x),
                        Box::new(alpha_conversion_canonical_rec(*e, v)),
                    )
                }
                LamGrabDelim::App(e1, e2) => LamGrabDelim::App(
                    Box::new(alpha_conversion_canonical_rec(*e1, v.clone())),
                    Box::new(alpha_conversion_canonical_rec(*e2, v)),
                ),
                LamGrabDelim::Grab(k, e) => {
                    v.push_var(&k);
                    LamGrabDelim::Grab(
                        v.get_table(&k),
                        Box::new(alpha_conversion_canonical_rec(*e, v)),
                    )
                }
                LamGrabDelim::Delim(e) => {
                    LamGrabDelim::Delim(Box::new(alpha_conversion_canonical_rec(*e, v)))
                }
            }
        }

        let maps: variable::VarMap =
            variable::VarMap::new_iter(self.free_variables().into_iter().chain(vs));

        alpha_conversion_canonical_rec(self, maps)
    }

    fn subst(self, x: Var, t: LamGrabDelim) -> Self {
        pub fn simple_subst(e: LamGrabDelim, x: Var, t: LamGrabDelim) -> LamGrabDelim {
            match e {
                LamGrabDelim::Var(y) => {
                    if x == y {
                        t
                    } else {
                        LamGrabDelim::v(y)
                    }
                }
                LamGrabDelim::Lam(y, e) => {
                    if x == y {
                        LamGrabDelim::l(y, *e)
                    } else {
                        LamGrabDelim::l(y, simple_subst(*e, x, t))
                    }
                }
                LamGrabDelim::App(e1, e2) => LamGrabDelim::a(
                    simple_subst(*e1, x.clone(), t.clone()),
                    simple_subst(*e2, x, t),
                ),
                LamGrabDelim::Delim(e) => LamGrabDelim::d(simple_subst(*e, x, t)),
                LamGrabDelim::Grab(k, e) => {
                    if k == x {
                        LamGrabDelim::g(k, *e)
                    } else {
                        LamGrabDelim::g(k, simple_subst(*e, x, t))
                    }
                }
            }
        }

        let free_t = t.free_variables();
        let e = self.alpha_conversion_canonical(free_t);
        simple_subst(e, x, t)
    }

    fn redex_step(r: Self::RedexInfo) -> Self {
        match r {
            RedexInfo::AbsApp { x, e, v } => e.subst(x, v),
            RedexInfo::DelimVal { v } => v,
            RedexInfo::DelimGrab { cxt, k, e } => {
                let new_var: Var = 0.into();
                let cont = LamGrabDelim::l(
                    new_var.clone(),
                    LamGrabDelim::d(cxt.plug(LamGrabDelim::v(new_var))),
                );
                e.subst(k, cont)
            }
        }
    }

    fn step(self) -> Option<Self> {
        if let Some(r) = RedexInfo::from_super(&self) {
            Some(LamGrabDelim::redex_step(r))
        } else {
            let e: LamGrabDelim = match self {
                LamGrabDelim::Var(_) | LamGrabDelim::Lam(_, _) | LamGrabDelim::Grab(_, _) => {
                    return None
                }
                LamGrabDelim::App(e1, e2) => {
                    if let Some(v) = Value::from_super(&e1) {
                        LamGrabDelim::a(v.into_super(), e2.step()?)
                    } else {
                        LamGrabDelim::a(e1.step()?, *e2)
                    }
                }
                LamGrabDelim::Delim(e) => LamGrabDelim::d(e.step()?),
            };
            Some(e)
        }
    }
}

pub fn decomp_with_cxt(mut t: LamGrabDelim) -> Option<(RedexInfo, GrabCxt)> {
    if Value::from_super(&t).is_some() {
        return None;
    }
    let mut cxt = GrabCxt::Hole;
    let rdx = loop {
        if let Some(rdx) = RedexInfo::from_super(&t) {
            break rdx;
        }
        let nt = match t {
            LamGrabDelim::Var(_) => {
                return None;
            }
            LamGrabDelim::Lam(_, _) => {
                unreachable!()
            }
            LamGrabDelim::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    cxt = cxt.extend_r(v);
                    *e2
                } else {
                    cxt = cxt.extend_l(*e2);
                    *e1
                }
            }
            LamGrabDelim::Delim(e) => {
                cxt = cxt.extend_d();
                *e
            }
            LamGrabDelim::Grab(_, _) => {
                return None;
            }
        };
        t = nt;
    };
    Some((rdx, cxt))
}

pub fn step_with_cxt(t: LamGrabDelim) -> Option<LamGrabDelim> {
    let (rdx, cxt) = decomp_with_cxt(t)?;
    Some(cxt.plug(LamGrabDelim::redex_step(rdx)))
}

pub struct State {
    stack: Vec<Frame>,
    lam: LamGrabDelim,
}

pub fn step_machine(State { mut stack, lam }: State) -> Option<State> {
    if Value::from_super(&lam).is_some() {
        if let Some(top) = stack.pop() {
            let lam = top.plug(lam);
            return Some(State { stack, lam });
        } else {
            return None;
        }
    }
    if let Some(rdx) = RedexInfo::from_super(&lam) {
        Some(State {
            stack,
            lam: LamGrabDelim::redex_step(rdx),
        })
    } else {
        match lam {
            LamGrabDelim::Var(_) => None,
            LamGrabDelim::Lam(_, _) => None,
            LamGrabDelim::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    stack.push(Frame::EvalR(v));
                    Some(State { stack, lam: *e2 })
                } else {
                    stack.push(Frame::EvalL(*e2));
                    Some(State { stack, lam: *e1 })
                }
            }
            LamGrabDelim::Grab(k, e) => {
                let i = stack.iter().position(|frame| *frame == Frame::Del)?;
                let mut old = stack.split_off(i);
                old.pop().unwrap();
                let cxt = frames_to_cxt(stack).purify().unwrap();
                let rdx = RedexInfo::DelimGrab { cxt, k, e: *e };
                Some(State {
                    stack: old,
                    lam: rdx.into_super(),
                })
            }
            LamGrabDelim::Delim(e) => {
                stack.push(Frame::Del);
                Some(State { stack, lam: *e })
            }
        }
    }
}
