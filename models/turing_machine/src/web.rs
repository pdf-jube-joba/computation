use crate::{
    machine::{CodeEntry as CoreCodeEntry, Sign, Tape, TuringMachineDefinition, TuringMachineSet},
    manipulation,
};
use serde::Serialize;
use utils::Machine;

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
            key_sign: entry.0 .0.to_string(),
            key_state: entry.0 .1.to_string(),
            next_sign: entry.1 .0.to_string(),
            next_state: entry.1 .1.to_string(),
            direction: entry.1 .2.to_string(),
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
    type This = Current;

    fn parse_code(input: &str) -> Result<Self::Code, String> {
        let definition =
            manipulation::code::parse_definition(input).map_err(|e| format!("{e:?}"))?;
        Ok(definition)
    }

    fn parse_ainput(input: &str) -> Result<Self::AInput, String> {
        parse_tape(input)
    }

    fn parse_rinput(_input: &str) -> Result<Self::RInput, String> {
        Ok(())
    }

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(TuringMachineSet::new(code, ainput))
    }

    fn step(&mut self, _input: Self::RInput) -> Result<Option<Self::Output>, String> {
        match self.step(1) {
            Ok(_) => Ok(None),
            Err(err) => Err(format!("{err}")),
        }
    }

    fn current(&self) -> Self::This {
        let now = self
            .next_code()
            .map(|(idx, _)| idx)
            .unwrap_or(self.code().len());
        let tape = self.now_tape().clone();
        let state = self.now_state().to_string();

        Current {
            code: self.code().iter().cloned().map(CodeEntry::from).collect(),
            now,
            state,
            tape,
        }
    }
}

fn parse_tape(tape: &str) -> Result<Tape, String> {
    let parts: Vec<&str> = tape.split('|').collect();
    if parts.len() != 3 {
        return Err("Invalid tape format | format ... 0,1,2|3|4,5,6".to_string());
    }
    let left: Vec<Sign> = parts
        .first()
        .ok_or_else(|| "Missing left part".to_string())?
        .split(',')
        .map(|s| s.trim().parse().map_err(|e| format!("{e}")))
        .collect::<Result<_, _>>()?;
    let head: Sign = parts[1].trim().parse().map_err(|e| format!("{e}"))?;
    let mut right: Vec<Sign> = parts
        .get(2)
        .ok_or_else(|| "Missing right part".to_string())?
        .split(',')
        .map(|s| s.trim().parse().map_err(|e| format!("{e}")))
        .collect::<Result<_, _>>()?;
    right.reverse();
    Ok(Tape::new(left, head, right))
}
