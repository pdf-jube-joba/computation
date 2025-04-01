use std::{collections::HashSet, fmt::Display};

use utils::{
    set::SubSet,
    variable::{self, Var},
};

use crate::{LambdaContext, LambdaExt, State};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
    Delim(Box<Lam>),
    Grab(Var, Box<Lam>),
}

impl Lam {
    pub fn v<T>(n: T) -> Lam
    where
        T: Into<Var>,
    {
        Lam::Var(n.into())
    }
    pub fn l<T>(n: T, e: Lam) -> Lam
    where
        T: Into<Var>,
    {
        Lam::Lam(n.into(), Box::new(e))
    }
    pub fn a(e1: Lam, e2: Lam) -> Lam {
        Lam::App(Box::new(e1), Box::new(e2))
    }
    pub fn d(e: Lam) -> Lam {
        Lam::Delim(Box::new(e))
    }
    pub fn g<T>(k: T, e: Lam) -> Lam
    where
        T: Into<Var>,
    {
        Lam::Grab(k.into(), Box::new(e))
    }
}

impl Display for Lam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Lam::Var(var) => format!("{var}"),
            Lam::Lam(var, term) => format!("\\{var}.{term}"),
            Lam::App(term1, term2) => format!("({term1} @ {term2})"),
            Lam::Delim(term) => format!("delim {term}"),
            Lam::Grab(k, term) => format!("grab {k}. {term}"),
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Function(Var, Box<Lam>),
}

