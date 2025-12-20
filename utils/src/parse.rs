use crate::{alphabet::Alphabet, TextCodec};

pub trait ParseTextCodec {
    fn parse_tc<T: TextCodec>(&self) -> Result<T, String>;
}

impl<T: AsRef<str>> ParseTextCodec for T {
    fn parse_tc<U: TextCodec>(&self) -> Result<U, String> {
        U::parse(self.as_ref())
    }
}

// Implementations for common types
impl TextCodec for () {
    fn parse(text: &str) -> Result<Self, String> {
        if text.trim().is_empty() {
            Ok(())
        } else {
            Err("Expected empty input".to_string())
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "")
    }
}

impl TextCodec for crate::number::Number {
    fn parse(text: &str) -> Result<Self, String> {
        let trimed = text.trim();
        let n = if trimed.is_empty() {
            0
        } else {
            trimed.parse::<usize>().map_err(|e| e.to_string())?
        };
        Ok(crate::number::Number::from(n))
    }
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.as_usize())
    }
}

impl TextCodec for String {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(text.to_string())
    }
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TextCodec for Alphabet {
    fn parse(text: &str) -> Result<Self, String> {
        Alphabet::new(text).map_err(|e| e.to_string())
    }
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// wrapped by "(" and ")", separated by ","
// - trailing comma is allowed: "(item1, item2)" | "(item1, item2,)" are same
// - empty vec is allowed: "()"
// - spaces around items are trimmed: "( item1 , item2 )" | "(item1,item2)" are same
impl<T> TextCodec for Vec<T>
where
    T: TextCodec,
{
    fn parse(text: &str) -> Result<Self, String> {
        let trimed = text.trim();
        if !trimed.starts_with('(') || !trimed.ends_with(')') {
            return Err("Expected to start with '(' and end with ')'".to_string());
        }
        let mut inner = &trimed[1..trimed.len() - 1];
        let mut v = Vec::new();

        loop {
            let until_comma_or_end = inner.find(',').unwrap_or(inner.len());
            let (item_str, rest) = inner.split_at(until_comma_or_end);
            let item_str = item_str.trim();
            if !item_str.is_empty() {
                let item = T::parse(item_str)?;
                v.push(item);
            }
            inner = rest.trim_start();
            if inner.is_empty() {
                break;
            }
            if !inner.starts_with(',') {
                return Err("Expected ',' between items".to_string());
            }
            inner = inner[1..].trim_start();
        }

        Ok(v)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            item.write_fmt(f)?;
        }
        write!(f, ")")
    }
}
