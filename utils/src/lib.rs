pub mod bool;
pub mod cli;
pub mod cli_compiler;
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

pub enum StepResult<M: Machine> {
    Continue {
        next: M,
        output: M::ROutput,
    },
    Halt {
        snapshot: M::SnapShot,
        output: M::FOutput,
    },
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

    // for each steps
    // representation of the current state
    type SnapShot;
    // runtime input
    type RInput: TextCodec;
    // runtime output after a step
    type ROutput: TextCodec;

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
    fn step(self, rinput: Self::RInput) -> Result<StepResult<Self>, String>;
    fn current(&self) -> Self::SnapShot;
}

pub trait Compiler: Sized {
    type Source: Machine;
    type Target: Machine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
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

#[doc(hidden)]
pub mod web_util;

#[macro_export]
macro_rules! model_entry {
    ($machine:path) => {
        #[cfg(target_arch = "wasm32")]
        $crate::web_model!($machine);

        #[cfg(not(target_arch = "wasm32"))]
        $crate::cli_model!($machine);
    };
}

#[macro_export]
macro_rules! compiler_entry {
    ($compiler:path) => {
        #[cfg(target_arch = "wasm32")]
        $crate::web_compiler!($compiler);

        #[cfg(not(target_arch = "wasm32"))]
        $crate::cli_compiler!($compiler);
    };
}
