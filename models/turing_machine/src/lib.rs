use crate::machine::{CodeEntry as CoreCodeEntry, Tape, TuringMachine, TuringMachineDefinition};
use serde::Serialize;
use serde_json::json;
use utils::{Machine, StepResult, TextCodec, json_text};

pub mod machine;
pub mod manipulation;
pub mod parse;
#[cfg(test)]
pub mod tests;

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

impl From<Current> for serde_json::Value {
    fn from(current: Current) -> Self {
        let now = current.now;
        let rows: Vec<serde_json::Value> = current
            .code
            .into_iter()
            .enumerate()
            .map(|(idx, entry)| {
                let mut row = json!({
                    "cells": [
                        json_text!(entry.key_sign),
                        json_text!(entry.key_state),
                        json_text!(entry.next_sign),
                        json_text!(entry.next_state),
                        json_text!(entry.direction)
                    ]
                });
                if idx == now
                    && let Some(map) = row.as_object_mut()
                {
                    map.insert("className".to_string(), json!("highlight"));
                }

                row
            })
            .collect();

        let code_table = json!({
            "kind": "table",
            "title": "code",
            "columns": [
                json_text!("key_sign"),
                json_text!("key_state"),
                json_text!("next_sign"),
                json_text!("next_state"),
                json_text!("direction")
            ],
            "rows": rows
        });

        let state_text = json_text!(current.state, title: "state");

        let (tapes, head_pos) = current.tape.into_vec();
        let tape_children: Vec<serde_json::Value> = tapes
            .into_iter()
            .enumerate()
            .map(|(idx, sign)| {
                let mut block = json_text!(sign.print());
                if idx == head_pos
                    && let Some(map) = block.as_object_mut()
                {
                    map.insert("className".to_string(), json!("highlight"));
                }
                block
            })
            .collect();
        let tape_container = json!({
            "kind": "container",
            "title": "tape",
            "orientation": "horizontal",
            "display": "block",
            "children": tape_children
        });

        json!([code_table, state_text, tape_container])
    }
}

impl Machine for TuringMachine {
    type Code = TuringMachineDefinition;
    type AInput = Tape;
    type RInput = ();
    type SnapShot = Current;
    type ROutput = ();
    type FOutput = Tape;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(TuringMachine::new(code, ainput))
    }

    fn step(self, _input: Self::RInput) -> Result<StepResult<Self>, String> {
        let mut machine = self;
        let _ = TuringMachine::step(&mut machine, 1);
        if machine.is_terminate() {
            let snapshot = machine.current();
            let output = snapshot.tape.clone();
            Ok(StepResult::Halt { snapshot, output })
        } else {
            Ok(StepResult::Continue {
                next: machine,
                output: (),
            })
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
