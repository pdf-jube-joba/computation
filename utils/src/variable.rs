use std::{collections::HashSet, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Var {
    S(String),
    U(usize),
}

impl Var {
    fn to_s(self) -> Self {
        match self {
            Var::S(s) => Var::S(s),
            Var::U(u) => Var::S(u.to_string()),
        }
    }
}

impl From<usize> for Var {
    fn from(value: usize) -> Self {
        Var::S(value.to_string())
    }
}

impl From<&str> for Var {
    fn from(value: &str) -> Self {
        Var::S(value.to_string())
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Var::U(u) => u.to_string(),
            Var::S(s) => s.clone(),
        };
        write!(f, "({})", s)
    }
}

pub fn new_var<'a, T>(vars: T) -> Var
where
    T: IntoIterator<Item = &'a Var>,
{
    let max = vars
        .into_iter()
        .filter_map(|v| match v {
            Var::S(_) => None,
            Var::U(u) => Some(*u),
        })
        .max()
        .unwrap_or(0);
    (max + 1).into()
}

pub trait LamVar: Sized {
    fn decomposition_to_var(&self) -> Option<(Var, Box<dyn Fn(Var) -> Self>)>;
    fn decomposition_to_bound(&self) -> Option<(Var, Self, Box<dyn Fn(Var, Self) -> Self>)>;
    fn traversal(&self, f: Box<dyn Fn(Self) -> Self>) -> Self;
    fn fold<T, A>(&self, f: Box<dyn Fn(Self) -> T>, add: A) -> T
    where
        A: Fn(T, T) -> T;
}

fn add_set(mut s1: HashSet<Var>, s2: HashSet<Var>) -> HashSet<Var> {
    s1.extend(s2);
    s1
}

pub fn free_variables<T>(e: &T) -> HashSet<Var>
where
    T: LamVar,
{
    if let Some((y, _)) = e.decomposition_to_var() {
        return vec![y].into_iter().collect();
    }
    if let Some((y, r, _)) = e.decomposition_to_bound() {
        let mut s = free_variables(&r);
        s.remove(&y);
        return s;
    }
    e.fold(Box::new(|m| free_variables(&m)), add_set)
}

pub fn bound_variables<T>(e: &T) -> HashSet<Var>
where
    T: LamVar,
{
    if let Some((y, r, _)) = e.decomposition_to_bound() {
        let mut s: HashSet<_> = vec![y].into_iter().collect();
        s.extend(bound_variables(&r));
        return s;
    }
    e.fold(Box::new(|m| bound_variables(&m)), add_set)
}

fn alpha_conversion_canonical_rec<T>(e: T, maps: Vec<Var>) -> T
where
    T: LamVar,
{
    if let Some((y, f)) = e.decomposition_to_var() {
        if let Some(new_y) = maps.iter().position(|v| *v == y) {
            return f(new_y.into());
        } else {
            return f(y.to_s());
        }
    }

    if let Some((y, r, f)) = e.decomposition_to_bound() {
        let new_y: Var = maps.len().into();
        let new_y = new_y.to_s();
        let mut maps = maps.clone();
        maps.push(y);
        let new_r = alpha_conversion_canonical_rec(r, maps);
        return f(new_y, new_r);
    }

    e.traversal(Box::new(move |m| {
        alpha_conversion_canonical_rec(m, maps.clone())
    }))
}

pub fn alpha_conversion_canonical<T>(e: T) -> T
where
    T: LamVar,
{
    alpha_conversion_canonical_rec(e, vec![])
}

pub fn subst<L>(e: L, x: Var, t: L) -> L
where
    L: LamVar,
{
    if let Some((y, f)) = e.decomposition_to_var() {
        if x == y {
            return t;
        } else {
            return f(y);
        }
    }
    if let Some((y, r, f)) = e.decomposition_to_bound() {
        if x == y {
            return e;
        }
    }
    todo!()
}
