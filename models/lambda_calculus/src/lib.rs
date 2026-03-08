pub mod machine;
pub mod manipulation;

use crate::machine::{is_normal_form, LambdaTerm, MarkedTerm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utils::identifier::Var;
use utils::{Machine, StepResult, TextCodec};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SnapshotTerm {
    Var(usize),
    Abs(usize, Box<SnapshotTerm>),
    App(Box<SnapshotTerm>, Box<SnapshotTerm>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub term: SnapshotTerm,
    pub table: HashMap<usize, String>,
}

fn encode_snapshot_term(term: &LambdaTerm, table: &mut HashMap<usize, String>) -> SnapshotTerm {
    match term {
        LambdaTerm::Var(var) => {
            let id = var.as_ptr_usize();
            table.entry(id).or_insert_with(|| var.as_str().to_string());
            SnapshotTerm::Var(id)
        }
        LambdaTerm::Abs(var, body) => {
            let id = var.as_ptr_usize();
            table.entry(id).or_insert_with(|| var.as_str().to_string());
            SnapshotTerm::Abs(id, Box::new(encode_snapshot_term(body, table)))
        }
        LambdaTerm::App(lhs, rhs) => SnapshotTerm::App(
            Box::new(encode_snapshot_term(lhs, table)),
            Box::new(encode_snapshot_term(rhs, table)),
        ),
    }
}

fn decode_snapshot_term(term: &SnapshotTerm, vars: &mut HashMap<usize, Var>) -> LambdaTerm {
    match term {
        SnapshotTerm::Var(id) => {
            let var = vars
                .entry(*id)
                .or_insert_with(|| Var::new(&format!("v{id}")))
                .clone();
            LambdaTerm::Var(var)
        }
        SnapshotTerm::Abs(id, body) => {
            let var = vars
                .entry(*id)
                .or_insert_with(|| Var::new(&format!("v{id}")))
                .clone();
            LambdaTerm::Abs(var, Box::new(decode_snapshot_term(body, vars)))
        }
        SnapshotTerm::App(lhs, rhs) => LambdaTerm::App(
            Box::new(decode_snapshot_term(lhs, vars)),
            Box::new(decode_snapshot_term(rhs, vars)),
        ),
    }
}

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
    type SnapShot = Snapshot;
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
            Ok(StepResult::Halt { output: lambda })
        } else {
            Ok(StepResult::Continue {
                next: lambda,
                output: (),
            })
        }
    }

    fn snapshot(&self) -> Self::SnapShot {
        let mut table = HashMap::new();
        let term = encode_snapshot_term(self, &mut table);
        Snapshot { term, table }
    }

    fn restore(snapshot: Self::SnapShot) -> Self {
        let mut vars = HashMap::new();
        for (id, name) in snapshot.table {
            vars.insert(id, Var::new(&name));
        }
        decode_snapshot_term(&snapshot.term, &mut vars)
    }

    fn render(snapshot: Self::SnapShot) -> utils::RenderState {
        fn term_block(term: MarkedTerm) -> utils::RenderBlock {
            match term {
                MarkedTerm::Var(var) => utils::render_text!(var.as_str()),
                MarkedTerm::Abs(var, body) => utils::render_container!(
                    children: vec![
                        utils::render_text!("\\"),
                        term_block(MarkedTerm::Var(var)),
                        utils::render_text!("."),
                        term_block(*body)
                    ],
                    orientation: utils::RenderOrientation::Horizontal,
                    display: utils::RenderDisplay::Inline
                ),
                MarkedTerm::App(lhs, rhs) => utils::render_container!(
                    children: vec![term_block(*lhs), utils::render_text!(" "), term_block(*rhs)],
                    orientation: utils::RenderOrientation::Horizontal,
                    display: utils::RenderDisplay::Inline
                ),
                MarkedTerm::Red(var, abs_term, app_term) => utils::render_container!(
                    children: vec![
                        utils::render_text!(format!("\\ {}.", var.as_str())),
                        term_block(*abs_term),
                        term_block(*app_term)
                    ],
                    orientation: utils::RenderOrientation::Horizontal,
                    display: utils::RenderDisplay::Block
                ),
            }
        }

        let restored = Self::restore(snapshot);
        let marked = crate::machine::mark_redex(&restored);
        utils::render_state![term_block(marked)]
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
