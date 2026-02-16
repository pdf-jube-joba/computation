use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::symbols;

// ... [x] - A_0 - A_1 - ... - A_{n-1} x - - ...
// ... [x] - A_i x - - ...
// A_i: list of 'l'
pub fn projection(n: usize, i: usize) -> TuringMachineBuilder {
    todo!()
}
