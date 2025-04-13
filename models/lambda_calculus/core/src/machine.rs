use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use utils::variable::Var;

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaTerm {
    Variable(Var),
    Abstraction(Var, Box<LambdaTerm>),
    Application(Box<LambdaTerm>, Box<LambdaTerm>),
}

impl From<Var> for LambdaTerm {
    fn from(var: Var) -> Self {
        LambdaTerm::Variable(var)
    }
}

impl LambdaTerm {
    pub fn var(var: Var) -> Self {
        LambdaTerm::Variable(var)
    }
    pub fn abs(var: Var, term: LambdaTerm) -> Self {
        LambdaTerm::Abstraction(var, Box::new(term))
    }
    pub fn app(term1: LambdaTerm, term2: LambdaTerm) -> Self {
        LambdaTerm::Application(Box::new(term1), Box::new(term2))
    }
    pub fn free_variable(&self) -> HashSet<Var> {
        match self {
            LambdaTerm::Variable(var) => vec![var.clone()].into_iter().collect(),
            LambdaTerm::Abstraction(var, term) => {
                let mut set: HashSet<_> = term.free_variable();
                set.remove(var);
                set
            }
            LambdaTerm::Application(term1, term2) => {
                let mut set: HashSet<Var> = term1.free_variable();
                set.extend(term2.free_variable());
                set
            }
        }
    }

