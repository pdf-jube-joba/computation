use crate::{LambdaContext, LambdaExt, State};
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

    impl LambdaContext for Lam {
        type Frame = Frame;
        fn decomp(e: Self) -> Option<(Frame, Self)> {
            match e {
                Lam::Var(_) => None,
                Lam::Lam(_, _) => None,
                Lam::App(e1, e2) => {
                    if e1.is_value().is_some() {
                        Some((Frame::EvalR(*e1), *e2))
                    } else {
                        Some((Frame::EvalL(*e2), *e1))
                    }
                }
            }
        }

        fn plug(frame: Self::Frame, t: Self) -> Self {
            match frame {
                Frame::EvalR(v) => Lam::a(v, t),
                Frame::EvalL(e) => Lam::a(t, e),
            }
        }

        fn step_state(State { mut stack, top }: State<Self>) -> Option<State<Self>> {
            if top.is_value().is_some() {
                if let Some(frame) = stack.pop() {
                    let new_lam = Lam::plug(frame, top);
                    Some(State {
                        stack,
                        top: new_lam,
                    })
                } else {
                    None
                }
            } else if let Some(rdxinfo) = top.is_redex() {
                Some(State {
                    stack,
                    top: Lam::redex_step(rdxinfo),
                })
            } else {
                let (frame, e) = Lam::decomp(top)?;
                stack.push(frame);
                Some(State { stack, top: e })
            }
        }
    }
}

mod ext {
    use super::*;
    use utils::{bool::Bool, number::Number};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Lam {
        Var(Var),
        Lam(Var, Box<Lam>),
        App(Box<Lam>, Box<Lam>),
        Zero,
        Succ(Box<Lam>),
        Pred(Box<Lam>),
        IfZ(Box<Lam>, Box<Lam>, Box<Lam>),
        Let(Var, Box<Lam>, Box<Lam>),
        Rec(Var, Box<Lam>, Box<Lam>),
    }

    impl Display for Lam {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let string = match self {
                Lam::Var(x) => format!("{x}"),
                Lam::Lam(x, e) => format!("\\{x}. {e}"),
                Lam::App(e1, e2) => format!("({e1} @ {e2})"),
                Lam::Zero => format!("0"),
                Lam::Succ(e) => format!("succ {e}"),
                Lam::Pred(e) => format!("pred {e}"),
                Lam::IfZ(e1, e2, e3) => format!("if {e1} then {e2} else {e3}"),
                Lam::Let(x, e1, e2) => format!("let {x} = {e1} in {e2}"),
                Lam::Rec(x, e1, e2) => format!("fix {x} = {e1} in {e2}"),
            };
            write!(f, "{string}")
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Value {
        Num(Number),
        Function(Var, Lam),
    }

    fn num_to_exp(e: Number) -> Lam {
        if e.is_zero() {
            Lam::Zero
        } else {
            Lam::Succ(Box::new(num_to_exp(e - 1)))
        }
    }

