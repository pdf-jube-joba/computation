use std::{collections::HashSet, fmt::Display};
use utils::variable::Var;

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaTerm {
    Var(Var),
    Abs(Var, Box<LambdaTerm>),
    App(Box<LambdaTerm>, Box<LambdaTerm>),
}

impl LambdaTerm {
    pub fn free_variable(&self) -> HashSet<Var> {
        match self {
            LambdaTerm::Var(var) => vec![var.clone()].into_iter().collect(),
            LambdaTerm::Abs(var, term) => {
                let mut set: HashSet<_> = term.free_variable();
                set.remove(var);
                set
            }
            LambdaTerm::App(term1, term2) => {
                let mut set: HashSet<Var> = term1.free_variable();
                set.extend(term2.free_variable());
                set
            }
        }
    }

    pub fn bounded_variable(&self) -> HashSet<Var> {
        match self {
            LambdaTerm::Var(_) => HashSet::new(),
            LambdaTerm::Abs(var, _) => vec![var.clone()].into_iter().collect(),
            LambdaTerm::App(term1, term2) => {
                let mut set: HashSet<Var> = term1.bounded_variable();
                set.extend(term2.bounded_variable());
                set
            }
        }
    }
}

impl Display for LambdaTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LambdaTerm::Var(var) => write!(f, "[{var}]"),
            LambdaTerm::Abs(var, term) => {
                write!(f, "\\{var}.{}", term)
            }
            LambdaTerm::App(term1, term2) => {
                write!(f, "({} {})", term1, term2)
            }
        }
    }
}

impl From<Var> for LambdaTerm {
    fn from(var: Var) -> Self {
        LambdaTerm::Var(var)
    }
}

impl From<&Var> for LambdaTerm {
    fn from(var: &Var) -> Self {
        LambdaTerm::Var(var.clone())
    }
}

pub fn var_change(var_pre: Var, var_post: Var, term: LambdaTerm) -> LambdaTerm {
    match &term {
        LambdaTerm::Var(variable) => {
            if var_pre == *variable {
                LambdaTerm::Var(var_post)
            } else {
                LambdaTerm::Var(variable.clone())
            }
        }
        LambdaTerm::Abs(variable, abs_term) => {
            if var_pre == *variable {
                term
            } else {
                LambdaTerm::Abs(
                    variable.clone(),
                    Box::new(var_change(var_pre, var_post, *abs_term.clone())),
                )
            }
        }
        LambdaTerm::App(app_term1, app_term2) => LambdaTerm::App(
            Box::new(var_change(
                var_pre.clone(),
                var_post.clone(),
                *app_term1.clone(),
            )),
            Box::new(var_change(
                var_pre.clone(),
                var_post.clone(),
                *app_term2.clone(),
            )),
        ),
    }
}

pub fn alpha_conversion_rec(term: &LambdaTerm, map: &mut Vec<(Var, Var)>) -> LambdaTerm {
    match term {
        LambdaTerm::Var(var) => {
            if let Some((_, new_var)) = map.iter().find(|(v, _)| v == var) {
                LambdaTerm::Var(new_var.clone())
            } else {
                LambdaTerm::Var(var.clone())
            }
        }
        LambdaTerm::Abs(var, abs_term) => {
            let new_var: Var = var.as_str().into();
            map.push((var.clone(), new_var.clone()));
            let res = LambdaTerm::Abs(new_var, Box::new(alpha_conversion_rec(abs_term, map)));
            map.pop();
            res
        }
        LambdaTerm::App(app_term1, app_term2) => LambdaTerm::App(
            Box::new(alpha_conversion_rec(app_term1, map)),
            Box::new(alpha_conversion_rec(app_term2, map)),
        ),
    }
}

pub fn alpha_eq(term1: &LambdaTerm, term2: &LambdaTerm) -> bool {
    alpha_eq_rec(term1, term2, &mut Vec::new(), &mut Vec::new())
}

