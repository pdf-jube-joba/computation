use std::{collections::HashSet, fmt::Display};

use utils::{set::SubSet, variable::Var};

use crate::LambdaExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
    Abort(Box<Lam>),
    Control(Box<Lam>),
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
    pub fn ab(e: Lam) -> Lam {
        Lam::Abort(Box::new(e))
    }
    pub fn ct<T>(e: Lam) -> Lam
    where
        T: Into<Var>,
    {
        Lam::Control(Box::new(e))
    }
}

impl Display for Lam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Lam::Var(var) => format!("{var}"),
            Lam::Lam(var, term) => format!("\\{var}. {term}"),
            Lam::App(e1, e2) => format!("({e1} @ {e2})"),
            Lam::Abort(e) => format!("abort {e}"),
            Lam::Control(e) => format!("control {e}"),
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

pub enum RedexInfo {
    // (\x. e) v
    AbsApp {
        x: Var,
        e: Lam,
        v: Value, // e2.is_value()
    },
}

impl SubSet for RedexInfo {
    type Super = Lam;
    fn from_super(s: &Self::Super) -> Option<Self> {
        match s {
            Lam::App(e1, e2) => {
                let Value::Function(x, e) = Value::from_super(e1)?;
                let v = Value::from_super(e2)?;
                Some(RedexInfo::AbsApp { x, e: *e, v })
            }
            _ => None,
        }
    }
    fn into_super(self) -> Self::Super {
        match self {
            RedexInfo::AbsApp { x, e, v } => {
                Lam::App(Box::new(Lam::Lam(x, Box::new(e))), Box::new(v.into_super()))
            }
        }
    }
}

pub enum Cxt {
    Hole,
    EvalL(Lam, Box<Cxt>),
    EvalR(Value, Box<Cxt>),
}

impl Cxt {
    pub fn plug(self, t: Lam) -> Lam {
        match self {
            Cxt::Hole => t,
            Cxt::EvalL(e, cxt) => cxt.plug(Lam::a(t, e)),
            Cxt::EvalR(v, cxt) => cxt.plug(Lam::a(v.into_super(), t)),
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

impl LambdaExt for Lam {
    type RedexInfo = RedexInfo;
    type Value = Value;
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
            Lam::Abort(e) => e.free_variables(),
            Lam::Control(e) => e.free_variables(),
        }
    }
    fn bound_variables(&self) -> HashSet<Var> {
        let mut s = HashSet::new();
        match self {
            Lam::Var(_) => {}
            Lam::Lam(x, e) => {
                s.insert(x.clone());
                s.extend(e.bound_variables());
            }
            Lam::App(e1, e2) => {
                s.extend(e1.bound_variables());
                s.extend(e2.bound_variables());
            }
            Lam::Abort(e) => {
                s.extend(e.bound_variables());
            }
            Lam::Control(e) => s.extend(e.bound_variables()),
        }
        s
    }
    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
        fn alpha_conversion_canonical_rec(e: Lam, mut v: utils::variable::VarMap) -> Lam {
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
                Lam::Abort(e) => Lam::Abort(Box::new(alpha_conversion_canonical_rec(*e, v))),
                Lam::Control(e) => Lam::Control(Box::new(alpha_conversion_canonical_rec(*e, v))),
            }
        }

        let maps: utils::variable::VarMap =
            utils::variable::VarMap::new_iter(self.free_variables().into_iter().chain(vs));

        alpha_conversion_canonical_rec(self, maps)
    }

    fn subst(self, x: Var, t: Self) -> Self {
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
                Lam::Abort(e) => Lam::Abort(Box::new(simple_subst(*e, x, t))),
                Lam::Control(e) => Lam::Control(Box::new(simple_subst(*e, x, t))),
            }
        }

        let free_t = t.free_variables();
        let e = self.alpha_conversion_canonical(free_t);
        simple_subst(e, x, t)
    }

    fn redex_step(r: Self::RedexInfo) -> Self {
        match r {
            RedexInfo::AbsApp { x, e, v } => e.subst(x, v.into_super()),
        }
    }

    fn step(self) -> Option<Self> {
        // t = E[Abort(M)] ?
        fn destruct_abort(t: Lam) -> Option<(Cxt, Lam)> {
            todo!()
        }
        // t = E[Control(M)] ?
        fn destruct_control(t: Lam) -> Option<(Cxt, Lam)> {
            todo!()
        }
        if let Some(r) = RedexInfo::from_super(&self) {
            return Some(Lam::redex_step(r));
        }
        if let Some((e, t)) = destruct_abort(self.clone()) {
            return Some(t);
        }
        if let Some((e, t)) = destruct_control(self.clone()) {
            let cont: Lam = todo!();
            todo!()
        }
        match self {
            Lam::App(e1, e2) => {
                if let Some(v) = Value::from_super(&e1) {
                    Some(Lam::a(v.into_super(), e2.step()?))
                } else {
                    Some(Lam::a(e1.step()?, *e2))
                }
            }
            _ => None,
        }
    }
}
