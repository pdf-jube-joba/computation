use std::{collections::HashSet, fmt::Display};

use utils::{set::SubSet, variable::Var};

use crate::{LambdaContext, LambdaExt, State};

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
    pub fn ct(e: Lam) -> Lam {
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
            RedexInfo::AbsApp { x, e, v } => Lam::a(Lam::l(x, e), v.into_super()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Frame {
    EvalL(Lam),   // [[] t]
    EvalR(Value), // [v []]
}

impl Frame {
    fn plug(self, t: Lam) -> Lam {
        match self {
            Frame::EvalL(e) => Lam::a(t, e),
            Frame::EvalR(v) => Lam::a(v.into_super(), t),
        }
    }
    fn free_variables(&self) -> HashSet<Var> {
        match self {
            Frame::EvalL(t) => t.free_variables(),
            Frame::EvalR(v) => v.clone().into_super().free_variables(),
        }
    }
    fn bound_variables(&self) -> HashSet<Var> {
        match self {
            Frame::EvalL(t) => t.free_variables(),
            Frame::EvalR(v) => v.clone().into_super().free_variables(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cxt(Vec<Frame>);

impl Cxt {
    pub fn plug(mut self, t: Lam) -> Lam {
        if let Some(frame) = self.0.pop() {
            Cxt(self.0).plug(frame.plug(t))
        } else {
            t
        }
    }
    pub fn free_variables(&self) -> HashSet<Var> {
        self.0
            .iter()
            .flat_map(|frame| frame.free_variables().into_iter())
            .collect()
    }
    pub fn bound_variables(&self) -> HashSet<Var> {
        self.0
            .iter()
            .flat_map(|frame| frame.bound_variables().into_iter())
            .collect()
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
                    Lam::l(v.get_table(&x), alpha_conversion_canonical_rec(*e, v))
                }
                Lam::App(e1, e2) => Lam::a(
                    alpha_conversion_canonical_rec(*e1, v.clone()),
                    alpha_conversion_canonical_rec(*e2, v),
                ),
                Lam::Abort(e) => Lam::ab(alpha_conversion_canonical_rec(*e, v)),
                Lam::Control(e) => Lam::ct(alpha_conversion_canonical_rec(*e, v)),
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
                Lam::Abort(e) => Lam::ab(simple_subst(*e, x, t)),
                Lam::Control(e) => Lam::ct(simple_subst(*e, x, t)),
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
        fn destruct_abort(mut t: Lam) -> Option<(Cxt, Lam)> {
            let mut stack = vec![];
            loop {
                match t {
                    Lam::Var(_) | Lam::Lam(_, _) | Lam::Control(_) => return None,
                    Lam::App(e1, e2) => {
                        if let Some(v) = Value::from_super(&e1) {
                            stack.push(Frame::EvalR(v));
                            t = *e2;
                        } else {
                            stack.push(Frame::EvalL(*e2));
                            t = *e1;
                        }
                    }
                    Lam::Abort(m) => {
                        return Some((Cxt(stack), *m));
                    }
                }
            }
        }
        // t = E[Control(M)] ?
        fn destruct_control(mut t: Lam) -> Option<(Cxt, Lam)> {
            let mut stack = vec![];
            loop {
                match t {
                    Lam::Var(_) | Lam::Lam(_, _) | Lam::Abort(_) => return None,
                    Lam::App(e1, e2) => {
                        if let Some(v) = Value::from_super(&e1) {
                            stack.push(Frame::EvalR(v));
                            t = *e2;
                        } else {
                            stack.push(Frame::EvalL(*e2));
                            t = *e1;
                        }
                    }
                    Lam::Control(m) => {
                        return Some((Cxt(stack), *m));
                    }
                }
            }
        }
        if let Some(r) = RedexInfo::from_super(&self) {
            return Some(Lam::redex_step(r));
        }
        if let Some((_, t)) = destruct_abort(self.clone()) {
            return Some(t);
        }
        if let Some((cxt, t)) = destruct_control(self.clone()) {
            let mut sets = HashSet::new();
            sets.extend(t.free_variables());
            sets.extend(t.bound_variables());
            sets.extend(cxt.free_variables());
            sets.extend(cxt.bound_variables());
            let new_var: Var = utils::variable::new_var(sets);
            let cont: Lam = Lam::l(new_var.clone(), Lam::ab(cxt.plug(Lam::v(new_var))));
            return Some(Lam::a(t, cont));
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

impl LambdaContext for Lam {
    type Frame = Frame;
    fn decomp(e: Self) -> Option<(Self::Frame, Self)> {
        if let Lam::App(e1, e2) = e {
            if let Some(v) = Value::from_super(&e1) {
                Some((Frame::EvalR(v), *e1))
            } else {
                Some((Frame::EvalL(*e2), *e1))
            }
        } else {
            None
        }
    }
    fn plug(frame: Self::Frame, e: Self) -> Self {
        match frame {
            Frame::EvalR(v) => Lam::a(v.into_super(), e),
            Frame::EvalL(t) => Lam::a(e, t),
        }
    }
    fn step_state(State { mut stack, mut top }: State<Self>) -> Option<State<Self>> {
        if let Some(v) = Value::from_super(&top) {
            if let Some(frame) = stack.pop() {
                let top = frame.plug(top);
                Some(State { stack, top })
            } else {
                None
            }
        } else if let Some(r) = RedexInfo::from_super(&top) {
            Some(State {
                stack,
                top: Lam::redex_step(r),
            })
        } else if let Lam::Abort(t) = top {
            Some(State {
                stack: vec![],
                top: *t,
            })
        } else if let Lam::Control(t) = top {
            let mut sets = HashSet::new();
            sets.extend(t.free_variables());
            sets.extend(t.bound_variables());
            for frame in &stack {
                sets.extend(frame.free_variables());
                sets.extend(frame.bound_variables());
            }
            let new_var: Var = utils::variable::new_var(sets);
            let cont: Lam = Lam::l(new_var.clone(), Lam::ab(Cxt(stack).plug(Lam::v(new_var))));
            Some(State {
                stack: vec![],
                top: Lam::a(*t, cont),
            })
        } else if let Some((frame, t)) = <Lam as LambdaContext>::decomp(top) {
            stack.push(frame);
            Some(State { stack, top: t })
        } else {
            None
        }
    }
}
