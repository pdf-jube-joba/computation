use std::{collections::HashSet, fmt::Display};
use serde::Serialize;
use utils::variable::Var;

#[derive(Debug, Clone, PartialEq, Serialize)]
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

// this enumerates all redexes in a term
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MarkedTerm {
    Var(Var),
    Abs(Var, Box<MarkedTerm>),
    App(Box<MarkedTerm>, Box<MarkedTerm>),
    Red(Var, Box<MarkedTerm>, Box<MarkedTerm>),
}

impl From<MarkedTerm> for LambdaTerm {
    fn from(marked_term: MarkedTerm) -> Self {
        match marked_term {
            MarkedTerm::Var(var) => LambdaTerm::Var(var),
            MarkedTerm::Abs(var, abs_term) => LambdaTerm::Abs(var, Box::new((*abs_term).into())),
            MarkedTerm::App(app_term1, app_term2) => {
                LambdaTerm::App(Box::new((*app_term1).into()), Box::new((*app_term2).into()))
            }
            MarkedTerm::Red(var, abs_term, app_term) => LambdaTerm::App(
                Box::new(LambdaTerm::Abs(var, Box::new((*abs_term).into()))),
                Box::new((*app_term).into()),
            ),
        }
    }
}

pub fn mark_redex(term: &LambdaTerm) -> MarkedTerm {
    match term {
        LambdaTerm::Var(var) => MarkedTerm::Var(var.clone()),
        LambdaTerm::Abs(var, abs_term) => {
            MarkedTerm::Abs(var.clone(), Box::new(mark_redex(abs_term.as_ref())))
        }
        LambdaTerm::App(app_term1, app_term2) => match app_term1.as_ref() {
            LambdaTerm::Abs(var, abs_term) => MarkedTerm::Red(
                var.clone(),
                Box::new(mark_redex(abs_term.as_ref())),
                Box::new(mark_redex(app_term2.as_ref())),
            ),
            _ => MarkedTerm::App(
                Box::new(mark_redex(app_term1.as_ref())),
                Box::new(mark_redex(app_term2.as_ref())),
            ),
        },
    }
}

pub fn unmark_redex(marked_term: MarkedTerm) -> LambdaTerm {
    match marked_term {
        MarkedTerm::Var(var) => LambdaTerm::Var(var),
        MarkedTerm::Abs(var, abs_term) => {
            LambdaTerm::Abs(var, Box::new(unmark_redex(*abs_term)))
        }
        MarkedTerm::App(app_term1, app_term2) => LambdaTerm::App(
            Box::new(unmark_redex(*app_term1)),
            Box::new(unmark_redex(*app_term2)),
        ),
        MarkedTerm::Red(var, abs_term, app_term) => LambdaTerm::App(
            Box::new(LambdaTerm::Abs(var, Box::new(unmark_redex(*abs_term)))),
            Box::new(unmark_redex(*app_term)),
        ),
    }
}

// step function that reduces the nth redex in the marked term
// how to count the redexes: left to right, depth-first
// return None if there is no nth redex
pub fn step(marked_term: &MarkedTerm, num: usize) -> Option<LambdaTerm> {
    fn step_rec(marked_term: &MarkedTerm, num: &mut isize) -> Option<LambdaTerm> {
        match marked_term {
            MarkedTerm::Var(var) => Some(LambdaTerm::Var(var.clone())),
            MarkedTerm::Abs(var, abs_term) => {
                let body = step_rec(abs_term.as_ref(), num)?;
                Some(LambdaTerm::Abs(var.clone(), Box::new(body)))
            }
            MarkedTerm::App(app_term1, app_term2) => {
                let left = step_rec(app_term1.as_ref(), num)?;
                let right = step_rec(app_term2.as_ref(), num)?;
                Some(LambdaTerm::App(Box::new(left), Box::new(right)))
            }
            MarkedTerm::Red(var, abs_term, app_term) => {
                if *num == 0 {
                    // reduce this redex
                    // 1. before substitution, rename bound variables in abs_term to avoid capture
                    let new_var = Var::from(format!("{}'", var.as_str()));

                    let avoided: LambdaTerm = var_change(
                        var.clone(),
                        new_var.clone(),
                        abs_term.as_ref().clone().into(),
                    );
                    let body = unchecked_subst(avoided, new_var, app_term.as_ref().clone().into());
                    Some(body)
                } else {
                    *num -= 1;
                    let left = step_rec(abs_term.as_ref(), num)?;
                    let right = step_rec(app_term.as_ref(), num)?;
                    Some(LambdaTerm::App(
                        Box::new(LambdaTerm::Abs(var.clone(), Box::new(left))),
                        Box::new(right),
                    ))
                }
            }
        }
    }

    let mut n = num as isize;
    let exp = step_rec(marked_term, &mut n);
    if n <= 0 {
        exp
    } else {
        None
    }
}

pub fn is_normal_form(term: &LambdaTerm) -> bool {
    match term {
        LambdaTerm::Var(_) => true,
        LambdaTerm::Abs(_, abs_term) => is_normal_form(abs_term.as_ref()),
        LambdaTerm::App(app_term1, app_term2) => match app_term1.as_ref() {
            LambdaTerm::Abs(_, _) => false,
            _ => is_normal_form(app_term1.as_ref()) && is_normal_form(app_term2.as_ref()),
        },
    }
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
    #[test]
    fn step_test() {
        // (\x. x) y
        let x_var = Var::from("x");
        let y_var = Var::from("y");
        let e =  LambdaTerm::App(
            Box::new(LambdaTerm::Abs(
                x_var.clone(),
                Box::new(LambdaTerm::Var(x_var.clone())),
            )),
            Box::new(LambdaTerm::Var(y_var.clone())),
        );
        let marked = mark_redex(&e);
        let stepped = step(&marked, 0).unwrap();
        let expected = LambdaTerm::Var(y_var.clone());
        assert_eq!(stepped, expected);

        // (\x. x) ((\y. y) z)
        let z_var = Var::from("z");
        let e = LambdaTerm::App(
            Box::new(LambdaTerm::Abs(
                x_var.clone(),
                Box::new(LambdaTerm::Var(x_var.clone())),
            )),
            Box::new(LambdaTerm::App(
                Box::new(LambdaTerm::Abs(
                    y_var.clone(),
                    Box::new(LambdaTerm::Var(y_var.clone())),
                )),
                Box::new(LambdaTerm::Var(z_var.clone())),
            )),
        );
        let marked = mark_redex(&e);
        let stepped = step(&marked, 0).unwrap();
        // expected: (\y. y) z
        let expected = LambdaTerm::App(
            Box::new(LambdaTerm::Abs(
                y_var.clone(),
                Box::new(LambdaTerm::Var(y_var.clone())),
            )),
            Box::new(LambdaTerm::Var(z_var.clone())),
        );
        assert_eq!(stepped, expected);
    }
}
