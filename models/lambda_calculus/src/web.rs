use crate::machine::{LambdaTerm, MarkedTerm, is_normal_form};
use serde::Serialize;
use utils::{number::Number, Machine, TextCodec};

impl TextCodec for LambdaTerm {
    fn parse(text: &str) -> Result<Self, String> {
        crate::manipulation::parse::parse_lambda_read_to_end(text)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            LambdaTerm::Var(var) => write!(f, "{}", var.as_str()),
            LambdaTerm::Abs(var, lambda_term) => {
                write!(f, "\\{}. ", var.as_str())?;
                lambda_term.write_fmt(f)
            }
            LambdaTerm::App(lambda_term, lambda_term1) => {
                write!(f, "(")?;
                lambda_term.write_fmt(f)?;
                write!(f, " ")?;
                lambda_term1.write_fmt(f)?;
                write!(f, ")")
            }
        }
    }
}

#[derive(Clone, Serialize)]
pub struct AInput(pub Vec<LambdaTerm>);

impl TextCodec for AInput {
    fn parse(text: &str) -> Result<Self, String> {
        let mut v = vec![];
        if text.trim().is_empty() {
            return Ok(AInput(v));
        }

        for txt in text.split(",") {
            let term = crate::manipulation::parse::parse_lambda_read_to_end(txt.trim())?;
            v.push(term);
        }
        Ok(AInput(v))
    }
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let mut first = true;
        for term in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            term.write_fmt(f)?;
            first = false;
        }
        Ok(())
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
            crate::machine::step(&marked, rinput.as_usize()).ok_or("No redex found at the given index")?;
        *self = lambda;
        if is_normal_form(self) {
            Ok(Some(self.clone()))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> Self::SnapShot {
        crate::machine::mark_redex(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_test() {
        let code = r"\x. \y. y";
        let e = LambdaTerm::parse(code).unwrap();
        eprintln!("Parsed term: {}", e);
    }
}
