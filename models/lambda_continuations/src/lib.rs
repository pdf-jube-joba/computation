use utils::variable::Var;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
}

impl Lam {
    fn is_value(&self) -> Option<(&Var, &Lam)> {
        match self {
            Lam::Lam(x, e) => Some((x, e)),
            _ => None,
        }
    }
}

pub struct RedInfo {
    x: Var,
    e: Lam,
    v: Lam, // should  v.is_value()
}

pub fn subst(e: Lam, x: Var, t: Lam) -> Lam {
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
                Lam::Lam(y, Box::new(subst(*e, x, t)))
            }
        }
        Lam::App(e1, e2) => Lam::App(
            Box::new(subst(*e1, x.clone(), t.clone())),
            Box::new(subst(*e2, x, t)),
        ),
    }
}

#[allow(clippy::type_complexity)]
pub fn decomp(e: Lam) -> Option<(RedInfo, Box<dyn Fn(Lam) -> Lam>)> {
    match e {
        Lam::Var(_) => None,
        Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => match (e1.is_value(), e2.is_value()) {
            (Some((x1, e1)), Some(_)) => {
                let cxt = |lam: Lam| -> Lam { lam };
                let rdx = RedInfo {
                    x: x1.clone(),
                    e: e1.clone(),
                    v: *e2,
                };
                Some((rdx, Box::new(cxt)))
            }
            (Some(_), None) => {
                let (rdx, cxt) = decomp(*e2)?;
                let cxt = move |lam: Lam| -> Lam {
                    let lam = cxt(lam);
                    Lam::App(e1.clone(), Box::new(lam))
                };
                Some((rdx, Box::new(cxt)))
            }
            (None, _) => {
                let (rdx, cxt) = decomp(*e1)?;
                let cxt = move |lam: Lam| -> Lam {
                    let lam = cxt(lam);
                    Lam::App(Box::new(lam), e2.clone())
                };
                Some((rdx, Box::new(cxt)))
            }
        },
    }
}

pub fn step(e: Lam) -> Option<Lam> {
    let (RedInfo { x, e, v }, cxt) = decomp(e)?;
    let reduced = subst(e, x, v);
    Some(cxt(reduced))
}

pub enum Cxt {
    Hole,                // []
    AppR(Box<Cxt>, Lam), // E[[] e]
    AppL(Box<Cxt>, Lam), // E[v []]
}

pub fn plug(cxt: Cxt, t: Lam) -> Lam {
    match cxt {
        Cxt::Hole => t,
        Cxt::AppL(cxt, e) => plug(*cxt, Lam::App(Box::new(e), Box::new(t))),
        Cxt::AppR(cxt, e) => plug(*cxt, Lam::App(Box::new(t), Box::new(e))),
    }
}

pub fn cxt_rec_hole(cxt: Cxt, cxt2: Cxt) -> Cxt {
    match cxt {
        Cxt::Hole => cxt2,
        Cxt::AppR(cxt, e) => Cxt::AppR(Box::new(cxt_rec_hole(*cxt, cxt2)), e),
        Cxt::AppL(cxt, v) => Cxt::AppL(Box::new(cxt_rec_hole(*cxt, cxt2)), v),
    }
}

pub fn decomp1(t: Lam) -> Option<(RedInfo, Cxt)> {
    match t {
        Lam::Var(_) => None,
        Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => match (e1.is_value(), e2.is_value()) {
            // v_1 v_2
            (Some((x1, e1)), Some(_)) => {
                let rdx = RedInfo {
                    x: x1.clone(),
                    e: e1.clone(),
                    v: *e2,
                };
                Some((rdx, Cxt::Hole))
            }
            // v_1 e_2
            (Some(_), None) => {
                let (rdx, cxt) = decomp1(*e2)?;
                let new_cxt = cxt_rec_hole(cxt, Cxt::AppL(Box::new(Cxt::Hole), *e1));
                Some((rdx, new_cxt))
            }
            // e_1 e_2
            (None, _) => {
                let (rdx, cxt) = decomp1(*e1)?;
                let new_cxt = cxt_rec_hole(cxt, Cxt::AppR(Box::new(Cxt::Hole), *e2));
                Some((rdx, new_cxt))
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
    AppR(Box<GrabPureCxt>, LamGrabDelim), // E[[] e]
    AppL(Box<GrabPureCxt>, LamGrabDelim), // E[v []]
}

pub enum GrabCxt {
    Hole,
    AppR(Box<GrabCxt>, LamGrabDelim), // E[[] e]
    AppL(Box<GrabCxt>, LamGrabDelim), // E[v []]
    Del(Box<GrabCxt>),                // E[delimit []] ,
}

pub enum LamSendRun {
    Var(Var),
    Lam(Var, Box<LamSendRun>),
    App(Box<LamSendRun>, Box<LamSendRun>),
    Send(Box<LamSendRun>),
    Run(Box<LamSendRun>, Var, Box<LamSendRun>),
}

pub enum SendPureCxt {
    Hole,
    AppR(Box<SendPureCxt>, LamSendRun), // E[[] e]
    AppL(Box<SendPureCxt>, LamSendRun), // E[v []]
}

pub enum SendCxt {
    Hole,
    AppR(Box<SendCxt>, LamSendRun), // E[[] e]
    AppL(Box<SendCxt>, LamSendRun), // E[v []]
    Del(Box<SendCxt>),              // E[send []] ,
}

pub enum LamEffect {
    Var(Var),
    Lam(Var, Box<LamEffect>),
    App(Box<LamEffect>, Box<LamEffect>),
    Op(String, Box<LamEffect>),
    Handle(Box<LamEffect>, Handlers),
}

pub enum EffPureCxt {
    Hole,
    AppR(Box<EffPureCxt>, LamEffect), // E[[] e]
    AppL(Box<EffPureCxt>, LamEffect), // E[v []]
}

pub enum EffCxt {
    Hole,
    AppR(Box<EffCxt>, LamEffect), // E[[] e]
    AppL(Box<EffCxt>, LamEffect), // E[v []]
    Del(Box<EffCxt>),             // E[op []] ,
}

pub struct Handlers(Vec<(String, Var, Var, Box<LamEffect>)>);
