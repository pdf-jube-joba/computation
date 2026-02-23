pub mod machine;
pub mod manipulation;

use crate::machine::{is_normal_form, LambdaTerm, MarkedTerm};
use serde::Serialize;
use serde_json::json;
use utils::{json_text, Machine, StepResult, TextCodec};

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
    type RInput = usize;
    type ROutput = ();
    type FOutput = LambdaTerm;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        let term = crate::machine::assoc_app(code, ainput.0);
        Ok(term)
    }

    fn step(self, rinput: Self::RInput) -> Result<StepResult<Self>, String> {
        let marked = crate::machine::mark_redex(&self);
        let lambda =
            crate::machine::step(&marked, rinput).ok_or("No redex found at the given index")?;
        if is_normal_form(&lambda) {
            let snapshot = crate::machine::mark_redex(&lambda);
            Ok(StepResult::Halt {
                snapshot,
                output: lambda,
            })
        } else {
            Ok(StepResult::Continue {
                next: lambda,
                output: (),
            })
        }
    }

    fn current(&self) -> Self::SnapShot {
        crate::machine::mark_redex(self)
    }
}

impl From<MarkedTerm> for serde_json::Value {
    fn from(term: MarkedTerm) -> Self {
        fn term_block(term: MarkedTerm) -> serde_json::Value {
            match term {
                MarkedTerm::Var(var) => json_text!(var.as_str()),
                MarkedTerm::Abs(var, body) => {
                    let children = vec![
                        json_text!("\\"),
                        term_block(MarkedTerm::Var(var)),
                        json_text!("."),
                        term_block(*body),
                    ];
                    json!({
                        "kind": "container",
                        "orientation": "horizontal",
                        "display": "inline",
                        "children": children
                    })
                }
                MarkedTerm::App(lhs, rhs) => {
                    let children = vec![term_block(*lhs), json_text!(" "), term_block(*rhs)];
                    json!({
                        "kind": "container",
                        "orientation": "horizontal",
                        "display": "inline",
                        "children": children
                    })
                }
                MarkedTerm::Red(var, abs_term, app_term) => {
                    let children = vec![
                        json_text!(format!("\\ {}.", var.as_str())),
                        term_block(*abs_term),
                        term_block(*app_term),
                    ];
                    json!({
                        "kind": "container",
                        "orientation": "horizontal",
                        "display": "block",
                        "children": children
                    })
                }
            }
        }

        json!([term_block(term)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_test() {
        let code = r"\x. \y. y";
        let e = LambdaTerm::parse(code).unwrap();
        eprintln!("Parsed term: {}", e.print());
    }
}
