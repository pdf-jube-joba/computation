use crate::machine::{CodeEntry as CoreCodeEntry, Tape, TuringMachineDefinition, TuringMachineSet};
use serde::Serialize;
use utils::{Machine, TextCodec};

pub mod machine;
pub mod manipulation;
pub mod parse;
#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, Serialize)]
pub struct CodeEntry {
    key_sign: String,
    key_state: String,
    next_sign: String,
    next_state: String,
    direction: String,
}

impl From<CoreCodeEntry> for CodeEntry {
    fn from(entry: CoreCodeEntry) -> Self {
        CodeEntry {
            key_sign: entry.0.0.print(),
            key_state: entry.0.1.print(),
            next_sign: entry.1.0.print(),
            next_state: entry.1.1.print(),
            direction: entry.1.2.print(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Current {
    code: Vec<CodeEntry>,
    now: usize,
    state: String,
    tape: Tape,
}

impl Machine for TuringMachineSet {
    type Code = TuringMachineDefinition;
    type AInput = Tape;
    type RInput = ();
    type Output = ();
    type SnapShot = Current;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(TuringMachineSet::new(code, ainput))
    }

    fn step(&mut self, _input: Self::RInput) -> Result<Option<Self::Output>, String> {
        let _ = self.step(1);
        if self.is_terminate() {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> Self::SnapShot {
        let now = self
            .next_code()
            .map(|(idx, _)| idx)
            .unwrap_or(self.code().len());
        let tape = self.now_tape().clone();
        let state = self.now_state().print();
        Current {
            code: self.code().iter().cloned().map(CodeEntry::from).collect(),
            now,
            state,
            tape,
        }
    }
}
