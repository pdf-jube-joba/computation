use std::fmt::Display;

use utils::variable::Var;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LamGrabDelim {
    Var(Var),
    Lam(Var, Box<LamGrabDelim>),
    App(Box<LamGrabDelim>, Box<LamGrabDelim>),
    Abort(Box<LamGrabDelim>),
    Control(Box<LamGrabDelim>),
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
    pub fn ab(e: LamGrabDelim) -> LamGrabDelim {
        LamGrabDelim::Abort(Box::new(e))
    }
    pub fn ct<T>(e: LamGrabDelim) -> LamGrabDelim
    where
        T: Into<Var>,
    {
        LamGrabDelim::Control(Box::new(e))
    }
}
