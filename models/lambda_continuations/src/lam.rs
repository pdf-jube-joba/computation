use crate::{LambdaContext, LambdaExt};
use std::{collections::HashSet, fmt::Display};
use utils::variable::{self, Var};

pub mod nat {
    use super::*;

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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RedexInfo {
        x: Var,
        e: Lam,
        v: Value, // should  v.is_value()
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

        fn is_value(&self) -> Option<Self::Value> {
            match self {
                Lam::Lam(v, e) => Some(Value::Function(v.clone(), e.clone())),
                _ => None,
            }
        }

        fn value_as_exp(v: Self::Value) -> Self {
            let Value::Function(x, e) = v;
            Lam::Lam(x, e)
        }

        fn is_redex(&self) -> Option<RedexInfo> {
            match self {
                Lam::App(e1, e2) => match (e1.is_value(), e2.is_value()) {
                    (Some(Value::Function(x, e)), Some(v)) => Some(RedexInfo { x, e: *e, v }),
                    _ => None,
                },
                _ => None,
            }
        }

        fn redex_as_exp(r: Self::RedexInfo) -> Self {
            let RedexInfo { x, e, v } = r;
            Lam::a(Lam::l(x, e), Lam::value_as_exp(v))
        }

        fn redex_step(r: Self::RedexInfo) -> Self {
            let RedexInfo { x, e, v } = r;
            e.subst(x, Lam::value_as_exp(v))
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

        fn step(self) -> Option<Self> {
            match self {
                Lam::Var(_) | Lam::Lam(_, _) => None,
                Lam::App(e1, e2) => match (e1.is_value(), e2.is_value().is_some()) {
                    (Some(Value::Function(x, e)), true) => Some(e.subst(x, *e2)),
                    (Some(_), false) => Some(Lam::a(*e1, e2.step()?)),
                    (None, _) => Some(Lam::a(e1.step()?, *e2)),
                },
            }
        }
    }

    // t = ... (v (r e)) ... v: value, r: redex とすると、
    // (r, M |-> ... (v (M e)) ... ) と分解する。
    #[allow(clippy::type_complexity)]
    pub fn decomp_with_cxt_as_func(e: Lam) -> Option<(RedexInfo, Box<dyn Fn(Lam) -> Lam>)> {
        if let Some(rdx) = e.is_redex() {
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
        Some(cxt(Lam::redex_step(rdx)))
    }

    pub enum Cxt {
        Hole,                 // []
        EvalL(Lam, Box<Cxt>), // E[[] e]
        EvalR(Lam, Box<Cxt>), // E[v []]
    }

    impl Cxt {
        pub fn plug(self, t: Lam) -> Lam {
            match self {
                Cxt::Hole => t,
                Cxt::EvalR(e, cxt) => cxt.plug(Lam::a(e, t)),
                Cxt::EvalL(e, cxt) => cxt.plug(Lam::a(t, e)),
            }
        }
        pub fn extend_r(self, e: Lam) -> Self {
            match self {
                Cxt::Hole => Cxt::EvalR(e, Box::new(Cxt::Hole)),
                Cxt::EvalL(e1, c) => Cxt::EvalL(e1, Box::new(c.extend_r(e))),
                Cxt::EvalR(e1, c) => Cxt::EvalR(e1, Box::new(c.extend_r(e))),
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

    pub enum Frame {
        EvalL(Lam), // [] e
        EvalR(Lam), // v []
    }

    impl Frame {
        pub fn plug(self, t: Lam) -> Lam {
            match self {
                Frame::EvalR(v) => Lam::a(v, t),
                Frame::EvalL(e) => Lam::a(t, e),
            }
        }
    }

    impl Display for Cxt {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let string = match self {
                Cxt::Hole => "[]".to_string(),
                Cxt::EvalL(e, cxt) => format!("{}[[] @ {}]", e, cxt),
                Cxt::EvalR(e, cxt) => format!("{}[{} @ []]", e, cxt),
            };
            write!(f, "{}", string)
        }
    }

    pub fn decomp_with_cxt(t: Lam) -> Option<(RedexInfo, Cxt)> {
        if let Some(rdx) = Lam::is_redex(&t) {
            return Some((rdx, Cxt::Hole));
        }
        match t {
            Lam::Var(_) => None,
            Lam::Lam(_, _) => None,
            Lam::App(e1, e2) => {
                if e1.is_value().is_some() {
                    let (rdx, cxt) = decomp_with_cxt(*e2)?;
                    let new_cxt = cxt.extend_l(*e1);
                    Some((rdx, new_cxt))
                } else {
                    let (rdx, cxt) = decomp_with_cxt(*e1)?;
                    let new_cxt = cxt.extend_r(*e2);
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
        } else if let Some(rdxinfo) = lam.is_redex() {
            Some(State {
                stack,
                lam: Lam::redex_step(rdxinfo),
            })
        } else {
            match lam {
                Lam::Var(_) => None,
                Lam::Lam(_, _) => unreachable!(),
                Lam::App(e1, e2) => {
                    if e1.is_value().is_some() {
                        stack.push(Frame::EvalR(*e1));
                        Some(State { stack, lam: *e2 })
                    } else {
                        stack.push(Frame::EvalL(*e2));
                        Some(State { stack, lam: *e1 })
                    }
                }
            }
        }
    }
}

mod ext {
    use utils::bool::Bool;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Lam {
        Var(Var),
        Lam(Var, Box<Lam>),
        App(Box<Lam>, Box<Lam>),
        Zero,
        Succ(Box<Lam>),
        Pred(Box<Lam>),
        Cst(Bool),
        If(Box<Lam>, Box<Lam>, Box<Lam>),
        EqZero(Box<Lam>),
        Let(Var, Box<Lam>, Box<Lam>),
        Fix(Var, Box<Lam>),
    }

    // impl Lam {
    //     pub fn is_value(&self) ->
    // }

    // pub fn step(t: Lam) -> Lam {
    //     match t {
    //         Lam::Var(x) => Var(x),
    //         Lam::Lam(x, e) => Lam::Lam(x, v),
    //         Lam::Zero => Lam::Zero,
    //     }
    // }

    pub enum Frame {}
}

#[cfg(test)]
mod tests {
    use super::nat::*;
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
            println!("- {:?}", rdx);
            let mut cxt0 = &cxt;
            loop {
                match &cxt0 {
                    Cxt::Hole => break,
                    Cxt::EvalL(e, cxt1) => {
                        println!("[] {}", e);
                        cxt0 = cxt1.as_ref();
                    }
                    Cxt::EvalR(e, cxt1) => {
                        println!("{} []", e);
                        cxt0 = cxt1.as_ref();
                    }
                }
            }
            let rdx = Lam::redex_step(rdx);
            println!("-> {}", rdx);
            lam = cxt.plug(rdx);
        }
    }
}
