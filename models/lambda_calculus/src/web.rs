use crate::machine::{LambdaTerm, MarkedTerm};
use utils::Machine;

impl Machine for LambdaTerm {
    type Code = LambdaTerm;
    type AInput = ();
    type This = MarkedTerm;
    type RInput = usize;
    type Output = ();

    fn parse_code(code: &str) -> Result<Self::Code, String> {
        crate::manipulation::parse::parse_lambda_read_to_end(code)
    }

    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        if ainput.trim().is_empty() {
            Ok(())
        } else {
            Err("Lambda calculus machine does not take ahead-of-time input".to_string())
        }
    }

    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        if rinput.is_empty() {
            Ok(0)
        } else {
            rinput
                .trim()
                .parse::<usize>()
                .map_err(|e| format!("Failed to parse input '{}': {}", rinput, e))
        }
    }

    fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
        Ok(code)
    }

    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let marked = crate::machine::mark_redex(self);
        let lambda =
            crate::machine::step(&marked, rinput).ok_or("No redex found at the given index")?;
        *self = lambda;
        Ok(Some(()))
    }

    fn current(&self) -> Self::This {
        crate::machine::mark_redex(self)
    }
}
