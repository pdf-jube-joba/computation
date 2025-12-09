use crate::machine::{LambdaTerm, MarkedTerm};
use utils::MealyMachine;

impl MealyMachine for LambdaTerm {
    type Input = usize;
    type Output = ();
    type This = MarkedTerm;

    fn parse_self(input: &str) -> Result<Self, String> {
        crate::manipulation::parse::parse_lambda_read_to_end(input)
    }

    fn parse_input(input: &str) -> Result<Self::Input, String> {
        if input.is_empty() {
            Ok(0)
        } else {
            input
                .trim()
                .parse::<usize>()
                .map_err(|e| format!("Failed to parse input '{}': {}", input, e))
        }
    }

    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String> {
        let marked = crate::machine::mark_redex(self);
        let lambda =
            crate::machine::step(&marked, input).ok_or("No redex found at the given index")?;
        *self = lambda;
        Ok(Some(()))
    }

    fn current(&self) -> Self::This {
        crate::machine::mark_redex(self)
    }
}
