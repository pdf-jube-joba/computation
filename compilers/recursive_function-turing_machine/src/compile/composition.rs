use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

// [x] - A_1 - ... - A_n x
// [x] - B_1 - B_m x
// [x] - C x
// where
// 1.   [x] - A_1 - ... - A_n x
//      [x] - B_i x
//    by inner_builder[i] (1-index in this comment)
// 1.   [x] - B_1 - ... - B_n x
//      [x] - C x
//    by outer_builder
pub fn composition(
    inner_builder: Vec<TuringMachineBuilder>,
    outer_builder: TuringMachineBuilder,
) -> TuringMachineBuilder {
    todo!()
}
