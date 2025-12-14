use serde::Serialize;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod set;
pub mod variable;

pub trait TextCodec: Sized {
    fn parse(text: &str) -> Result<Self, String>;
    fn print(data: &Self) -> Result<String, String>;
    // we expect print(parse(x)).is_ok() && print(parse(x)).is_ok()
    // but not necessarily "is identity" (something lossy is allowed)
}

impl TextCodec for () {
    fn parse(text: &str) -> Result<Self, String> {
        if text.trim().is_empty() {
            Ok(())
        } else {
            Err("Expected empty input".to_string())
        }
    }
    fn print(_data: &Self) -> Result<String, String> {
        Ok("".to_string())
    }
}

pub trait Machine: Sized {
    type Code: Serialize + TextCodec; // static code
    type AInput: Serialize + TextCodec; // ahead of time input
    type SnapShot: Serialize; // representation of the current state
    type RInput: Serialize + TextCodec; // runtime input
    type Output: Serialize; // output after a step

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
