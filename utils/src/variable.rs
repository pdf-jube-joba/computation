use std::{borrow::Borrow, collections::HashSet, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Var {
    S(String),
    U(usize),
}

impl Var {
    pub fn into_s(self) -> Self {
        match self {
            Var::S(s) => Var::S(s),
            Var::U(u) => Var::S(u.to_string()),
        }
    }
    pub fn from_num(u: usize) -> Self {
        Var::U(u)
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

#[derive(Debug, Clone, PartialEq)]
pub struct VarSet(HashSet<Var>);

impl<I, V> From<I> for VarSet
where
    I: IntoIterator<Item = V>,
    V: AsRef<Var>,
{
    fn from(value: I) -> Self {
        VarSet(value.into_iter().map(|v| v.as_ref().clone()).collect())
    }
}

pub fn new_var<Iter, Item>(vars: Iter) -> Var
where
    Iter: IntoIterator<Item = Item>,
    Item: Borrow<Var>,
{
    let max = vars
        .into_iter()
        .filter_map(|v| match v.borrow() {
            Var::S(_) => None,
            Var::U(u) => Some(*u),
        })
        .max()
        .unwrap_or(0);
    (max + 1).into()
}

// free variable をよけて bound variable を付け替えるためにつかう
// つまり、 alpha conversion 用
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarMap {
    shouldnot_use: HashSet<usize>,
    maps: Vec<Option<Var>>,
}

impl VarMap {
    pub fn new_iter<T>(shouldnot_use: T) -> Self
    where
        T: IntoIterator<Item = Var>,
    {
        let nums = shouldnot_use
            .into_iter()
            .filter_map(|x| match x {
                Var::U(u) => Some(u),
                Var::S(_) => None,
            })
            .collect();
        Self {
            shouldnot_use: nums,
            maps: vec![],
        }
    }
    pub fn push_var(&mut self, v: &Var) {
        let l = self.maps.len();
        if self.shouldnot_use.contains(&l) {
            self.maps.push(None);
            self.maps.push(Some(v.clone()));
        } else {
            self.maps.push(Some(v.clone()));
        }
    }
    pub fn get_table(&self, x: &Var) -> Var {
        match self.maps.iter().position(|y| Some(x) == y.as_ref()) {
            Some(u) => Var::U(u),
            None => x.clone(),
        }
    }
}

// pub trait LamVar: Sized {
//     // return Some() if it is variable
//     fn decomposition_to_var(&self) -> Option<(Var, Box<dyn Fn(Var) -> Self>)>;
//     // return Some() if it bound variable
//     fn decomposition_to_bound(&self) -> Option<(Var, Box<dyn Fn(Var) -> Self>)>;
//     fn traversal(&self, f: Box<dyn Fn(Self) -> Self>) -> Self;
//     fn fold<T, F, A>(&self, f: &F, add: A) -> T
//     where
//         F: Fn(Self) -> T,
//         T: Default,
//         A: Fn(T, T) -> T;
// }

// fn add_set(mut s1: HashSet<Var>, s2: HashSet<Var>) -> HashSet<Var> {
//     s1.extend(s2);
//     s1
// }

// pub fn free_variables<T>(e: &T) -> HashSet<Var>
// where
//     T: LamVar,
// {

//     let mut s = e.fold(&Box::new(|m| free_variables(&m)), add_set);

//     if let Some((y, _)) = e.decomposition_to_var() {
//         s.insert(y);
//     }

//     if let Some((y, _)) = e.decomposition_to_bound() {
//         s.remove(&y);
//     }
//     s
// }

// pub fn bound_variables<T>(e: &T) -> HashSet<Var>
// where
//     T: LamVar,
// {
//     let mut s = e.fold(&Box::new(|m| bound_variables(&m)), add_set);
//     if let Some((y, _)) = e.decomposition_to_bound() {
//         s.remove(&y);
//     }
//     s
// }

// fn alpha_conversion_canonical_rec<T>(e: T, maps: Vec<Var>) -> T
// where
//     T: LamVar,
// {
//     if let Some((y, f)) = e.decomposition_to_var() {
//         if let Some(new_y) = maps.iter().position(|v| *v == y) {
//             return f(new_y.into());
//         } else {
//             return f(y.into_s());
//         }
//     }

//     if let Some((y, f)) = e.decomposition_to_bound() {
//         let new_y: Var = maps.len().into();
//         let new_y = new_y.into_s();
//         let mut maps = maps.clone();
//         maps.push(y);
//         let new_r = alpha_conversion_canonical_rec(r, maps);
//         return f(new_y, new_r);
//     }

//     e.traversal(Box::new(move |m| {
//         alpha_conversion_canonical_rec(m, maps.clone())
//     }))
// }

// pub fn alpha_conversion_canonical<T>(e: T) -> T
// where
//     T: LamVar,
// {
//     alpha_conversion_canonical_rec(e, vec![])
// }

// pub fn subst<L>(e: L, x: Var, t: L) -> L
// where
//     L: LamVar,
// {
//     if let Some((y, f)) = e.decomposition_to_var() {
//         if x == y {
//             return t;
//         } else {
//             return f(y);
//         }
//     }
//     if let Some((y, r, f)) = e.decomposition_to_bound() {
//         if x == y {
//             return e;
//         }
//     }
//     todo!()
// }

// mod test_lambda_calculus {
//     use super::*;

//     #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
//     enum Exp {
//         Var(Var),
//         Abs(Var, Box<Exp>),
//         App(Box<Exp>, Box<Exp>),
//         Let(Var, Box<Exp>, Box<Exp>),
//     }
// }
