use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var(usize);

impl Var {
    fn next_new(&self) -> Var {
        Var(self.0 + 1)
    }
}

impl ToString for Var {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<usize> for Var {
    fn from(value: usize) -> Self {
        Var(value)
    }
}

fn new_var(set: &HashSet<Var>) -> Var {
    Var(set.iter().map(|var| var.0).max().unwrap_or_default() + 1)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaTerm {
    Variable(Var),
    Abstraction(Var, Box<LambdaTerm>),
    Application(Box<LambdaTerm>, Box<LambdaTerm>),
}

impl LambdaTerm {
    pub fn var(i: usize) -> Self {
        LambdaTerm::Variable(i.into())
    }
    pub fn abs(i: usize, term: LambdaTerm) -> Self {
        LambdaTerm::Abstraction(Var(i), Box::new(term))
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
                set.extend(term2.free_variable().into_iter());
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
                set.extend(term2.bounded_variable().into_iter());
                set
            }
        }
    }
}

impl ToString for LambdaTerm {
    fn to_string(&self) -> String {
        match self {
            LambdaTerm::Variable(var) => var.to_string(),
            LambdaTerm::Abstraction(var, term) => {
                "\\".to_owned() + &var.to_string() + "." + &*term.to_string()
            }
            LambdaTerm::Application(term1, term2) => {
                "(".to_owned() + &term1.to_string() + " " + &term2.to_string() + ")"
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
            alpha_eq_rec(term1, term2, &new_corr1, &new_corr2, new_var.next_new())
        }
        (LambdaTerm::Application(term11, term12), LambdaTerm::Application(term21, term22)) => {
            alpha_eq_rec(term11, term21, corr1, corr2, new_var.clone())
                && alpha_eq_rec(term12, term22, corr1, corr2, new_var)
        }
        _ => false,
    }
}

pub fn alpha_eq(term1: &LambdaTerm, term2: &LambdaTerm) -> bool {
    let new_var = {
        let mut set = HashSet::new();
        set.extend(term1.free_variable().into_iter());
        set.extend(term1.bounded_variable().into_iter());
        set.extend(term2.free_variable().into_iter());
        set.extend(term2.bounded_variable().into_iter());
        new_var(&set)
    };
    alpha_eq_rec(term1, term2, &HashMap::new(), &HashMap::new(), new_var)
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
                    set.extend(term1.free_variable().into_iter());
                    set.extend(term2.free_variable().into_iter());
                    new_var(&set)
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

pub fn is_beta_redux(term: &LambdaTerm) -> bool {
    match term {
        LambdaTerm::Application(term, _) => {
            matches!(term.as_ref(), LambdaTerm::Abstraction(_, _))
        }
        _ => false,
    }
}

pub fn is_normal(term: &LambdaTerm) -> bool {
    if is_beta_redux(term) {
        return false;
    }
    match term {
        LambdaTerm::Variable(_) => true,
        LambdaTerm::Abstraction(_, term) => is_normal(term.as_ref()),
        LambdaTerm::Application(term1, term2) => {
            is_normal(term1.as_ref()) && is_normal(term2.as_ref())
        }
    }
}

pub fn unchecked_beta_redux_reduce(term: LambdaTerm) -> LambdaTerm {
    match term {
        LambdaTerm::Application(term, term2) => match *term {
            LambdaTerm::Abstraction(var, term1) => subst(*term1, var, *term2),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

pub fn list_up_reduce(term: LambdaTerm) -> Vec<LambdaTerm> {
    let mut vec = Vec::new();
    if is_beta_redux(&term) {
        vec.push(unchecked_beta_redux_reduce(term.clone()))
    }
    match term {
        LambdaTerm::Variable(_) => {}
        LambdaTerm::Abstraction(var, term) => {
            vec.extend(
                list_up_reduce(*term)
                    .into_iter()
                    .map(|term| LambdaTerm::Abstraction(var.clone(), Box::new(term))),
            );
        }
        LambdaTerm::Application(term1, term2) => {
            vec.extend(
                list_up_reduce(*term1.clone())
                    .into_iter()
                    .map(|term| LambdaTerm::Application(Box::new(term), Box::new(*term2.clone()))),
            );
            vec.extend(
                list_up_reduce(*term2)
                    .into_iter()
                    .map(|term| LambdaTerm::Application(Box::new(*term1.clone()), Box::new(term))),
            );
        }
    }
    vec
}

pub fn left_most_reduction(term: LambdaTerm) -> LambdaTerm {
    let list = list_up_reduce(term);
    list[0].clone()
}

pub fn left_most_reduction_step(term: LambdaTerm, step: usize) -> LambdaTerm {
    let mut term = term;
    for _ in 0..step {
        term = left_most_reduction(term);
    }
    term
}

pub mod utils {
    use super::alpha_eq;
    use super::LambdaTerm;

    pub fn true_lambda() -> LambdaTerm {
        LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(0)))
    }

    pub fn false_lambda() -> LambdaTerm {
        LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(1)))
    }

    pub fn is_true(term: LambdaTerm) -> bool {
        let l = true_lambda();
        alpha_eq(&l, &term)
    }

    pub fn is_false(term: LambdaTerm) -> bool {
        let l = false_lambda();
        alpha_eq(&l, &term)
    }

    pub fn if_lambda(l: LambdaTerm, m: LambdaTerm, n: LambdaTerm) -> LambdaTerm {
        LambdaTerm::app(LambdaTerm::app(l, m), n)
    }

    pub fn pair(m: LambdaTerm, n: LambdaTerm) -> LambdaTerm {
        LambdaTerm::abs(
            0,
            LambdaTerm::app(LambdaTerm::app(LambdaTerm::var(0), m), n),
        )
    }

    pub fn first() -> LambdaTerm {
        LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(0), true_lambda()))
    }

    pub fn second() -> LambdaTerm {
        LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(0), false_lambda()))
    }

    pub fn take_n_abs(list: Vec<usize>, term: LambdaTerm) -> LambdaTerm {
        if let Some((head, tail)) = list.split_first() {
            LambdaTerm::abs(*head, take_n_abs(tail.to_owned(), term))
        } else {
            term
        }
    }

    pub fn fold_left(list: Vec<LambdaTerm>) -> LambdaTerm {
        list.into_iter().reduce(LambdaTerm::app).unwrap()
    }

    pub fn y_combinator() -> LambdaTerm {
        LambdaTerm::abs(
            0,
            LambdaTerm::app(
                LambdaTerm::abs(
                    1,
                    LambdaTerm::app(
                        LambdaTerm::var(0),
                        LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(1)),
                    ),
                ),
                LambdaTerm::abs(
                    1,
                    LambdaTerm::app(
                        LambdaTerm::var(0),
                        LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(1)),
                    ),
                ),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_eq_test() {
        let tests = vec![
            (LambdaTerm::var(0), LambdaTerm::var(0), true),
            (LambdaTerm::var(0), LambdaTerm::var(1), false),
            (
                LambdaTerm::abs(0, LambdaTerm::var(0)),
                LambdaTerm::abs(1, LambdaTerm::var(1)),
                true,
            ),
            (
                LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::var(1))),
                LambdaTerm::abs(2, LambdaTerm::app(LambdaTerm::var(2), LambdaTerm::var(1))),
                true,
            ),
            (
                LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::var(0))),
                LambdaTerm::abs(2, LambdaTerm::app(LambdaTerm::var(2), LambdaTerm::var(1))),
                false,
            ),
            (
                LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(0))),
                LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(1))),
                false,
            ),
            (
                LambdaTerm::abs(0, LambdaTerm::abs(0, LambdaTerm::var(0))),
                LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(1))),
                true,
            ),
            (
                LambdaTerm::abs(0, LambdaTerm::abs(0, LambdaTerm::var(2))),
                LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(2))),
                true,
            ),
            (
                LambdaTerm::app(
                    LambdaTerm::abs(0, LambdaTerm::var(0)),
                    LambdaTerm::abs(1, LambdaTerm::var(1)),
                ),
                LambdaTerm::app(
                    LambdaTerm::abs(1, LambdaTerm::var(1)),
                    LambdaTerm::abs(1, LambdaTerm::var(1)),
                ),
                true,
            ),
            (
                LambdaTerm::app(
                    LambdaTerm::abs(0, LambdaTerm::var(0)),
                    LambdaTerm::abs(1, LambdaTerm::var(1)),
                ),
                LambdaTerm::app(
                    LambdaTerm::abs(1, LambdaTerm::var(1)),
                    LambdaTerm::abs(1, LambdaTerm::var(1)),
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
