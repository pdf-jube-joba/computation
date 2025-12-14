use crate::{
    machine::{CodeEntry as CoreCodeEntry, Sign, Tape, TuringMachineDefinition, TuringMachineSet},
    manipulation,
};
use serde::Serialize;
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

impl TextCodec for TuringMachineDefinition {
    fn parse(text: &str) -> Result<Self, String> {
        let definition =
            manipulation::code::parse_definition(text).map_err(|e| format!("{e:?}"))?;
        Ok(definition)
    }

    fn print(data: &Self) -> String {
        let init_state = data.init_state();
        let accepted_states = data.accepted_state();
        let code = data.code();

        let mut s = String::new();
        s.push_str(&format!("{}\n", init_state));
        s.push_str(&format!(
            "{}\n",
            accepted_states
                .iter()
                .map(|st| st.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ));
        for entry in code {
            let line = format!(
                "{},{},{},{},{}\n",
                entry.0 .0, entry.0 .1, entry.1 .0, entry.1 .1, entry.1 .2
            );
            s.push_str(&line);
        }
        s.trim_end().to_string()
    }
}

impl TextCodec for Tape {
    fn parse(text: &str) -> Result<Self, String> {
        parse_tape(text)
    }

    fn print(data: &Self) -> String {
        let (tapes, pos) = data.into_vec();
        let mut s = String::new();
        for (i, sign) in tapes.iter().enumerate() {
            if i == pos {
                s.push_str(&format!("|{}|", sign));
            } else {
                s.push_str(&format!("{},", sign));
            }
        }
        s.trim_end_matches(',').to_string()
    }
}

impl Machine for TuringMachineSet {
    type Code = TuringMachineDefinition;
    type AInput = Tape;
    type RInput = ();
    type Output = ();
    type SnapShot = Current;

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
    let mut v = vec![];
    for s in parts[0].split(',') {
        let sign: Sign = s.trim().parse().map_err(|e| format!("{e}"))?;
        v.push(sign);
    }
    let pos = v.len();
    let head: Sign = parts[1].trim().parse().map_err(|e| format!("{e}"))?;
    v.push(head.clone());
    for s in parts[2].split(',') {
        let sign: Sign = s.trim().parse().map_err(|e| format!("{e}"))?;
        v.push(sign);
    }
    Ok(Tape::from_vec(v, pos))
}
