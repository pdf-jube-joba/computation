mod machine;
mod flatten;
mod parser;
mod validation;

pub use machine::{
    CallArg, Environment, Function, Program, RecTmIrMachine, Snapshot, Stmt,
};
pub use validation::validate_no_recursion;
pub use flatten::flatten_program;

#[cfg(test)]
mod test;
