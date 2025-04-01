use std::{collections::HashSet, fmt::Display};

use utils::{
    set::SubSet,
    variable::{self, Var},
};

use crate::{LambdaContext, LambdaExt, State};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
    Delim(Box<Lam>),
    Grab(Var, Box<Lam>),
    Zero,
    Succ(Box<Lam>),
    Pred(Box<Lam>),
    IfZ(Box<Lam>, Box<Lam>, Box<Lam>),
    Let(Var, Box<Lam>, Box<Lam>),
    Rec(Var, Var, Box<Lam>),
}

