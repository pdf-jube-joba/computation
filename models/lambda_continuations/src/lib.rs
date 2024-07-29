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
            Lam1::Lam(x, e) => Some((x, e)),
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
    match e {
        Lam1::Var(y) => {
            if x == y {
                t
            } else {
                Lam1::Var(y)
            }
        }
        Lam1::Lam(y, e) => {
            if x == y {
                Lam1::Lam(y, e)
            } else {
                Lam1::Lam(y, Box::new(subst(*e, x, t)))
            }
        }
        Lam1::App(e1, e2) => Lam1::App(
            Box::new(subst(*e1, x.clone(), t.clone())),
            Box::new(subst(*e2, x, t)),
        ),
    }
}

#[allow(clippy::type_complexity)]
pub fn decomp(e: Lam1) -> Option<(RedInfo, Box<dyn Fn(Lam1) -> Lam1>)> {
    match e {
        Lam1::Var(_) => None,
        Lam1::Lam(_, _) => None,
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

pub fn step(e: Lam1) -> Option<Lam1> {
    let (RedInfo { x, e, v }, cxt) = decomp(e)?;
    let reduced = subst(e, x, v);
    Some(cxt(reduced))
}

pub enum Cxt {
    Hole,                 // []
    AppR(Box<Cxt>, Lam1), // E[e []]
    AppL(Box<Cxt>, Lam1), // E[[] v]
}

pub fn plug(cxt: Cxt, t: Lam1) -> Lam1 {
    match cxt {
        Cxt::Hole => t,
        Cxt::AppL(cxt, e) => plug(*cxt, Lam1::App(Box::new(e), Box::new(t))),
        Cxt::AppR(cxt, e) => plug(*cxt, Lam1::App(Box::new(t), Box::new(e))),
    }
}

pub fn decomp1(t: Lam1) -> Option<(RedInfo, Cxt)> {
    match t {
        Lam1::Var(_) => None,
        Lam1::Lam(_, _) => None,
        Lam1::App(e1, e2) => match (e1.is_value(), e2.is_value()) {
            (Some((x1, e1)), Some(_)) => {
                let rdx = RedInfo {
                    x: x1.clone(),
                    e: e1.clone(),
                    v: *e2,
                };
                Some((rdx, Cxt::Hole))
            }
            // 違う気がする。
            (Some(_), None) => {
                let (rdx, cxt) = decomp1(*e2)?;
                Some((rdx, Cxt::AppR(Box::new(cxt), *e1)))
            }
            (None, _) => {
                let (rdx, cxt) = decomp1(*e1)?;
                Some((rdx, Cxt::AppL(Box::new(cxt), *e2)))
            }
        },
    }
}

pub enum LamGrabDelim {
    Var(Var),
    Lam(Var, Box<LamGrabDelim>),
    App(Box<LamGrabDelim>, Box<LamGrabDelim>),
    Grab(Var, Box<LamGrabDelim>),
    Delim(Box<LamGrabDelim>),
}

pub enum GrabPureCxt {
    Hole,
    AppR(Box<GrabPureCxt>, Lam1), // E[e []]
    AppL(Box<GrabPureCxt>, Lam1), // E[[] v]
}

pub enum GrabCxt {
    Hole,
    AppR(Box<GrabCxt>, Lam1), // E[e []]
    AppL(Box<GrabCxt>, Lam1), // E[[] v]
    Del(Box<GrabCxt>), // E[delimit []] ,
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

pub struct Handlers(Vec<(String, Var, Var, Box<LamEffect>)>);
