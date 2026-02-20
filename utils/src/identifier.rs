use std::fmt::Display;

use serde::Serialize;

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
