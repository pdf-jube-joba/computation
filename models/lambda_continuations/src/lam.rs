use std::fmt::Display;

use utils::variable::Var;

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

fn subst(e: Lam, x: Var, t: Lam) -> Lam {
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

pub fn natural_l2rcbv(l: Lam) -> Option<Lam> {
    match l {
        Lam::Var(_) | Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => match (e1.is_value(), e2.is_value().is_some()) {
            (Some((x, e)), true) => {
                let rdxinfo = RdXInfo { x, e, v: *e2 };
                Some(rdxinfo.step())
            }
            (Some(_), false) => Some(Lam::a(*e1, natural_l2rcbv(*e2)?)),
            (None, _) => Some(Lam::a(natural_l2rcbv(*e1)?, *e2)),
        },
    }
}

pub struct RdXInfo {
    x: Var,
    e: Lam,
    v: Lam, // should  v.is_value()
}

impl RdXInfo {
    pub fn as_lam(self) -> Lam {
        let RdXInfo { x, e, v } = self;
        Lam::a(Lam::l(x, e), v)
    }
    pub fn step(self) -> Lam {
        let RdXInfo { x, e, v } = self;
        subst(e, x, v)
    }
}

impl Lam {
    fn is_value(&self) -> Option<(Var, Lam)> {
        match self {
            Lam::Lam(x, e) => Some((x.clone(), e.as_ref().clone())),
            _ => None,
        }
    }
    fn is_redux(&self) -> Option<RdXInfo> {
        match self {
            Lam::App(e1, e2) => match (e1.is_value(), e2.is_value()) {
                (Some((x, e)), Some(_)) => Some(RdXInfo {
                    x,
                    e,
                    v: e2.as_ref().clone(),
                }),
                _ => None,
            },
            _ => None,
        }
    }
}

// t = ... (v (r e)) ... v: value, r: redex とすると、
// (r, M |-> ... (v (M e)) ... ) と分解する。
#[allow(clippy::type_complexity)]
pub fn decomp_with_cxt_as_func(e: Lam) -> Option<(RdXInfo, Box<dyn Fn(Lam) -> Lam>)> {
    if let Some(rdx) = e.is_redux() {
        let cxt = |lam: Lam| -> Lam { lam };
        return Some((rdx, Box::new(cxt)));
    }
    match e {
        Lam::Var(_) => None,
        Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => {
            if e1.is_value().is_some() {
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
    Some(cxt(rdx.step()))
}

pub enum Cxt {
    Hole,                // []
    AppR(Box<Cxt>, Lam), // E[[] e]
    AppL(Box<Cxt>, Lam), // E[v []]
}

pub enum Frame {
    AppR(Lam), // [] e
    AppL(Lam), // v []
}

impl Frame {
    pub fn plug(self, t: Lam) -> Lam {
        match self {
            Frame::AppL(v) => Lam::a(v, t),
            Frame::AppR(e) => Lam::a(t, e),
        }
    }
}

impl Display for Cxt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Cxt::Hole => "[]".to_string(),
            Cxt::AppR(cxt, e) => format!("{}[[] @ {}]", cxt, e),
            Cxt::AppL(cxt, e) => format!("{}[{} @ []]", cxt, e),
        };
        write!(f, "{}", string)
    }
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

pub fn decomp_with_cxt(t: Lam) -> Option<(RdXInfo, Cxt)> {
    if let Some(rdx) = t.is_redux() {
        return Some((rdx, Cxt::Hole));
    }
    match t {
        Lam::Var(_) => None,
        Lam::Lam(_, _) => None,
        Lam::App(e1, e2) => {
            if e1.is_value().is_some() {
                let (rdx, cxt) = decomp_with_cxt(*e2)?;
                let new_cxt = cxt_rec_hole(cxt, Cxt::AppL(Box::new(Cxt::Hole), *e1));
                Some((rdx, new_cxt))
            } else {
                let (rdx, cxt) = decomp_with_cxt(*e1)?;
                let new_cxt = cxt_rec_hole(cxt, Cxt::AppR(Box::new(Cxt::Hole), *e2));
                Some((rdx, new_cxt))
            }
        }
    }
}

pub fn step_with_cxt(t: Lam) -> Option<Lam> {
    let (rdx, cxt) = decomp_with_cxt(t)?;
    let reduced = rdx.step();
    Some(plug(cxt, reduced))
}

pub struct State {
    stack: Vec<Frame>,
    lam: Lam,
}

pub fn step_machine(State { mut stack, lam }: State) -> Option<State> {
    if lam.is_value().is_some() {
        if let Some(frame) = stack.pop() {
            let new_lam = frame.plug(lam);
            Some(State {
                stack,
                lam: new_lam,
            })
        } else {
            None
        }
    } else if let Some(rdxinfo) = lam.is_redux() {
        Some(State {
            stack,
            lam: rdxinfo.step(),
        })
    } else {
        match lam {
            Lam::Var(_) => None,
            Lam::Lam(_, _) => unreachable!(),
            Lam::App(e1, e2) => {
                if e1.is_value().is_some() {
                    stack.push(Frame::AppL(*e1));
                    Some(State { stack, lam: *e2 })
                } else {
                    stack.push(Frame::AppR(*e2));
                    Some(State { stack, lam: *e1 })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t() {
        // \ z s . z
        let zero_lam = Lam::l(0, Lam::l(1, Lam::v(0)));
        // \ n . \ z s . s (n z s)
        let succ_lam = Lam::l(
            0,
            Lam::l(
                1,
                Lam::l(
                    2,
                    Lam::a(Lam::v(0), Lam::a(Lam::a(Lam::v(1), Lam::v(2)), Lam::v(0))),
                ),
            ),
        );
        let three = Lam::a(
            succ_lam.clone(),
            Lam::a(succ_lam.clone(), Lam::a(succ_lam, zero_lam)),
        );
        let mut lam = three.clone();
        loop {
            println!();
            println!("{}", lam);
            if lam.is_value().is_some() {
                break;
            }
            let (rdx, cxt) = decomp_with_cxt(lam).unwrap();
            println!("- (\\{}. {})@ {}", rdx.x, rdx.e, rdx.v);
            let mut cxt0 = &cxt;
            loop {
                match &cxt0 {
                    Cxt::Hole => break,
                    Cxt::AppR(cxt1, e) => {
                        println!("[] {}", e);
                        cxt0 = cxt1.as_ref();
                    }
                    Cxt::AppL(cxt1, e) => {
                        println!("{} []", e);
                        cxt0 = cxt1.as_ref();
                    }
                }
            }
            let rdx = rdx.step();
            println!("-> {}", rdx);
            lam = plug(cxt, rdx);
        }
    }
}