    fn exp_to_num(e: &Lam) -> Option<Number> {
        match e {
            Lam::Zero => Some(0.into()),
            Lam::Succ(e) => Some(exp_to_num(&e)? + 1),
            _ => None,
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum RedexInfo {
        App {
            var: Var,
            exp: Lam,
            value: Value,
        },
        Let {
            var: Var,
            value_of_var: Value,
            exp: Lam,
        },
        Rec {
            var: Var,
            fix_exp: Lam,
            exp: Lam,
        },
        Pred {
            n: Num,
        },
    }

    impl LambdaExt for Lam {
        type Value = Value;
        type RedexInfo = RedexInfo;
        fn free_variables(&self) -> HashSet<Var> {
            let mut set = HashSet::new();
            match self {
                Lam::Var(x) => {
                    set.insert(x.clone());
                }
                Lam::Lam(x, e) => {
                    set.extend(e.free_variables());
                    set.remove(x);
                }
                Lam::App(e1, e2) => {
                    set.extend(e1.free_variables());
                    set.extend(e2.free_variables());
                }
                Lam::Zero => {}
                Lam::Succ(e1) => {
                    set.extend(e1.free_variables());
                }
                Lam::Pred(e1) => {
                    set.extend(e1.free_variables());
                }
                Lam::IfZ(e1, e2, e3) => {
                    set.extend(e1.free_variables());
                    set.extend(e2.free_variables());
                    set.extend(e3.free_variables());
                }
                Lam::Let(x, e1, e2) => {
                    set.extend(e1.free_variables());
                    set.extend(e2.free_variables());
                    set.remove(x);
                }
                Lam::Rec(x, e1, e2) => {
                    set.extend(e1.free_variables());
                    set.extend(e2.free_variables());
                    set.remove(x);
                }
            }
            set
        }
        fn bound_variables(&self) -> HashSet<Var> {
            let mut set = HashSet::new();
            match self {
                Lam::Var(x) => {}
                Lam::Lam(x, e) => {
                    set.extend(e.bound_variables());
                    set.insert(x.clone());
                }
                Lam::App(e1, e2) => {
                    set.extend(e1.bound_variables());
                    set.extend(e2.bound_variables());
                }
                Lam::Zero => {}
                Lam::Succ(e1) => {
                    set.extend(e1.bound_variables());
                }
                Lam::Pred(e1) => {
                    set.extend(e1.bound_variables());
                }
                Lam::IfZ(e1, e2, e3) => {
                    set.extend(e1.bound_variables());
                    set.extend(e2.bound_variables());
                    set.extend(e3.bound_variables());
                }
                Lam::Let(x, e1, e2) => {
                    set.extend(e1.bound_variables());
                    set.extend(e2.bound_variables());
                    set.insert(x.clone());
                }
                Lam::Rec(x, e1, e2) => {
                    set.extend(e1.bound_variables());
                    set.extend(e2.bound_variables());
                    set.insert(x.clone());
                }
            }
            set
        }
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
            fn alpha_conversion_canonical_rec(e: Lam, mut vs: variable::VarMap) -> Lam {
                match e {
                    Lam::Var(x) => Lam::Var(vs.get_table(&x)),
                    Lam::Lam(x, e) => {
                        vs.push_var(&x);
                        let new_x = vs.get_table(&x);
                        Lam::Lam(new_x, Box::new(alpha_conversion_canonical_rec(*e, vs)))
                    }
                    Lam::App(e1, e2) => Lam::App(
                        Box::new(alpha_conversion_canonical_rec(*e1, vs.clone())),
                        Box::new(alpha_conversion_canonical_rec(*e2, vs)),
                    ),
                    Lam::Zero => Lam::Zero,
                    Lam::Succ(e) => Lam::Succ(Box::new(alpha_conversion_canonical_rec(*e, vs))),
                    Lam::Pred(e) => Lam::Pred(Box::new(alpha_conversion_canonical_rec(*e, vs))),
                    Lam::IfZ(e1, e2, e3) => Lam::IfZ(
                        Box::new(alpha_conversion_canonical_rec(*e1, vs.clone())),
                        Box::new(alpha_conversion_canonical_rec(*e2, vs.clone())),
                        Box::new(alpha_conversion_canonical_rec(*e3, vs)),
                    ),
                    Lam::Let(x, e1, e2) => {
                        let new_e1 = alpha_conversion_canonical_rec(*e1, vs.clone());
                        let new_x = vs.get_table(&x);
                        let new_e2 = alpha_conversion_canonical_rec(*e2, vs);
                        Lam::Let(new_x, Box::new(new_e1), Box::new(new_e2))
                    }
                    Lam::Rec(x, e1, e2) => {
                        let new_e1 = alpha_conversion_canonical_rec(*e1, vs.clone());
                        let new_x = vs.get_table(&x);
                        let new_e2 = alpha_conversion_canonical_rec(*e2, vs);
                        Lam::Rec(new_x, Box::new(new_e1), Box::new(new_e2))
                    }
                }
            }
            let vs = variable::VarMap::new_iter(self.free_variables().into_iter().chain(vs));
            alpha_conversion_canonical_rec(self, vs)
        }

        fn is_value(&self) -> Option<Self::Value> {
            if let Some(n) = exp_to_num(&self) {
                Some(Value::Num(n))
            } else if let Lam::Lam(x, e) = &self {
                Some(Value::Function(x.clone(), e.as_ref().clone()))
            } else {
                None
            }
        }

        fn value_as_exp(v: Self::Value) -> Self {
            match v {
                Value::Num(n) => num_to_exp(n),
                Value::Function(x, e) => Lam::Lam(x, Box::new(e)),
            }
        }

        fn is_redex(&self) -> Option<Self::RedexInfo> {
            match self {
                Lam::App(e1, e2) => {
                    let Value::Function(x, e) = e1.is_value()? else {
                        return None;
                    };
                    let e2 = e2.is_value()?;
                    Some(RedexInfo::App {
                        var: x,
                        exp: e,
                        value: e2,
                    })
                }
                Lam::Let(x, e1, e2) => {
                    let e1 = e1.is_value()?;
                    Some(RedexInfo::Let {
                        var: x.clone(),
                        value_of_var: e1,
                        exp: e2.as_ref().clone(),
                    })
                }
                Lam::Rec(x, e1, e2) => Some(RedexInfo::Rec {
                    var: x.clone(),
                    fix_exp: e1.as_ref().clone(),
                    exp: e2.as_ref().clone(),
                }),
                Lam::Pred(e) => {
                    let n = exp_to_num(e)?;
                    Some(RedexInfo::Pred { n })
                }
                Lam::Var(_) => todo!(),
                Lam::Lam(_, _) => todo!(),
                Lam::Zero => todo!(),
                Lam::Succ(_) => todo!(),
                Lam::IfZ(_, _, _) => todo!(),
            }
        }

        fn redex_as_exp(r: Self::RedexInfo) -> Self {
            match r {
                RedexInfo::App { var, exp, value } => Lam::App(
                    Box::new(Lam::Lam(var, Box::new(exp))),
                    Box::new(Lam::value_as_exp(value)),
                ),
                RedexInfo::Let {
                    var,
                    value_of_var,
                    exp,
                } => Lam::Let(
                    var,
                    Box::new(Lam::value_as_exp(value_of_var)),
                    Box::new(exp),
                ),
                RedexInfo::Rec { var, fix_exp, exp } => {
                    Lam::Rec(var, Box::new(fix_exp), Box::new(exp))
                }
                RedexInfo::Pred { n } => Lam::Pred(Box::new(num_to_exp(n))),
            }
        }

        fn redex_step(r: Self::RedexInfo) -> Self {
            match r {
                RedexInfo::App { var, exp, value } => {
                    exp.subst(var, value)
                },
                RedexInfo::Let { var, value_of_var, exp } => exp.subst(var, value_of_var),
                RedexInfo::Rec { var, fix_exp, exp } => exp.subst(var, RedexInfo::Rec { var: (), fix_exp: (), exp: () }),
                RedexInfo::Pred { n } => todo!(),
            }
        }

        fn step(self) -> Option<Self> {
            todo!()
        }

        fn subst(self, x: Var, t: Self) -> Self {
            
        }
    }

    // impl LambdaContext for Lam {

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