    pub fn bounded_variable(&self) -> HashSet<Var> {
        match self {
            LambdaTerm::Variable(_) => HashSet::new(),
            LambdaTerm::Abstraction(var, _) => vec![var.clone()].into_iter().collect(),
            LambdaTerm::Application(term1, term2) => {
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
            LambdaTerm::Variable(var) => write!(f, "[{var}]"),
            LambdaTerm::Abstraction(var, term) => {
                write!(f, "\\{var}.{}", term)
            }
            LambdaTerm::Application(term1, term2) => {
                write!(f, "({} {})", term1, term2)
            }
        }
    }
}

pub fn var_change(var_pre: Var, var_post: Var, term: LambdaTerm) -> LambdaTerm {
    match &term {
        LambdaTerm::Variable(variable) => {
            if var_pre == *variable {
                LambdaTerm::Variable(var_post)
            } else {
                LambdaTerm::Variable(variable.clone())
            }
        }
        LambdaTerm::Abstraction(variable, abs_term) => {
            if var_pre == *variable {
                term
            } else {
                LambdaTerm::Abstraction(
                    variable.clone(),
                    Box::new(var_change(var_pre, var_post, *abs_term.clone())),
                )
            }
        }
        LambdaTerm::Application(app_term1, app_term2) => LambdaTerm::Application(
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

pub fn alpha_eq(term1: &LambdaTerm, term2: &LambdaTerm) -> bool {
    let new_var = {
        let mut set = HashSet::new();
        set.extend(term1.free_variable());
        set.extend(term1.bounded_variable());
        set.extend(term2.free_variable());
        set.extend(term2.bounded_variable());
        utils::variable::VarSet::from(&set).new_var_modify()
    };
    alpha_eq_rec(term1, term2, &HashMap::new(), &HashMap::new(), new_var)
}

fn alpha_eq_rec(
    term1: &LambdaTerm,
    term2: &LambdaTerm,
    corr1: &HashMap<Var, Var>,
    corr2: &HashMap<Var, Var>,
    new_var: Var,
) -> bool {
    match (term1, term2) {
        (LambdaTerm::Variable(var1), LambdaTerm::Variable(var2)) => {
            match (corr1.get(var1), corr2.get(var2)) {
                (Some(var1), Some(var2)) => *var1 == *var2,
                (None, None) => *var1 == *var2,
                _ => false,
            }
        }
        (LambdaTerm::Abstraction(var1, term1), LambdaTerm::Abstraction(var2, term2)) => {
            let mut new_corr1 = corr1.clone();
            new_corr1.insert(var1.clone(), new_var.clone());
            let mut new_corr2 = corr2.clone();
            new_corr2.insert(var2.clone(), new_var.clone());
            alpha_eq_rec(
                term1,
                term2,
                &new_corr1,
                &new_corr2,
                utils::variable::VarSet::from(vec![&new_var]).new_var_modify(),
            )
        }
        (LambdaTerm::Application(term11, term12), LambdaTerm::Application(term21, term22)) => {
            alpha_eq_rec(term11, term21, corr1, corr2, new_var.clone())
                && alpha_eq_rec(term12, term22, corr1, corr2, new_var)
        }
        _ => false,
    }
}

pub fn unchecked_subst(term1: LambdaTerm, var: Var, term2: LambdaTerm) -> LambdaTerm {
    match term1.clone() {
        LambdaTerm::Variable(variable) => {
            if variable == var {
                term2
            } else {
                term1
            }
        }
        LambdaTerm::Abstraction(variable, abs_term) => {
            if variable == var {
                term1
            } else {
                LambdaTerm::Abstraction(variable, Box::new(unchecked_subst(*abs_term, var, term2)))
            }
        }
        LambdaTerm::Application(app_term1, app_term2) => LambdaTerm::Application(
            Box::new(unchecked_subst(*app_term1, var.clone(), term2.clone())),
            Box::new(unchecked_subst(*app_term2, var, term2)),
        ),
    }
}

pub fn subst(term1: LambdaTerm, var: Var, term2: LambdaTerm) -> LambdaTerm {
    match term1 {
        LambdaTerm::Variable(var1) => {
            if var1 == var {
                term2
            } else {
                LambdaTerm::Variable(var1.clone())
            }
        }
        LambdaTerm::Abstraction(var1, term1) => {
            if var1 == var {
                LambdaTerm::Abstraction(var1, term1)
            } else if !term2.free_variable().contains(&var1) {
                LambdaTerm::Abstraction(var1, Box::new(subst(*term1, var, term2)))
            } else {
                let new_var = {
                    let mut set = HashSet::new();
                    set.extend(term1.free_variable());
                    set.extend(term2.free_variable());
                    utils::variable::VarSet::from(&set).new_var_modify()
                };
                LambdaTerm::Abstraction(
                    new_var.clone(),
                    Box::new(subst(
                        subst(*term1, var1, LambdaTerm::Variable(new_var)),
                        var,
                        term2,
                    )),
                )
            }
        }
        LambdaTerm::Application(term_m, term_n) => LambdaTerm::Application(
            Box::new(subst(*term_m, var.clone(), term2.clone())),
            Box::new(subst(*term_n, var, term2)),
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
        let mut term = self.body;
        term = subst(term, self.var, self.arg);
        term
    }
    pub fn from_term(term: &LambdaTerm) -> Option<Self> {
        if let LambdaTerm::Application(term1, term2) = term {
            if let LambdaTerm::Abstraction(var, body) = term1.as_ref() {
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
        LambdaTerm::app(LambdaTerm::abs(redux.var, redux.body), redux.arg)
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
            MarkedTerm::Abstraction(var, term) => {
                LambdaTerm::Abstraction(var, Box::new(term.eval()))
            }
            MarkedTerm::ApplicationL(term1, term2) => {
                LambdaTerm::Application(Box::new(term1.eval()), term2)
            }
            MarkedTerm::ApplicationR(term1, term2) => {
                LambdaTerm::Application(term1, Box::new(term2.eval()))
            }
        }
    }
}

impl MarkedTerm {
    fn eval(self) -> LambdaTerm {
        match self {
            MarkedTerm::Redux(redux) => redux.eval(),
            MarkedTerm::Abstraction(var, term) => {
                LambdaTerm::Abstraction(var, Box::new(term.eval()))
            }
            MarkedTerm::ApplicationL(term1, term2) => {
                LambdaTerm::Application(Box::new(term1.eval()), term2)
            }
            MarkedTerm::ApplicationR(term1, term2) => {
                LambdaTerm::Application(term1, Box::new(term2.eval()))
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
        LambdaTerm::Variable(_) => {}
        LambdaTerm::Abstraction(var, term) => {
            for a in listup_marked_term(term.as_ref()) {
                vec.push(MarkedTerm::Abstraction(var.clone(), Box::new(a)));
            }
        }
        LambdaTerm::Application(term1, term2) => {
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
        let tests = vec![
            (LambdaTerm::var(0.into()), LambdaTerm::var(0.into()), true),
            (LambdaTerm::var(0.into()), LambdaTerm::var(1.into()), false),
            (
                LambdaTerm::abs(0.into(), LambdaTerm::var(0.into())),
                LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                true,
            ),
            (
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::app(LambdaTerm::var(0.into()), LambdaTerm::var(1.into())),
                ),
                LambdaTerm::abs(
                    2.into(),
                    LambdaTerm::app(LambdaTerm::var(2.into()), LambdaTerm::var(1.into())),
                ),
                true,
            ),
            (
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::app(LambdaTerm::var(0.into()), LambdaTerm::var(0.into())),
                ),
                LambdaTerm::abs(
                    2.into(),
                    LambdaTerm::app(LambdaTerm::var(2.into()), LambdaTerm::var(1.into())),
                ),
                false,
            ),
            (
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(0.into())),
                ),
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                ),
                false,
            ),
            (
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::abs(0.into(), LambdaTerm::var(0.into())),
                ),
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                ),
                true,
            ),
            (
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::abs(0.into(), LambdaTerm::var(2.into())),
                ),
                LambdaTerm::abs(
                    0.into(),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(2.into())),
                ),
                true,
            ),
            (
                LambdaTerm::app(
                    LambdaTerm::abs(0.into(), LambdaTerm::var(0.into())),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                ),
                LambdaTerm::app(
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                ),
                true,
            ),
            (
                LambdaTerm::app(
                    LambdaTerm::abs(0.into(), LambdaTerm::var(0.into())),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                ),
                LambdaTerm::app(
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                    LambdaTerm::abs(1.into(), LambdaTerm::var(1.into())),
                ),
                true,
            ),
        ];

        for (term1, term2, b) in tests {
            eprintln!("{:?} {:?}", &term1, &term2);
            assert_eq!(alpha_eq(&term1, &term2), b)
        }
    }
}
