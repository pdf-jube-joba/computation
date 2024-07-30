use std::fmt::Display;

use utils::variable::Var;

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

pub fn subst(e: LamGrabDelim, x: Var, t: LamGrabDelim) -> LamGrabDelim {
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
                LamGrabDelim::l(y, subst(*e, x, t))
            }
        }
        LamGrabDelim::App(e1, e2) => {
            LamGrabDelim::a(subst(*e1, x.clone(), t.clone()), subst(*e2, x, t))
        }
        LamGrabDelim::Delim(e) => LamGrabDelim::d(subst(*e, x, t)),
        LamGrabDelim::Grab(k, e) => {
            if k == x {
                LamGrabDelim::g(k, *e)
            } else {
                LamGrabDelim::g(k, subst(*e, x, t))
            }
        }
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

impl LamGrabDelim {
    pub fn is_value(&self) -> Option<(Var, LamGrabDelim)> {
        match &self {
            LamGrabDelim::Lam(x, e) => Some((x.clone(), *(*e).clone())),
            _ => None,
        }
    }
}

pub enum GrabPureCxt {
    Hole,                                  // []
    EvalL(LamGrabDelim, Box<GrabPureCxt>), // E[[] e]
    EvalR(LamGrabDelim, Box<GrabPureCxt>), // E[v []]
}

