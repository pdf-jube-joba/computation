mod machine;
mod flatten;
mod parser;

pub use machine::{
    validate_no_recursion, Environment, Function, Program, RecTmIrMachine, Snapshot, Stmt,
};
pub use flatten::flatten_program;

#[cfg(test)]
mod test;
