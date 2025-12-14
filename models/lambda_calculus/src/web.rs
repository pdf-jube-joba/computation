use crate::machine::{LambdaTerm, MarkedTerm};
use serde::Serialize;
use utils::{number::Number, Machine, TextCodec};

impl TextCodec for LambdaTerm {
    fn parse(text: &str) -> Result<Self, String> {
        crate::manipulation::parse::parse_lambda_read_to_end(text)
    }

    fn print(data: &Self) -> String {
        match data {
            LambdaTerm::Var(var) => var.as_str().to_string(),
            LambdaTerm::Abs(var, lambda_term) => {
                let body = LambdaTerm::print(lambda_term);
                format!("\\{}. {}", var.as_str(), body)
            }
            LambdaTerm::App(lambda_term, lambda_term1) => {
                let lhs = LambdaTerm::print(lambda_term);
                let rhs = LambdaTerm::print(lambda_term1);
                format!("({} {})", lhs, rhs)
            }
        }
    }
}

#[derive(Clone, Serialize)]
pub struct AInput(pub Vec<LambdaTerm>);

impl TextCodec for AInput {
    fn parse(text: &str) -> Result<Self, String> {
        let mut v = vec![];
        for txt in text.split(",") {
            let term = crate::manipulation::parse::parse_lambda_read_to_end(txt.trim())?;
            v.push(term);
        }
        Ok(AInput(v))
    }

    fn print(data: &Self) -> String {
        let mut strs = vec![];
        for term in &data.0 {
            let s = LambdaTerm::print(term);
            strs.push(s);
        }
        strs.join(", ")
    }
}

impl Machine for LambdaTerm {
    type Code = LambdaTerm;
    type AInput = AInput;
    type SnapShot = MarkedTerm;
    type RInput = Number;
    type Output = LambdaTerm;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let term = crate::machine::assoc_app(code, ainput.0);
        Ok(term)
    }

    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let marked = crate::machine::mark_redex(self);
        let lambda =
            crate::machine::step(&marked, rinput.0).ok_or("No redex found at the given index")?;
        *self = lambda;
        Ok(Some(self.clone()))
    }

    fn current(&self) -> Self::SnapShot {
        crate::machine::mark_redex(self)
    }
}
