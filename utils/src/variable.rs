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
