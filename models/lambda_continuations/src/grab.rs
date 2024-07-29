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
    pub fn v(n: usize) -> LamGrabDelim {
        LamGrabDelim::Var(n.into())
    }
    pub fn l(n: usize, e: LamGrabDelim) -> LamGrabDelim {
        LamGrabDelim::Lam(n.into(), Box::new(e))
    }
    pub fn a(e1: LamGrabDelim, e2: LamGrabDelim) -> LamGrabDelim {
        LamGrabDelim::App(Box::new(e1), Box::new(e2))
    }
    pub fn d(e: LamGrabDelim) -> LamGrabDelim {
        LamGrabDelim::Delim(Box::new(e))
    }
    pub fn g(k: usize, e: LamGrabDelim) -> LamGrabDelim {
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

impl LamGrabDelim {
    pub fn is_value(&self) -> Option<(Var, LamGrabDelim)> {
        match &self {
            LamGrabDelim::Lam(x, e) => Some((x.clone(), *(*e).clone())),
            _ => None,
        }
    }
}

pub enum GrabPureCxt {
    Hole,                                 // []
    AppR(Box<GrabPureCxt>, LamGrabDelim), // E[[] e]
    AppL(Box<GrabPureCxt>, LamGrabDelim), // E[v []]
}

pub enum GrabCxt {
    Hole,
    AppR(Box<GrabCxt>, LamGrabDelim), // E[[] e]
    AppL(Box<GrabCxt>, LamGrabDelim), // E[v []]
    Del(Box<GrabCxt>),                // E[delimit []] ,
}

pub enum Frame {
    AppR(LamGrabDelim),
    AppL(LamGrabDelim),
    Del(LamGrabDelim),
}

impl GrabCxt {
    pub fn purify(&self) -> Option<GrabPureCxt> {
        match self {
            GrabCxt::Hole => Some(GrabPureCxt::Hole),
            GrabCxt::AppR(f, e) => {
                Some(GrabPureCxt::AppR(Box::new(f.as_ref().purify()?), e.clone()))
            }
            GrabCxt::AppL(f, e) => {
                Some(GrabPureCxt::AppL(Box::new(f.as_ref().purify()?), e.clone()))
            }
            GrabCxt::Del(_) => None,
        }
    }
}

pub enum RdxInFo {
    AbsApp {
        x: Var,
        e: LamGrabDelim,
        v: LamGrabDelim,
    }, // x e1 e2 where e2.is_value()
    DelimVal {
        v: LamGrabDelim,
    }, // e where e.is_value()
    DelimGrab {
        f: GrabPureCxt,
        k: Var,
        e: LamGrabDelim,
    },
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
                    let mut e = e.as_ref().clone();
                    // let mut frames = 
                    todo!()
                }
            }
            _ => None,
        }
    }
}


