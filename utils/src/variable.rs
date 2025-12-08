use std::{
    fmt::Display,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static DUMMY_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct Var(Rc<str>);

impl Var {
    pub fn new(s: &str) -> Self {
        Var(Rc::from(s))
    }
    pub fn dummy() -> Self {
        let id = DUMMY_COUNTER.fetch_add(1, Ordering::Relaxed);
        Var(Rc::from(format!("_dummy_{id}")))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for Var
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        Var::new(s.as_ref())
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::hash::Hash for Var {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}