impl GrabPureCxt {
    pub fn plug(self, t: LamGrabDelim) -> LamGrabDelim {
        match self {
            GrabPureCxt::Hole => t,
            GrabPureCxt::EvalL(e, cxt) => cxt.plug(LamGrabDelim::a(t, e)),
            GrabPureCxt::EvalR(e, cxt) => cxt.plug(LamGrabDelim::a(e, t)),
        }
    }
    pub fn extend_r(self, e: LamGrabDelim) -> Self {
        match self {
            GrabPureCxt::Hole => GrabPureCxt::EvalR(e, Box::new(GrabPureCxt::Hole)),
            GrabPureCxt::EvalL(e1, c) => GrabPureCxt::EvalL(e1, Box::new(c.extend_r(e))),
            GrabPureCxt::EvalR(e1, c) => GrabPureCxt::EvalR(e1, Box::new(c.extend_r(e))),
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
    EvalR(LamGrabDelim, Box<GrabCxt>), // E[v []]
    Del(Box<GrabCxt>),                 // E[delimit []] ,
}

impl GrabCxt {
    pub fn plug(self, t: LamGrabDelim) -> LamGrabDelim {
        match self {
            GrabCxt::Hole => t,
            GrabCxt::EvalL(e, cxt) => cxt.plug(LamGrabDelim::a(t, e)),
            GrabCxt::EvalR(v, cxt) => cxt.plug(LamGrabDelim::a(v, t)),
            GrabCxt::Del(cxt) => cxt.plug(LamGrabDelim::d(t)),
        }
    }
    pub fn extend_r(self, e: LamGrabDelim) -> Self {
        match self {
            GrabCxt::Hole => GrabCxt::EvalR(e, Box::new(GrabCxt::Hole)),
            GrabCxt::EvalL(e1, c) => GrabCxt::EvalL(e1, Box::new(c.extend_r(e))),
            GrabCxt::EvalR(e1, c) => GrabCxt::EvalR(e1, Box::new(c.extend_r(e))),
            GrabCxt::Del(c) => GrabCxt::Del(Box::new(c.extend_r(e))),
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
    EvalR(LamGrabDelim), // [v []]
    Del,
}

impl Frame {
    fn plug(self, t: LamGrabDelim) -> LamGrabDelim {
        match self {
            Frame::EvalL(e) => LamGrabDelim::a(t, e),
            Frame::EvalR(v) => LamGrabDelim::a(v, t),
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

pub enum RdxInFo {
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

impl RdxInFo {
    pub fn as_lam(self) -> LamGrabDelim {
        match self {
            RdxInFo::AbsApp { x, e, v } => {
                LamGrabDelim::App(Box::new(LamGrabDelim::Lam(x, Box::new(e))), Box::new(v))
            }
            RdxInFo::DelimVal { v } => LamGrabDelim::Delim(Box::new(v)),
            RdxInFo::DelimGrab { cxt, k, e } => {
                LamGrabDelim::Delim(Box::new(cxt.plug(LamGrabDelim::Grab(k, Box::new(e)))))
            }
        }
    }
    pub fn step(self) -> LamGrabDelim {
        match self {
            RdxInFo::AbsApp { x, e, v } => subst(e, x, v),
            RdxInFo::DelimVal { v } => v,
            RdxInFo::DelimGrab { cxt, k, e } => {
                let new_var: Var = 0.into();
                let cont = LamGrabDelim::l(
                    new_var.clone(),
                    LamGrabDelim::d(cxt.plug(LamGrabDelim::v(new_var))),
                );
                subst(e, k, cont)
            }
        }
    }
}

// t = delim F[grab k. e] と書けるか
pub fn cxt_grab(mut e: LamGrabDelim) -> Option<(Var, LamGrabDelim, GrabPureCxt)> {
    let mut cxt = GrabPureCxt::Hole;
    let (k, e) = loop {
        let ne = match e {
            LamGrabDelim::Var(_) | LamGrabDelim::Lam(_, _) | LamGrabDelim::Delim(_) => {
                return None;
            }
            LamGrabDelim::App(ref e1, ref e2)
                if e1.is_value().is_some() && e2.is_value().is_some() =>
            {
                return None;
            }
            LamGrabDelim::App(e1, e2) => {
                if e1.is_value().is_some() {
                    cxt = cxt.extend_r(*e1);
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

impl LamGrabDelim {
    pub fn is_redux(&self) -> Option<RdxInFo> {
        match self {
            LamGrabDelim::Var(_) => None,
            LamGrabDelim::Lam(_, _) => None,
            LamGrabDelim::App(e1, e2) => {
                let (x, e) = e1.is_value()?;
                e2.is_value()?;
                Some(RdxInFo::AbsApp {
                    x,
                    e,
                    v: *(*e2).clone(),
                })
            }
            LamGrabDelim::Delim(e) => {
                if e.is_value().is_some() {
                    Some(RdxInFo::DelimVal {
                        v: e.as_ref().clone(),
                    })
                } else {
                    let e = e.as_ref().clone();
                    let (k, e, cxt) = cxt_grab(e)?;
                    Some(RdxInFo::DelimGrab { cxt, k, e })
                }
            }
            _ => None,
        }
    }
}

// incorrect
pub fn incorrect_natural_l2rcbv(t: LamGrabDelim) -> Option<LamGrabDelim> {
    if let Some(rdxinfo) = t.is_redux() {
        return Some(rdxinfo.step());
    }
    match t {
        LamGrabDelim::Var(_) => None,
        LamGrabDelim::Lam(_, _) => None,
        LamGrabDelim::App(e1, e2) => {
            if e1.is_value().is_some() {
                Some(LamGrabDelim::a(*e1, incorrect_natural_l2rcbv(*e2)?))
            } else {
                Some(LamGrabDelim::a(incorrect_natural_l2rcbv(*e1)?, *e2))
            }
        }
        LamGrabDelim::Delim(e) => Some(LamGrabDelim::d(incorrect_natural_l2rcbv(*e)?)),
        LamGrabDelim::Grab(_, _) => None,
    }
}

pub fn decomp_with_cxt(mut t: LamGrabDelim) -> Option<(RdxInFo, GrabCxt)> {
    if t.is_value().is_some() {
        return None;
    }
    let mut cxt = GrabCxt::Hole;
    let rdx = loop {
        if let Some(rdx) = t.is_redux() {
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
                if e1.is_value().is_some() {
                    cxt = cxt.extend_r(*e1);
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
    Some(cxt.plug(rdx.step()))
}

pub struct State {
    stack: Vec<Frame>,
    lam: LamGrabDelim,
}

pub fn step_machine(State { mut stack, lam }: State) -> Option<State> {
    if lam.is_value().is_some() {
        if let Some(top) = stack.pop() {
            let lam = top.plug(lam);
            return Some(State { stack, lam });
        } else {
            return None;
        }
    }
    if let Some(rdx) = lam.is_redux() {
        Some(State {
            stack,
            lam: rdx.step(),
        })
    } else {
        match lam {
            LamGrabDelim::Var(_) => None,
            LamGrabDelim::Lam(_, _) => None,
            LamGrabDelim::App(e1, e2) => {
                if e1.is_value().is_some() {
                    stack.push(Frame::EvalR(*e1));
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
                let rdx = RdxInFo::DelimGrab { cxt, k, e: *e };
                Some(State {
                    stack: old,
                    lam: rdx.as_lam(),
                })
            }
            LamGrabDelim::Delim(e) => {
                stack.push(Frame::Del);
                Some(State { stack, lam: *e })
            }
        }
    }
}
