// web/component 向けマクロで参照するために re-export しておく。
pub use wit_bindgen;

// module for parsing and encoding text
pub mod parse;
pub use parse::TextCodec;

// data structures
pub mod data;

// utilities for web
pub mod wasm_util;

pub enum StepResult<M: Machine> {
    Continue { next: M, output: M::ROutput },
    Halt { output: M::FOutput },
}

// trait for models of computation
pub trait Machine: Sized {
    // "semantics" of the models
    // a model can be considered as a partial function (Code, AInput) -> FOutput
    // static code
    type Code: TextCodec;
    // ahead of time input
    type AInput: TextCodec;
    // final output at halt
    type FOutput: TextCodec;

    // small step semantics of the models
    // runtime input
    type RInput: TextCodec;
    // runtime output after a step
    type ROutput: TextCodec;

    // representation of the current state
    type SnapShot: serde::Serialize + serde::de::DeserializeOwned;

    // parsing
    fn parse_code(code: &str) -> Result<Self::Code, String> {
        Self::Code::parse(code)
    }
    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        Self::AInput::parse(ainput)
    }
    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        Self::RInput::parse(rinput)
    }

    // semantics
    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String>;
    fn step(self, rinput: Self::RInput) -> Result<StepResult<Self>, String>;

    // save/restore snapshot of the current state
    fn snapshot(&self) -> Self::SnapShot;
    fn restore(snapshot: Self::SnapShot) -> Self;

    // rendering of Snapshot for web
    fn render(snapshot: Self::SnapShot) -> serde_json::Value;
}

pub trait Compiler: Sized {
    type Source: Machine;
    type Target: Machine;

    // compile of code
    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
    // encoding/decoding of inputs and outputs ... how to interpret the source/target code as the same "program"?
    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String>;
    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String>;
    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String>;
    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String>;
}
