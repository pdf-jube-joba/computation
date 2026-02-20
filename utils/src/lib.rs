pub mod bool;
pub mod identifier;
pub mod number;
pub mod parse;
pub mod variable;

// ここら辺がないと動かないが、
// utils 側でも import をしておかないと、 `#[wasm_bindgen]` マクロが動かない。
pub use serde;
pub use serde_json;
pub use serde_wasm_bindgen;
pub use wasm_bindgen;

pub trait TextCodec: Sized {
    fn parse(text: &str) -> Result<Self, String>;
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result;
    fn print(&self) -> String {
        let mut s = String::new();
        self.write_fmt(&mut s).unwrap();
        s
    }
}

pub trait Machine: Sized {
    type Code: TextCodec; // static code
    type AInput: TextCodec; // ahead of time input
    type SnapShot; // representation of the current state
    type RInput: TextCodec; // runtime input
    type Output: TextCodec; // output after a step

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

mod web_util;
