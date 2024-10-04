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

impl From<String> for Var {
    fn from(value: String) -> Self {
        Var::S(value)
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
    V: Borrow<Var>,
{
    fn from(value: I) -> Self {
        VarSet(value.into_iter().map(|v| v.borrow().clone()).collect())
    }
}

impl VarSet {
    /// return v: Var s.t. v \notin self, and
    fn new_var(&self) -> Var {
        let new_u = self
            .0
            .iter()
            .filter_map(|v| match v {
                Var::S(_) => None,
                Var::U(u) => Some(*u),
            })
            .max()
            .unwrap_or(0);
        Var::U(new_u)
    }
    pub fn is_in(&self, elem: &Var) -> bool {
        self.0.contains(elem)
    }
    pub fn new_var_not_midify(&self) -> Var {
        self.new_var()
    }
    /// return (v: Var s.t. v \notin self), and self <= self + {v}
    pub fn new_var_modify(&mut self) -> Var {
        let new_var = self.new_var();
        self.insert(new_var.clone());
        new_var
    }
    /// if (default \notin self) then (return default) else (return v: Var s.t. v \notin self), and self <= self + {v}
    pub fn new_var_default<V>(&mut self, default: V) -> Var
    where
        V: Borrow<Var>,
    {
        if self.is_in(default.borrow()) {
            self.new_var()
        } else {
            default.borrow().clone()
        }
    }
    pub fn insert<V>(&mut self, var: V) -> bool
    where
        V: Borrow<Var>,
    {
        self.0.insert(var.borrow().clone())
    }
    pub fn remove<V>(&mut self, var: V) -> bool
    where
        V: Borrow<Var>,
    {
        self.0.remove(var.borrow())
    }
    pub fn union(&self, other: &Self) -> Self {
        let mut set = self.0.clone();
        set.extend(other.0.iter().cloned());
        VarSet(set)
    }
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
