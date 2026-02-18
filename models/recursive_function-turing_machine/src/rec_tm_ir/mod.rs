mod flatten;
mod machine;
mod parser;
mod validation;

pub use flatten::flatten_program;
pub use machine::{CallArg, Environment, Function, Program, RecTmIrMachine, Snapshot, Stmt};
pub use validation::validate_no_recursion;

#[cfg(test)]
mod test;
