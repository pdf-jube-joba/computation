use std::fmt::Display;

use serde::Serialize;

// string consists of
// (ASCII_CHAR | '_') ~ (ASCII_CHAR | DIGIT | '_' | '-')*
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Alphabet(String);

impl Alphabet {
    pub fn new(name: &str) -> Result<Self, anyhow::Error> {
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

        Ok(Alphabet(name.to_string()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identifier {
    User(String),
    System(String),
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::User(name) => write!(f, "{}", name),
            Identifier::System(name) => write!(f, "_{}_", name),
        }
    }
}

impl Identifier {
    pub fn new_user(name: &str) -> Result<Self, anyhow::Error> {
        // Changed from String to anyhow::Error
        if name.is_empty() {
            return Err(anyhow::anyhow!("alphabet cannot be empty"));
        }
        if name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Identifier::User(name.to_string()))
        } else {
            Err(anyhow::anyhow!(format!("invalid alphabet:[{}]", name)))
        }
    }
    pub fn new_system(name: &str) -> Self {
        Identifier::System(name.to_string())
    }
    pub fn to_usize(&self) -> Option<usize> {
        match self {
            Identifier::User(name) | Identifier::System(name) => name.parse::<usize>().ok(),
        }
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        match self {
            Identifier::User(name) => name,
            Identifier::System(name) => name,
        }
    }
}
