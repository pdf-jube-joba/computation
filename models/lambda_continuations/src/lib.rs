use either::Either;
use utils::variable::Var;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam1 {
    Var(Var),
    Lam(Var, Box<Lam1>),
    App(Box<Lam1>, Box<Lam1>),
}

impl Lam1 {
    fn is_value(&self) -> Option<(&Var, &Lam1)> {
        match self {
            Lam1::Lam(x, e) => Some((&x, &e)),
            _ => None,
        }
    }
}

pub struct RedInfo {
    x: Var,
    e: Lam1,
    v: Lam1, // should  v.is_value()
}

pub fn subst(e: Lam1, x: Var, t: Lam1) -> Lam1 {
    todo!()
}

pub fn decomp(e: Lam1) -> Option<(RedInfo, Box<dyn Fn(Lam1) -> Lam1>)> {
    match e {
        Lam1::Var(x) => None,
        Lam1::Lam(x, e) => None,
        Lam1::App(e1, e2) => match (e1.is_value(), e2.is_value()) {
            (Some((x1, e1)), Some(_)) => {
                let cxt = |lam: Lam1| -> Lam1 { lam };
                let rdx = RedInfo {
                    x: x1.clone(),
                    e: e1.clone(),
                    v: *e2,
                };
                Some((rdx, Box::new(cxt)))
            }
            (Some(_), None) => {
                let (rdx, cxt) = decomp(*e2)?;
                let cxt = move |lam: Lam1| -> Lam1 {
                    let lam = cxt(lam);
                    Lam1::App(e1.clone(), Box::new(lam))
                };
                Some((rdx, Box::new(cxt)))
            }
            (None, _) => {
                let (rdx, cxt) = decomp(*e1)?;
                let cxt = move |lam: Lam1| -> Lam1 {
                    let lam = cxt(lam);
                    Lam1::App(Box::new(lam), e2.clone())
                };
                Some((rdx, Box::new(cxt)))
            }
        },
    }
}

pub enum Cxt {
    Hole,                 // []
    AppR(Box<Cxt>, Lam1), // E[e []]
    AppL(Box<Cxt>, Lam1), // E[[] v]
}

pub fn decomp1(t: Lam1) -> Option<Either<Lam1, (Lam1, RedInfo)>> {
    match &t {
        Lam1::Var(_) => None,
        Lam1::Lam(x, v) => Some(Either::Left(t)),
        Lam1::App(e1, e2) => {
            todo!()
        }
    }
}

pub enum LamGrabDelim {
    Var(Var),
    Lam(Var, Box<LamGrabDelim>),
    App(Box<LamGrabDelim>, Box<LamGrabDelim>),
    Grab(Var, Box<LamGrabDelim>),
    Delim(Box<LamGrabDelim>),
}

pub enum LamSendRun {
    Var(Var),
    Lam(Var, Box<LamSendRun>),
    App(Box<LamSendRun>, Box<LamSendRun>),
    Send(Box<LamSendRun>),
    Run(Box<LamSendRun>, Var, Box<LamSendRun>),
}

pub enum LamEffect {
    Var(Var),
    Lam(Var, Box<LamEffect>),
    App(Box<LamEffect>, Box<LamEffect>),
    Op(String, Box<LamEffect>),
    Handle(Box<LamEffect>, Handlers),
}

pub struct Handlers(Vec<(Var, Var, Box<LamEffect>)>);
