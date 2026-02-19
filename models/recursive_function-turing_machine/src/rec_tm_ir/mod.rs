mod machine;
mod parser;
mod validation;

pub use machine::{Block, Environment, Function, Program, RecTmIrMachine, Snapshot, Stmt, RValue, LValue, Condition};
pub use validation::validate_no_recursion;
pub use turing_machine::machine::Tape;

#[cfg(test)]
mod test;
