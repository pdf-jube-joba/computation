use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Debug, Display};

// string consists of
// (ASCII_CHAR | '_') ~ (ASCII_CHAR | DIGIT | '_' | '-')*
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Identifier(String);

impl Identifier {
    pub fn new<T>(name: T) -> Result<Self, anyhow::Error>
    where
        T: AsRef<str>,
    {
        let name = name.as_ref();
        if name.is_empty() {
            return Err(anyhow::anyhow!("alphabet cannot be empty"));
        }

        if !name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            return Err(anyhow::anyhow!(format!(
                "invalid alphabet start:[{}]",
                name
            )));
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(anyhow::anyhow!(format!("invalid alphabet:[{}]", name)));
        }

        Ok(Identifier(name.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

use std::rc::Rc;

// variable for "take fresh variable" language
// NOTE: variables are compared by pointer equality
// WARNING: do care when constructing Var from string literals!
//    Var("x") != Var("x") unless they are from the same allocation!
#[derive(Clone, Eq, PartialOrd, Ord)]
pub struct Var(Rc<str>);

impl Var {
    pub fn as_ptr_usize(&self) -> usize {
        let ptr: *const () = Rc::as_ptr(&self.0) as *const ();
        ptr as usize
    }
}

impl Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Var({})[{:#x}]",
            self.as_str(),
            Rc::as_ptr(&self.0).addr()
        )
    }
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

/// JSON-friendly view of `Var` for the web side.
#[derive(Debug, Clone, Serialize)]
pub struct VarView {
    pub name: String,
    pub ptr: usize,
}

impl<T> From<T> for Var
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        Var::new(s.as_ref())
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

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Identifier::new(s).map_err(serde::de::Error::custom)
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
