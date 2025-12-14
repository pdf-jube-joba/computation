use serde::Serialize;

use crate::number::Number;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod set;
pub mod variable;

pub trait TextCodec: Sized {
    fn parse(text: &str) -> Result<Self, String>;
    fn print(data: &Self) -> String;
    // we expect print(parse(x)).is_ok() && print(parse(x)).is_ok()
    // but not necessarily "is identity" (something lossy is allowed)
}

pub trait Machine: Sized {
    type Code: Serialize + TextCodec; // static code
    type AInput: Serialize + TextCodec; // ahead of time input
    type SnapShot: Serialize; // representation of the current state
    type RInput: Serialize + TextCodec; // runtime input
    type Output: Serialize + TextCodec; // output after a step

    fn parse_code(code: &str) -> Result<Self::Code, String> {
        Self::Code::parse(code)
    }
    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        Self::AInput::parse(ainput)
    }
    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        Self::RInput::parse(rinput)
    }
    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String>;
    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String>;
    fn current(&self) -> Self::SnapShot;
}

pub trait Compiler: Sized {
    type Source: Machine; // source code
    type Target: Machine; // target code

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String>;
    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String>;
    fn decode_output(
        output: <<Self as Compiler>::Target as Machine>::Output,
    ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String>;
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
    fn print(_data: &Self) -> String {
        "".to_string()
    }
}

impl TextCodec for Number {
    fn parse(text: &str) -> Result<Self, String> {
        let n = text.trim().parse::<usize>().map_err(|e| e.to_string())?;
        Ok(Number::from(n))
    }
    fn print(data: &Self) -> String {
        data.to_string()
    }
}

impl TextCodec for String {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(text.to_string())
    }
    fn print(data: &Self) -> String {
        data.clone()
    }
}

// wrapped by "(" and ")"
impl TextCodec for Vec<Number> {
    fn parse(text: &str) -> Result<Self, String> {
        let trimmed = text.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return Err("Expected input to be wrapped in parentheses".to_string());
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        let mut result = vec![];
        for part in inner.split(',') {
            let n = part.trim().parse::<usize>().map_err(|e| e.to_string())?;
            result.push(Number::from(n));
        }
        Ok(result)
    }

    fn print(data: &Self) -> String {
        let parts: Vec<String> = data.iter().map(|n| n.to_string()).collect();
        format!("({})", parts.join(", "))
    }
}
