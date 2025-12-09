use std::{fmt::Display, rc::Rc};

use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct Var(Rc<str>);

impl Var {
    pub fn new(s: &str) -> Self {
        Var(Rc::from(s))
    }
    pub fn dummy() -> Self {
        Var(Rc::from("_"))
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

pub fn print_var(var: &Var) -> String {
    // format!("{}[{}]", content, lower 32 bit to base62 encoding)
    let content = var.as_str();

    let base62_chars = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut ptr: u32 = ((var.0.as_ptr() as usize) & 0xffff_ffff) as u32;
    let mut encoded: [u8; 6] = [0; 6];

    for i in (0..6).rev() {
        encoded[i] = base62_chars[(ptr % 62) as usize];
        ptr /= 62;
    }
    format!("{}[{}]", content, std::str::from_utf8(&encoded).unwrap())
}

impl Serialize for Var {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&print_var(self))
    }
}
