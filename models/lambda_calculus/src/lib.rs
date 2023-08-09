use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Var(usize);

fn new_var(set: &HashSet<Var>) -> Var {
    Var(set.iter().map(|var| var.0).max().unwrap_or_default() + 1)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaTerm {
    Variable(Var),
    Abstraction(Var, Box<LambdaTerm>),
    Application(Box<LambdaTerm>, Box<LambdaTerm>),
}

pub fn var_change(var_pre: Var, var_post: Var, term: LambdaTerm) -> LambdaTerm {
    match &term {
        LambdaTerm::Variable(variable) => {
            if var_pre == *variable { LambdaTerm::Variable(var_post) } else { LambdaTerm::Variable(variable.clone()) }
        },
        LambdaTerm::Abstraction(variable, abs_term) => {
            if var_pre == *variable { term } else {
                LambdaTerm::Abstraction(variable.clone(), Box::new(var_change(var_pre, var_post, *abs_term.clone())))
            }
        },
        LambdaTerm::Application(app_term1, app_term2)  => {
            LambdaTerm::Application(
                Box::new(var_change(var_pre.clone(), var_post.clone(), *app_term1.clone())),
                Box::new(var_change(var_pre.clone(), var_post.clone(), *app_term2.clone()))
            )
        }
    }
}

pub fn subst(term1: LambdaTerm, var: Var, term2: LambdaTerm) -> LambdaTerm {
    match term1.clone() {
        LambdaTerm::Variable(variable) => {
            if variable == var { term2 } else { term1 }
        },
        LambdaTerm::Abstraction(variable, abs_term) => {
            if variable == var { term1 } else { LambdaTerm::Abstraction(variable, Box::new(subst(*abs_term, var, term2))) }
        },
        LambdaTerm::Application(app_term1, app_term2) => {
            LambdaTerm::Application(
                Box::new(subst(*app_term1, var.clone(), term2.clone())),
                Box::new(subst(*app_term2, var, term2)),
            )
        }
    }
}
