use std::{fmt::Display, rc::Rc};

use serde::Serialize;

// variable for "take fresh variable" language
// NOTE: variables are compared by pointer equality
// WARNING: do care when constructing Var from string literals!
//    Var("x") != Var("x") unless they are from the same allocation!
#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct Var(Rc<str>);

/// JSON-friendly view of `Var` for the web side.
#[derive(Debug, Clone, Serialize)]
pub struct VarView {
    pub name: String,
    pub ptr: usize,
}

impl Var {
    pub fn new(s: &str) -> Self {
        Var(Rc::from(s))
    }
    // safe for make a dummy variable at any where (on code) and any time (at runtime)
    // no overwrapping issue
    pub fn dummy() -> Self {
        Var(Rc::from("_"))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn view(&self) -> VarView {
        VarView::from(self)
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

pub fn print_var(var: &Var) -> String {
    let view = var.view();
    format!("{}[{:#x}]", view.name, view.ptr)
}

impl Serialize for Var {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        VarView::from(self).serialize(serializer)
    }
}

impl From<&Var> for VarView {
    fn from(var: &Var) -> Self {
        let ptr = Rc::as_ptr(&var.0).addr();
        VarView {
            name: var.as_str().to_string(),
            ptr,
        }
    }
}

// variable for string representation
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize)]
pub struct VarStr(String);

impl VarStr {
    pub fn new<T>(s: T) -> Self
    where
        T: AsRef<str>,
    {
        VarStr(s.as_ref().to_string())
    }
    // no implementation for dummy variable, as string representation should be unique
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
