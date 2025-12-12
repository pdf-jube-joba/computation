use crate::machine::{LambdaTerm, MarkedTerm};
use utils::Machine;

impl Machine for LambdaTerm {
    type Code = LambdaTerm;
    type AInput = Vec<LambdaTerm>;
    type SnapShot = MarkedTerm;
    type RInput = usize;
    type Output = LambdaTerm;

    fn parse_code(code: &str) -> Result<Self::Code, String> {
        crate::manipulation::parse::parse_lambda_read_to_end(code)
    }

    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        ainput
            .split(",")
            .filter(|s| !s.trim().is_empty())
            .map(|s| crate::manipulation::parse::parse_lambda_read_to_end(s.trim()))
            .collect()
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

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let term = crate::machine::assoc_app(code, ainput);
        Ok(term)
    }

    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let marked = crate::machine::mark_redex(self);
        let lambda =
            crate::machine::step(&marked, rinput).ok_or("No redex found at the given index")?;
        *self = lambda;
        Ok(Some(self.clone()))
    }

    fn current(&self) -> Self::SnapShot {
        crate::machine::mark_redex(self)
    }
}
