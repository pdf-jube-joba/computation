use std::fmt::Display;

// string consists of ascii, number, underscore, hyphen,
// but not empty
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Alphabet(String);

impl TryFrom<&str> for Alphabet {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Alphabet::try_from(s.to_string())
    }
}

impl TryFrom<String> for Alphabet {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err("alphabet cannot be empty".to_string());
        }
        if s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Alphabet(s))
        } else {
            Err(format!("invalid alphabet:[{}]", s))
        }
    }
}

impl Display for Alphabet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
