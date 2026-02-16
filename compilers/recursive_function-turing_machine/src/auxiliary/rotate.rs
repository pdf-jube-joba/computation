use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::symbols;

// ... [x] A_1 x A_2 x ... x A_n x
// ... [x] A_2 x A_3 x ... x A_1 x
// A_i: list of {'-', 'l'}
pub fn rotate(n: usize) -> TuringMachineBuilder {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder_test, tape_from};

    #[test]
    fn builder_safe() {
        let _ = rotate(2);
    }
    #[test]
    fn rotate_test() {
        let mut builder = rotate(3);
        let tests = vec![
            (
                tape_from(&["x", "x", "x", "x"], 0),
                tape_from(&["x", "x", "x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "x", "-", "l", "x", "-", "l", "l", "x"], 0),
                tape_from(&["x", "-", "l", "x", "-", "l", "l", "x", "-", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }
}
