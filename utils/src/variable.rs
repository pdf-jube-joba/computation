use std::{borrow::Borrow, collections::HashSet, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Var {
    S(String),
    U(usize),
}

impl Var {
    pub fn new_s(s: String) -> Self {
        Var::S(s)
    }
    pub fn new_u(u: usize) -> Self {
        Var::U(u)
    }
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
        match self {
            Var::U(u) => {
                write!(f, "_{u}_")
            }
            Var::S(s) => {
                write!(f, "{s}")
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VarSet {
    set: HashSet<Var>,
    max: usize,
}

impl<I, V> From<I> for VarSet
where
    I: IntoIterator<Item = V>,
    V: Borrow<Var>,
{
    fn from(value: I) -> Self {
        let mut set = HashSet::new();
        let mut max = 0;
        for v in value {
            match v.borrow() {
                Var::S(_) => {}
                Var::U(u) => max = std::cmp::max(*u, max),
            }
            set.insert(v.borrow().clone());
        }
        VarSet { set, max }
    }
}

impl VarSet {
    /// return v: Var s.t. v \notin self, and
    fn new_var(&self) -> Var {
        Var::from_num(self.max + 1)
    }

    fn consume(&mut self) {
        self.max += 1;
    }

    pub fn contains(&self, elem: &Var) -> bool {
        self.set.contains(elem)
    }

    pub fn new_var_not_modify(&self) -> Var {
        self.new_var()
    }

    /// return (v: Var s.t. v \notin self)
    pub fn new_var_modify(&mut self) -> Var {
        let new_var = self.new_var();
        self.consume();
        new_var
    }

    /// if (default \notin self) then (return default) else (return v: Var s.t. v \notin self), and self <= self + {v}
    pub fn new_var_default<V>(&mut self, default: V) -> Var
    where
        V: Borrow<Var>,
    {
        if self.contains(default.borrow()) {
            self.new_var_modify()
        } else {
            default.borrow().clone()
        }
    }

    pub fn insert<V>(&mut self, var: V) -> bool
    where
        V: Borrow<Var>,
    {
        if let Var::U(u) = var.borrow() {
            self.max = std::cmp::max(self.max, *u);
        }
        self.set.insert(var.borrow().clone())
    }

    pub fn remove<V>(&mut self, var: V) -> bool
    where
        V: Borrow<Var>,
    {
        self.set.remove(var.borrow())
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut set = self.set.clone();
        set.extend(other.set.iter().cloned());
        VarSet {
            set,
            max: std::cmp::max(self.max, other.max),
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.set.extend(other.set.clone());
        self.max = std::cmp::max(self.max, other.max);
    }
}

impl FromIterator<Var> for VarSet {
    fn from_iter<T: IntoIterator<Item = Var>>(iter: T) -> Self {
        let mut set = HashSet::new();
        let mut max = 0;
        for v in iter {
            match v {
                Var::S(_) => {}
                Var::U(u) => max = std::cmp::max(u, max),
            }
            set.insert(v.borrow().clone());
        }
        VarSet { set, max }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variableset() {
        let v: Vec<Var> = vec!["x".into()];
        let mut varset: VarSet = v.into_iter().collect();
        assert!(varset.contains(&"x".into()));
        assert!(!varset.contains(&"y".into()));
        let new_var = varset.new_var();
        assert!(!varset.contains(&new_var));

        let new_var1 = varset.new_var_modify();
        let new_var2 = varset.new_var_modify();
        assert!(!varset.contains(&new_var1));
        assert!(!varset.contains(&new_var2));
        assert!(new_var1 != new_var2);
    }
}
