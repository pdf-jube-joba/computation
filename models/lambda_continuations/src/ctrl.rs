use std::{collections::HashSet, fmt::Display};

use utils::variable::Var;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LamAbortControl {
    Var(Var),
    Lam(Var, Box<LamAbortControl>),
    App(Box<LamAbortControl>, Box<LamAbortControl>),
    Abort(Box<LamAbortControl>),
    Control(Box<LamAbortControl>),
}

impl LamAbortControl {
    pub fn v<T>(n: T) -> LamAbortControl
    where
        T: Into<Var>,
    {
        LamAbortControl::Var(n.into())
    }
    pub fn l<T>(n: T, e: LamAbortControl) -> LamAbortControl
    where
        T: Into<Var>,
    {
        LamAbortControl::Lam(n.into(), Box::new(e))
    }
    pub fn a(e1: LamAbortControl, e2: LamAbortControl) -> LamAbortControl {
        LamAbortControl::App(Box::new(e1), Box::new(e2))
    }
    pub fn ab(e: LamAbortControl) -> LamAbortControl {
        LamAbortControl::Abort(Box::new(e))
    }
    pub fn ct<T>(e: LamAbortControl) -> LamAbortControl
    where
        T: Into<Var>,
    {
        LamAbortControl::Control(Box::new(e))
    }
}

impl Display for LamAbortControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            LamAbortControl::Var(var) => format!("{var}"),
            LamAbortControl::Lam(var, term) => format!("\\{var}. {term}"),
            LamAbortControl::App(e1, e2) => format!("({e1} @ {e2})"),
            LamAbortControl::Abort(e) => format!("abort {e}"),
            LamAbortControl::Control(e) => format!("control {e}"),
        };
        write!(f, "{}", string)
    }
}

impl LamAbortControl {
    pub fn free_variables(&self) -> HashSet<Var> {
        match self {
            LamAbortControl::Var(x) => HashSet::from_iter(vec![x.clone()]),
            LamAbortControl::Lam(x, e) => {
                let mut s = e.free_variables();
                s.remove(x);
                s
            }
            LamAbortControl::App(e1, e2) => {
                let mut s = HashSet::new();
                s.extend(e1.free_variables());
                s.extend(e2.free_variables());
                s
            }
            LamAbortControl::Abort(e) => e.free_variables(),
            LamAbortControl::Control(e) => e.free_variables(),
        }
    }
}
