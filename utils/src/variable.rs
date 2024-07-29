use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var(usize);

impl From<usize> for Var {
    fn from(value: usize) -> Self {
        Var(value)
    }
}

impl TryFrom<&str> for Var {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Var(value.parse().map_err(|_| ())?))
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var({})", self.0)
    }
}

pub fn new_var<'a, T>(vars: T) -> Var
where
    T: IntoIterator<Item = &'a Var>,
{
    let max = vars.into_iter().map(|v| v.0).max().unwrap_or(0);
    (max + 1).into()
}