fn alpha_eq_rec(
    term1: &LambdaTerm,
    term2: &LambdaTerm,
    corr1: &mut Vec<Var>,
    corr2: &mut Vec<Var>,
) -> bool {
    match (term1, term2) {
        (LambdaTerm::Var(var1), LambdaTerm::Var(var2)) => {
            let idx1 = corr1.iter().position(|v| v == var1);
            let idx2 = corr2.iter().position(|v| v == var2);
            match (idx1, idx2) {
                (Some(i), Some(j)) => i == j,
                (None, None) => var1 == var2,
                _ => false,
            }
        }
        (LambdaTerm::Abs(var1, term1), LambdaTerm::Abs(var2, term2)) => {
            corr1.push(var1.clone());
            corr2.push(var2.clone());
            let res = alpha_eq_rec(term1, term2, corr1, corr2);
            corr1.pop();
            corr2.pop();
            res
        }
        (LambdaTerm::App(term11, term12), LambdaTerm::App(term21, term22)) => {
            alpha_eq_rec(term11, term21, corr1, corr2) && alpha_eq_rec(term12, term22, corr1, corr2)
        }
        _ => false,
    }
}

pub fn unchecked_subst(term1: LambdaTerm, var: Var, term2: LambdaTerm) -> LambdaTerm {
    match term1 {
        LambdaTerm::Var(var1) => {
            if var1 == var {
                term2
            } else {
                LambdaTerm::Var(var1.clone())
            }
        }
        LambdaTerm::Abs(var1, term1) => {
            if var1 == var {
                LambdaTerm::Abs(var1.clone(), term1)
            } else {
                LambdaTerm::Abs(
                    var1.clone(),
                    Box::new(unchecked_subst(*term1, var.clone(), term2.clone())),
                )
            }
        }
        LambdaTerm::App(term_m, term_n) => LambdaTerm::App(
            Box::new(unchecked_subst(*term_m, var.clone(), term2.clone())),
            Box::new(unchecked_subst(*term_n, var, term2)),
        ),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redux {
    pub var: Var,
    pub body: LambdaTerm,
    pub arg: LambdaTerm,
}

impl Redux {
    pub fn eval(self) -> LambdaTerm {
        let new_var: Var = self.var.as_str().into();
        // first, rename bound variable to a new variable to avoid capture
        let mut term = unchecked_subst(
            self.body,
            self.var.clone(),
            LambdaTerm::Var(new_var.clone()),
        );
        // then, substitute the argument for the new variablef
        term = unchecked_subst(term, self.var, self.arg);
        term
    }
    pub fn from_term(term: &LambdaTerm) -> Option<Self> {
        if let LambdaTerm::App(term1, term2) = term {
            if let LambdaTerm::Abs(var, body) = term1.as_ref() {
                Some(Redux {
                    var: var.clone(),
                    body: *body.clone(),
                    arg: *term2.clone(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl From<Redux> for LambdaTerm {
    fn from(redux: Redux) -> Self {
        LambdaTerm::App(
            LambdaTerm::Abs(redux.var, redux.body.into()).into(),
            redux.arg.into(),
        )
    }
}

impl Display for Redux {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(\\{}.{} {})", self.var, self.body, self.arg)
    }
}

// it has exactly one redux in it
#[derive(Debug, Clone, PartialEq)]
pub enum MarkedTerm {
    Redux(Redux),
    Abstraction(Var, Box<MarkedTerm>),
    ApplicationL(Box<MarkedTerm>, Box<LambdaTerm>),
    ApplicationR(Box<LambdaTerm>, Box<MarkedTerm>),
}

impl From<MarkedTerm> for LambdaTerm {
    fn from(marked_term: MarkedTerm) -> Self {
        match marked_term {
            MarkedTerm::Redux(redux) => LambdaTerm::from(redux),
            MarkedTerm::Abstraction(var, term) => LambdaTerm::Abs(var, Box::new(term.eval())),
            MarkedTerm::ApplicationL(term1, term2) => {
                LambdaTerm::App(Box::new(term1.eval()), term2)
            }
            MarkedTerm::ApplicationR(term1, term2) => {
                LambdaTerm::App(term1, Box::new(term2.eval()))
            }
        }
    }
}

impl MarkedTerm {
    fn eval(self) -> LambdaTerm {
        match self {
            MarkedTerm::Redux(redux) => redux.eval(),
            MarkedTerm::Abstraction(var, term) => LambdaTerm::Abs(var, Box::new(term.eval())),
            MarkedTerm::ApplicationL(term1, term2) => {
                LambdaTerm::App(Box::new(term1.eval()), term2)
            }
            MarkedTerm::ApplicationR(term1, term2) => {
                LambdaTerm::App(term1, Box::new(term2.eval()))
            }
        }
    }
}

impl Display for MarkedTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkedTerm::Redux(redux) => write!(f, "{}", redux),
            MarkedTerm::Abstraction(var, term) => write!(f, "\\{var}.{}", term),
            MarkedTerm::ApplicationL(term1, term2) => write!(f, "({} {})", term1, term2),
            MarkedTerm::ApplicationR(term1, term2) => write!(f, "({} {})", term1, term2),
        }
    }
}

pub fn listup_marked_term(term: &LambdaTerm) -> Vec<MarkedTerm> {
    let mut vec = Vec::new();
    if let Some(redux) = Redux::from_term(term) {
        vec.push(MarkedTerm::Redux(redux));
    }
    match term {
        LambdaTerm::Var(_) => {}
        LambdaTerm::Abs(var, term) => {
            for a in listup_marked_term(term.as_ref()) {
                vec.push(MarkedTerm::Abstraction(var.clone(), Box::new(a)));
            }
        }
        LambdaTerm::App(term1, term2) => {
            for a in listup_marked_term(term2.as_ref()) {
                vec.push(MarkedTerm::ApplicationR(term1.clone(), Box::new(a)));
            }
            for a in listup_marked_term(term1.as_ref()) {
                vec.push(MarkedTerm::ApplicationL(Box::new(a), term2.clone()));
            }
        }
    }
    vec
}

pub fn is_normal(term: &LambdaTerm) -> bool {
    listup_marked_term(term).is_empty()
}

pub fn left_most_marked_term(term: LambdaTerm) -> Option<MarkedTerm> {
    listup_marked_term(&term).last().cloned()
}

pub fn left_most_reduction(term: LambdaTerm) -> Option<LambdaTerm> {
    listup_marked_term(&term).last().map(|x| x.clone().eval())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_eq_test() {
        fn v(var: &Var) -> LambdaTerm {
            LambdaTerm::Var(var.clone())
        }
        fn abs(var: &Var, body: LambdaTerm) -> LambdaTerm {
            LambdaTerm::Abs(var.clone(), body.into())
        }
        fn app(lhs: LambdaTerm, rhs: LambdaTerm) -> LambdaTerm {
            LambdaTerm::App(lhs.into(), rhs.into())
        }

        let x = Var::from("x");
        let y = Var::from("y");
        let z = Var::from("z");
        let w = Var::from("w");

        // new Var instances with the same display text are not equal
        let another_x = Var::from("x");
        let another_y = Var::from("y");

        let tests = vec![
            (v(&x), v(&x), true),
            (v(&x), v(&another_x), false),
            (abs(&x, v(&x)), abs(&y, v(&y)), true),
            (abs(&x, app(v(&x), v(&y))), abs(&z, app(v(&z), v(&y))), true),
            (
                abs(&x, app(v(&x), v(&y))),
                abs(&z, app(v(&z), v(&another_y))),
                false,
            ),
            (abs(&x, abs(&y, v(&y))), abs(&z, abs(&w, v(&w))), true),
            (abs(&x, abs(&y, v(&x))), abs(&z, abs(&w, v(&z))), true),
            (abs(&x, abs(&y, v(&x))), abs(&z, abs(&w, v(&w))), false),
            (
                app(abs(&x, v(&x)), abs(&y, v(&y))),
                app(abs(&z, v(&z)), abs(&w, v(&w))),
                true,
            ),
        ];

        for (term1, term2, expected) in tests {
            assert_eq!(alpha_eq(&term1, &term2), expected);
        }
    }
}
