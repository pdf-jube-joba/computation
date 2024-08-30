use crate::{LambdaContext, LambdaExt, State};
use std::{collections::HashSet, fmt::Display};
use utils::set::SubSet;
use utils::variable::{self, Var};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
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
}

impl Display for Lam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Lam::Var(var) => format!("{var}"),
            Lam::Lam(var, term) => format!("\\{var}. {term}"),
            Lam::App(term1, term2) => format!("({term1} @ {term2})"),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RedexInfo {
    x: Var,
    e: Lam,
    v: Value, // should  v.is_value()
}

impl SubSet for RedexInfo {
    type Super = Lam;
    fn from_super(s: &Self::Super) -> Option<Self> {
        match s {
            Lam::App(e1, e2) => match (Value::from_super(e1), Value::from_super(e2)) {
                (Some(Value::Function(x, e)), Some(v)) => Some(RedexInfo { x, e: *e, v }),
                _ => None,
            },
            _ => None,
        }
    }
    fn into_super(self) -> Self::Super {
        let RedexInfo { x, e, v } = self;
        Lam::a(Lam::l(x, e), v.into_super())
    }
}

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
        }
    }

    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
        fn alpha_conversion_canonical_rec(e: Lam, mut v: variable::VarMap) -> Lam {
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
            }
        }

        let free_t = t.free_variables();
        let e = self.alpha_conversion_canonical(free_t);
        simple_subst(e, x, t)
    }

    fn redex_step(r: Self::RedexInfo) -> Self {
        let RedexInfo { x, e, v } = r;
        e.subst(x, v.into_super())
    }

    fn step(self) -> Option<Self> {
        if let Some(redex) = RedexInfo::from_super(&self) {
            return Some(Lam::redex_step(redex));
        }
        match self {
            Lam::Var(_) | Lam::Lam(_, _) => None,
            Lam::App(e1, e2) => {
                if Value::from_super(&e1).is_some() {
                    Some(Lam::App(Box::new(e1.step()?), e2))
                } else {
                    Some(Lam::App(e1, Box::new(e2.step()?)))
                }
            }
        }
    }
}

// t = ... (v (r e)) ... v: value, r: redex とすると、
// (r, M |-> ... (v (M e)) ... ) と分解する。
#[allow(clippy::type_complexity)]
pub fn decomp_with_cxt_as_func(e: Lam) -> Option<(RedexInfo, Box<dyn Fn(Lam) -> Lam>)> {
    if let Some(rdx) = RedexInfo::from_super(&e) {
        let cxt = |lam: Lam| -> Lam { lam };
        return Some((rdx, Box::new(cxt)));
    }
    match e {
        Lam::Var(_) => None,
        Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => {
            if Value::from_super(&e1).is_some() {
                let (rdx, cxt) = decomp_with_cxt_as_func(*e2)?;
                let cxt = move |lam: Lam| -> Lam {
                    let lam = cxt(lam);
                    Lam::App(e1.clone(), Box::new(lam))
                };
                Some((rdx, Box::new(cxt)))
            } else {
                let (rdx, cxt) = decomp_with_cxt_as_func(*e1)?;
                let cxt = move |lam: Lam| -> Lam {
                    let lam = cxt(lam);
                    Lam::App(Box::new(lam), e2.clone())
                };
                Some((rdx, Box::new(cxt)))
            }
        }
    }
}

pub fn step_with_cxt_as_func(e: Lam) -> Option<Lam> {
    let (rdx, cxt) = decomp_with_cxt_as_func(e)?;
    Some(cxt(Lam::redex_step(rdx)))
}

pub enum Cxt {
    Hole,                   // []
    EvalL(Lam, Box<Cxt>),   // E[[] e]
    EvalR(Value, Box<Cxt>), // E[v []]
}

impl Cxt {
    pub fn plug(self, t: Lam) -> Lam {
        match self {
            Cxt::Hole => t,
            Cxt::EvalR(v, cxt) => cxt.plug(Lam::a(v.into_super(), t)),
            Cxt::EvalL(e, cxt) => cxt.plug(Lam::a(t, e)),
        }
    }
    pub fn extend_r(self, v: Value) -> Self {
        match self {
            Cxt::Hole => Cxt::EvalR(v, Box::new(Cxt::Hole)),
            Cxt::EvalL(e1, c) => Cxt::EvalL(e1, Box::new(c.extend_r(v))),
            Cxt::EvalR(e1, c) => Cxt::EvalR(e1, Box::new(c.extend_r(v))),
        }
    }
    pub fn extend_l(self, e: Lam) -> Self {
        match self {
            Cxt::Hole => Cxt::EvalL(e, Box::new(Cxt::Hole)),
            Cxt::EvalL(e1, c) => Cxt::EvalL(e1, Box::new(c.extend_l(e))),
            Cxt::EvalR(e1, c) => Cxt::EvalR(e1, Box::new(c.extend_l(e))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Frame {
    EvalL(Lam),   // [] e
    EvalR(Value), // v []
}

impl Frame {
    pub fn plug(self, t: Lam) -> Lam {
        match self {
            Frame::EvalR(v) => Lam::a(v.into_super(), t),
            Frame::EvalL(e) => Lam::a(t, e),
        }
    }
}

impl Display for Cxt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Cxt::Hole => "[]".to_string(),
            Cxt::EvalL(e, cxt) => format!("{}[[] @ {}]", e, cxt),
            Cxt::EvalR(v, cxt) => format!("{}[{} @ []]", v, cxt),
        };
        write!(f, "{}", string)
    }
}

pub fn decomp_with_cxt(t: Lam) -> Option<(RedexInfo, Cxt)> {
    if let Some(rdx) = RedexInfo::from_super(&t) {
        return Some((rdx, Cxt::Hole));
    }
    match t {
        Lam::Var(_) => None,
        Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => {
            if let Some(v) = Value::from_super(&e1) {
                let (rdx, cxt) = decomp_with_cxt(*e2)?;
                let new_cxt = cxt.extend_r(v);
                Some((rdx, new_cxt))
            } else {
                let (rdx, cxt) = decomp_with_cxt(*e1)?;
                let new_cxt = cxt.extend_l(*e2);
                Some((rdx, new_cxt))
            }
        }
    }
}

pub fn step_with_cxt(t: Lam) -> Option<Lam> {
    let (rdx, cxt) = decomp_with_cxt(t)?;
    let reduced = Lam::redex_step(rdx);
    Some(cxt.plug(reduced))
}

impl LambdaContext for Lam {
    type Frame = Frame;
    fn decomp(e: Self) -> Option<(Frame, Self)> {
        match e {
            Lam::Var(_) => None,
            Lam::Lam(_, _) => None,
            Lam::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    Some((Frame::EvalR(v), *e2))
                } else {
                    Some((Frame::EvalL(*e2), *e1))
                }
            }
        }
    }

    fn plug(frame: Self::Frame, e: Self) -> Self {
        match frame {
            Frame::EvalR(v) => Lam::a(v.into_super(), e),
            Frame::EvalL(e1) => Lam::a(e, e1),
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
        } else if let Some(rdxinfo) = RedexInfo::from_super(&top) {
            Some(State {
                stack,
                top: Lam::redex_step(rdxinfo),
            })
        } else {
            let (frame, e) = Lam::decomp(top)?;
            stack.push(frame);
            Some(State { stack, top: e })
        }
    }
}
