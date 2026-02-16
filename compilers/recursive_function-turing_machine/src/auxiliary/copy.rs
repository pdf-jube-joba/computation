use turing_machine::manipulation::builder::TuringMachineBuilder;

use crate::*;

// ... [x] - A x
// ... [x] - A x - A x
// A: list of 'l'
pub fn copy() -> TuringMachineBuilder {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_safe() {
        let _ = copy();
    }
    #[test]
    fn copy_test() {
        let mut builder = copy();
        let tests = vec![
            (tape_from(&["x", "x"], 0), tape_from(&["x", "x", "x"], 0)),
            (
                tape_from(&["x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "x", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
}
