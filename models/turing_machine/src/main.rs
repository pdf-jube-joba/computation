use serde::Serialize;
use turing_machine::machine::{
    CodeEntry as CoreCodeEntry, Tape, TuringMachineDefinition, TuringMachineSet,
};
use utils::{Machine, TextCodec};

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
            key_sign: entry.0 .0.print(),
            key_state: entry.0 .1.print(),
            next_sign: entry.1 .0.print(),
            next_state: entry.1 .1.print(),
            direction: entry.1 .2.print(),
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

pub struct TuringMachine(TuringMachineSet);

impl Machine for TuringMachine {
    type Code = TuringMachineDefinition;
    type AInput = Tape;
    type RInput = ();
    type Output = ();
    type SnapShot = Current;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(TuringMachine(TuringMachineSet::new(code, ainput)))
    }

    fn step(&mut self, _input: Self::RInput) -> Result<Option<Self::Output>, String> {
        let _ = self.0.step(1);
        if self.0.is_terminate() {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> Self::SnapShot {
        let now = self
            .0
            .next_code()
            .map(|(idx, _)| idx)
            .unwrap_or(self.0.code().len());
        let tape = self.0.now_tape().clone();
        let state = self.0.now_state().print();
        Current {
            code: self.0.code().iter().cloned().map(CodeEntry::from).collect(),
            now,
            state,
            tape,
        }
    }
}

web_builder::web_model!(TuringMachine);
