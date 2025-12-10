use serde::Serialize;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod set;
pub mod variable;

// This trait is implemented by concrete models. It stays generic and not object-safe.
pub trait Machine: Sized {
    type Code: Serialize; // static code
    type AInput: Serialize; // ahead of time input
    type This: Serialize; // representation of the current state
    type RInput: Serialize; // runtime input
    type Output: Serialize; // output after a step

    fn parse_code(code: &str) -> Result<Self::Code, String>;
    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String>;
    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String>;
    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String>;
    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String>;
    fn current(&self) -> Self::This;
}