impl SubSet for Value {
    type Super = Lam;
    fn from_super(s: &Self::Super) -> Option<Self> {
        match s {
            Lam::Lam(v, e) => Some(Value::Function(v.clone(), e.clone())),
            _ => None,
        }
    }
    fn into_super(self) -> Self::Super {
        let Value::Function(x, e) = self;
        Lam::Lam(x, e)
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
    EvalL(Lam, Box<GrabPureCxt>), // E[[] e]
    EvalR(Value, Box<GrabPureCxt>),        // E[v []]
}

impl GrabPureCxt {
    pub fn free_variables(&self) -> HashSet<Var> {
        let mut set = HashSet::new();
        match self {
            GrabPureCxt::Hole => {}
            GrabPureCxt::EvalL(e, c) => {
                set.extend(c.free_variables());
                set.extend(e.free_variables());
            }
            GrabPureCxt::EvalR(v, c) => {
                set.extend(c.free_variables());
                set.extend(v.clone().into_super().free_variables());
            }
        }
        set
    }
    pub fn plug(self, t: Lam) -> Lam {
        match self {
            GrabPureCxt::Hole => t,
            GrabPureCxt::EvalL(e, cxt) => cxt.plug(Lam::a(t, e)),
            GrabPureCxt::EvalR(v, cxt) => cxt.plug(Lam::a(v.into_super(), t)),
        }
    }
    pub fn extend_r(self, v: Value) -> Self {
        match self {
            GrabPureCxt::Hole => GrabPureCxt::EvalR(v, Box::new(GrabPureCxt::Hole)),
            GrabPureCxt::EvalL(e1, c) => GrabPureCxt::EvalL(e1, Box::new(c.extend_r(v))),
            GrabPureCxt::EvalR(e1, c) => GrabPureCxt::EvalR(e1, Box::new(c.extend_r(v))),
        }
    }
    pub fn extend_l(self, e: Lam) -> Self {
        match self {
            GrabPureCxt::Hole => GrabPureCxt::EvalL(e, Box::new(GrabPureCxt::Hole)),
            GrabPureCxt::EvalL(e1, c) => GrabPureCxt::EvalL(e1, Box::new(c.extend_l(e))),
            GrabPureCxt::EvalR(e1, c) => GrabPureCxt::EvalR(e1, Box::new(c.extend_l(e))),
        }
    }
}

pub enum GrabCxt {
    Hole,
    EvalL(Lam, Box<GrabCxt>), // E[[] e]
    EvalR(Value, Box<GrabCxt>),        // E[v []]
    Del(Box<GrabCxt>),                 // E[delimit []] ,
}

impl GrabCxt {
    pub fn plug(self, t: Lam) -> Lam {
        match self {
            GrabCxt::Hole => t,
            GrabCxt::EvalL(e, cxt) => cxt.plug(Lam::a(t, e)),
            GrabCxt::EvalR(v, cxt) => cxt.plug(Lam::a(v.into_super(), t)),
            GrabCxt::Del(cxt) => cxt.plug(Lam::d(t)),
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
    pub fn extend_l(self, e: Lam) -> Self {
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
    EvalL(Lam), // [[] e]
    EvalR(Value),        // [v []]
    Del,
}

impl Frame {
    fn plug(self, t: Lam) -> Lam {
        match self {
            Frame::EvalL(e) => Lam::a(t, e),
            Frame::EvalR(v) => Lam::a(v.into_super(), t),
            Frame::Del => Lam::d(t),
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
        e: Lam,
        v: Value, // e2.is_value()
    },
    // delim v
    DelimVal {
        v: Value, // v.is_value()
    },
    // delim F[grab k. e]
    DelimGrab {
        cxt: GrabPureCxt,
        k: Var,
        e: Lam,
    },
}

impl SubSet for RedexInfo {
    type Super = Lam;
    fn from_super(s: &Self::Super) -> Option<Self> {
        // t = delim F[grab k. e] と書けるか
        fn cxt_grab(mut e: Lam) -> Option<(Var, Lam, GrabPureCxt)> {
            let mut cxt = GrabPureCxt::Hole;
            let (k, e) = loop {
                let ne = match e {
                    Lam::Var(_) | Lam::Lam(_, _) | Lam::Delim(_) => {
                        return None;
                    }
                    Lam::App(ref e1, ref e2)
                        if Value::from_super(e1).is_some() && Value::from_super(e2).is_some() =>
                    {
                        return None;
                    }
                    Lam::App(e1, e2) => {
                        if let Some(v) = Value::from_super(&e1) {
                            cxt = cxt.extend_r(v);
                            *e2
                        } else {
                            cxt = cxt.extend_l(*e2);
                            *e1
                        }
                    }
                    Lam::Grab(k, e1) => {
                        break (k, e1);
                    }
                };
                e = ne;
            };
            Some((k, *e, cxt))
        }

        match s {
            Lam::Var(_) => None,
            Lam::Lam(_, _) => None,
            Lam::App(e1, e2) => {
                let Value::Function(x, e) = Value::from_super(e1)?;
                let v = Value::from_super(e2)?;
                Some(RedexInfo::AbsApp { x, e: *e, v })
            }
            Lam::Delim(e) => {
                if let Some(v) = Value::from_super(e) {
                    Some(RedexInfo::DelimVal { v })
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
            RedexInfo::AbsApp { x, e, v } => Lam::App(
                Box::new(Lam::Lam(x, Box::new(e))),
                Box::new(v.into_super()),
            ),
            RedexInfo::DelimVal { v } => Lam::Delim(Box::new(v.into_super())),
            RedexInfo::DelimGrab { cxt, k, e } => {
                Lam::Delim(Box::new(cxt.plug(Lam::Grab(k, Box::new(e)))))
            }
        }
    }
}

// incorrect
// pub fn incorrect_natural_l2rcbv(t: Lam) -> Option<Lam> {
//     if let Some(rdxinfo) = RedexInfo::from_super(&t) {
//         return Some(Lam::redex_step(rdxinfo));
//     }
//     match t {
//         Lam::Var(_) => None,
//         Lam::Lam(_, _) => None,
//         Lam::App(e1, e2) => {
//             if Value::from_super(&e1).is_some() {
//                 Some(Lam::a(*e1, incorrect_natural_l2rcbv(*e2)?))
//             } else {
//                 Some(Lam::a(incorrect_natural_l2rcbv(*e1)?, *e2))
//             }
//         }
//         Lam::Delim(e) => Some(Lam::d(incorrect_natural_l2rcbv(*e)?)),
//         Lam::Grab(_, _) => None,
//     }
// }

impl LambdaExt for Lam {
    type Value = Value;
    type RedexInfo = RedexInfo;
    fn free_variables(&self) -> HashSet<Var> {
        match self {
            Lam::Var(x) => HashSet::from_iter(vec![x.clone()]),
            Lam::Lam(x, e) => {
                let mut s = e.free_variables();
                s.remove(x);
                s
            }
            Lam::App(e1, e2) => {
                let mut s = HashSet::new();
                s.extend(e1.free_variables());
                s.extend(e2.free_variables());
                s
            }
            Lam::Delim(e) => e.free_variables(),
            Lam::Grab(k, e) => {
                let mut s = e.free_variables();
                s.remove(k);
                s
            }
        }
    }

    fn bound_variables(&self) -> HashSet<Var> {
        match self {
            Lam::Var(_) => HashSet::new(),
            Lam::Lam(x, e) => {
                let mut s = e.bound_variables();
                s.insert(x.clone());
                s
            }
            Lam::App(e1, e2) => {
                let mut s = HashSet::new();
                s.extend(e1.bound_variables());
                s.extend(e2.bound_variables());
                s
            }
            Lam::Grab(k, e) => {
                let mut s = e.bound_variables();
                s.insert(k.clone());
                s
            }
            Lam::Delim(e) => e.free_variables(),
        }
    }

    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
        fn alpha_conversion_canonical_rec(
            e: Lam,
            mut v: variable::VarMap,
        ) -> Lam {
            match e {
                Lam::Var(x) => Lam::Var(v.get_table(&x)),
                Lam::Lam(x, e) => {
                    v.push_var(&x);
                    Lam::Lam(
                        v.get_table(&x),
                        Box::new(alpha_conversion_canonical_rec(*e, v)),
                    )
                }
                Lam::App(e1, e2) => Lam::App(
                    Box::new(alpha_conversion_canonical_rec(*e1, v.clone())),
                    Box::new(alpha_conversion_canonical_rec(*e2, v)),
                ),
                Lam::Grab(k, e) => {
                    v.push_var(&k);
                    Lam::Grab(
                        v.get_table(&k),
                        Box::new(alpha_conversion_canonical_rec(*e, v)),
                    )
                }
                Lam::Delim(e) => {
                    Lam::Delim(Box::new(alpha_conversion_canonical_rec(*e, v)))
                }
            }
        }

        let maps: variable::VarMap =
            variable::VarMap::new_iter(self.free_variables().into_iter().chain(vs));

        alpha_conversion_canonical_rec(self, maps)
    }

    fn subst(self, x: Var, t: Lam) -> Self {
        pub fn simple_subst(e: Lam, x: Var, t: Lam) -> Lam {
            match e {
                Lam::Var(y) => {
                    if x == y {
                        t
                    } else {
                        Lam::v(y)
                    }
                }
                Lam::Lam(y, e) => {
                    if x == y {
                        Lam::l(y, *e)
                    } else {
                        Lam::l(y, simple_subst(*e, x, t))
                    }
                }
                Lam::App(e1, e2) => Lam::a(
                    simple_subst(*e1, x.clone(), t.clone()),
                    simple_subst(*e2, x, t),
                ),
                Lam::Delim(e) => Lam::d(simple_subst(*e, x, t)),
                Lam::Grab(k, e) => {
                    if k == x {
                        Lam::g(k, *e)
                    } else {
                        Lam::g(k, simple_subst(*e, x, t))
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
            RedexInfo::AbsApp { x, e, v } => e.subst(x, v.into_super()),
            RedexInfo::DelimVal { v } => v.into_super(),
            RedexInfo::DelimGrab { cxt, k, e } => {
                let free_variables = cxt.free_variables();
                let new_var: Var = variable::new_var(free_variables);
                let cont = Lam::l(
                    new_var.clone(),
                    Lam::d(cxt.plug(Lam::v(new_var))),
                );
                e.subst(k, cont)
            }
        }
    }

    fn step(self) -> Option<Self> {
        if let Some(r) = RedexInfo::from_super(&self) {
            Some(Lam::redex_step(r))
        } else {
            let e: Lam = match self {
                Lam::Var(_) | Lam::Lam(_, _) | Lam::Grab(_, _) => {
                    return None
                }
                Lam::App(e1, e2) => {
                    if let Some(v) = Value::from_super(&e1) {
                        Lam::a(v.into_super(), e2.step()?)
                    } else {
                        Lam::a(e1.step()?, *e2)
                    }
                }
                Lam::Delim(e) => Lam::d(e.step()?),
            };
            Some(e)
        }
    }
}

pub fn decomp_with_cxt(mut t: Lam) -> Option<(RedexInfo, GrabCxt)> {
    if Value::from_super(&t).is_some() {
        return None;
    }
    let mut cxt = GrabCxt::Hole;
    let rdx = loop {
        if let Some(rdx) = RedexInfo::from_super(&t) {
            break rdx;
        }
        let nt = match t {
            Lam::Var(_) => {
                return None;
            }
            Lam::Lam(_, _) => {
                unreachable!()
            }
            Lam::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    cxt = cxt.extend_r(v);
                    *e2
                } else {
                    cxt = cxt.extend_l(*e2);
                    *e1
                }
            }
            Lam::Delim(e) => {
                cxt = cxt.extend_d();
                *e
            }
            Lam::Grab(_, _) => {
                return None;
            }
        };
        t = nt;
    };
    Some((rdx, cxt))
}

pub fn step_with_cxt(t: Lam) -> Option<Lam> {
    let (rdx, cxt) = decomp_with_cxt(t)?;
    Some(cxt.plug(Lam::redex_step(rdx)))
}

impl LambdaContext for Lam {
    type Frame = Frame;
    fn decomp(e: Self) -> Option<(Self::Frame, Self)> {
        let (frame, exp) = match e {
            Lam::Var(_) | Lam::Lam(_, _) | Lam::Grab(_, _) => {
                return None
            }
            Lam::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    (Frame::EvalR(v), *e2)
                } else {
                    (Frame::EvalL(*e2), *e1)
                }
            }
            Lam::Delim(e) => (Frame::Del, *e),
        };
        Some((frame, exp))
    }
    fn plug(frame: Self::Frame, e: Self) -> Self {
        frame.plug(e)
    }
    fn step_state(State { mut stack, top }: State<Self>) -> Option<State<Self>> {
        if Value::from_super(&top).is_some() {
            if let Some(frame) = stack.pop() {
                let top = frame.plug(top);
                return Some(crate::State { stack, top });
            } else {
                return None;
            }
        }
        if let Some(rdx) = RedexInfo::from_super(&top) {
            Some(State {
                stack,
                top: Lam::redex_step(rdx),
            })
        } else if let Some((frame, top)) = Lam::decomp(top.clone()) {
            stack.push(frame);
            Some(State { stack, top })
        } else if let Lam::Grab(k, e) = top {
            let i = stack.iter().position(|frame| *frame == Frame::Del)?;
            let mut old = stack.split_off(i);
            old.pop().unwrap();
            let cxt = frames_to_cxt(stack).purify().unwrap();
            let rdx = RedexInfo::DelimGrab { cxt, k, e: *e };
            Some(State {
                stack: old,
                top: rdx.into_super(),
            })
        } else {
            None
        }
    }
}
