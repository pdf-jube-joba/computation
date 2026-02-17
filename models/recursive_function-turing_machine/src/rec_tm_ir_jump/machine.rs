use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde_json::json;
use turing_machine::machine::{Direction, Sign, Tape};
use utils::{json_text, Machine, TextCodec};

use super::parser::parse_identifier;

#[derive(Debug, Clone)]
pub struct Program {
    pub alphabet: Vec<Sign>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Lt,
    Rt,
    Read(String),
    Stor(String),
    StorConst(Sign),
    Assign(String, String),
    ConstAssign(String, Sign),
    Jump(usize),
    JumpIf {
        var: String,
        value: Sign,
        target: usize,
    },
}

impl Stmt {
    fn render(&self) -> String {
        match self {
            Stmt::Lt => "LT".to_string(),
            Stmt::Rt => "RT".to_string(),
            Stmt::Read(var) => format!("READ {}", var),
            Stmt::Stor(var) => format!("STOR {}", var),
            Stmt::StorConst(value) => format!("STOR const {}", value.print()),
            Stmt::Assign(dst, src) => format!("{} := {}", dst, src),
            Stmt::ConstAssign(dst, value) => format!("{} := const {}", dst, value.print()),
            Stmt::Jump(target) => format!("jump {}", target),
            Stmt::JumpIf { var, value, target } => {
                format!("jump if {} == {} {}", var, value.print(), target)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    values: BTreeMap<String, Sign>,
}

impl Environment {
    fn new(vars: impl IntoIterator<Item = String>) -> Self {
        let mut values = BTreeMap::new();
        for var in vars {
            values.insert(var, Sign::blank());
        }
        Environment { values }
    }

    fn get(&self, var: &str) -> Sign {
        self.values
            .get(var)
            .cloned()
            .unwrap_or_else(Sign::blank)
    }

    fn set(&mut self, var: &str, value: Sign) {
        self.values.insert(var.to_string(), value);
    }

    fn entries(&self) -> Vec<(String, Sign)> {
        self.values
            .iter()
            .map(|(var, value)| (var.clone(), value.clone()))
            .collect()
    }
}

impl TextCodec for Environment {
    fn parse(text: &str) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, '=');
            let var_str = parts
                .next()
                .ok_or_else(|| "Invalid environment line".to_string())?
                .trim();
            let value_str = parts
                .next()
                .ok_or_else(|| "Invalid environment line".to_string())?
                .trim();
            let var = parse_identifier(var_str)?;
            let value = <Sign as TextCodec>::parse(value_str)?;
            values.insert(var, value);
        }
        Ok(Environment { values })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (idx, (var, value)) in self.entries().iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "{} = {}", var, value.print())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pc: usize,
    instruction: Option<String>,
    env: Environment,
    tape: Tape,
}

impl From<Snapshot> for serde_json::Value {
    fn from(snapshot: Snapshot) -> Self {
        let pc_text = json_text!(snapshot.pc.to_string(), title: "pc");
        let instruction_text = json_text!(
            snapshot
                .instruction
                .unwrap_or_else(|| "halt".to_string()),
            title: "next"
        );

        let env_rows: Vec<serde_json::Value> = snapshot
            .env
            .entries()
            .into_iter()
            .map(|(var, value)| {
                json!({
                    "cells": [
                        json_text!(var),
                        json_text!(value.print())
                    ]
                })
            })
            .collect();
        let env_table = json!({
            "kind": "table",
            "title": "env",
            "columns": [json_text!("var"), json_text!("value")],
            "rows": env_rows
        });

        let (tapes, head_pos) = snapshot.tape.into_vec();
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

        json!([pc_text, instruction_text, env_table, tape_container])
    }
}

pub struct RecTmIrJumpMachine {
    program: Program,
    pc: usize,
    env: Environment,
    tape: Tape,
    allowed: HashSet<Sign>,
}

impl Machine for RecTmIrJumpMachine {
    type Code = Program;
    type AInput = Tape;
    type SnapShot = Snapshot;
    type RInput = ();
    type Output = Environment;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        validate_program(&code)?;
        let alphabet = validate_alphabet(&code.alphabet)?;
        let vars = collect_vars(&code.body);
        let env = Environment::new(vars);
        let allowed = alphabet_set(&alphabet);
        validate_signs_in_program(&code, &allowed)?;
        validate_tape(&ainput, &allowed)?;
        Ok(RecTmIrJumpMachine {
            program: code,
            pc: 0,
            env,
            tape: ainput,
            allowed,
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        if self.pc >= self.program.body.len() {
            return Ok(Some(self.env.clone()));
        }
        let instr = self.program.body[self.pc].clone();
        match instr {
            Stmt::Lt => {
                self.tape.move_to(&Direction::Left);
                self.pc += 1;
            }
            Stmt::Rt => {
                self.tape.move_to(&Direction::Right);
                self.pc += 1;
            }
            Stmt::Read(var) => {
                let sign = self.tape.head_read().clone();
                if !self.allowed.contains(&sign) {
                    return Err(format!("Unknown sign on tape: {}", sign.print()));
                }
                self.env.set(&var, sign);
                self.pc += 1;
            }
            Stmt::Stor(var) => {
                let sign = self.env.get(&var);
                if !self.allowed.contains(&sign) {
                    return Err(format!("Unknown sign in env: {}", sign.print()));
                }
                self.tape.head_write(&sign);
                self.pc += 1;
            }
            Stmt::StorConst(value) => {
                if !self.allowed.contains(&value) {
                    return Err(format!("Unknown sign in const: {}", value.print()));
                }
                self.tape.head_write(&value);
                self.pc += 1;
            }
            Stmt::Assign(dest, src) => {
                let value = self.env.get(&src);
                self.env.set(&dest, value);
                self.pc += 1;
            }
            Stmt::ConstAssign(dest, value) => {
                if !self.allowed.contains(&value) {
                    return Err(format!("Unknown sign in const: {}", value.print()));
                }
                self.env.set(&dest, value);
                self.pc += 1;
            }
            Stmt::Jump(target) => {
                if target >= self.program.body.len() {
                    return Err(format!("jump target out of range: {}", target));
                }
                self.pc = target;
            }
            Stmt::JumpIf { var, value, target } => {
                if target >= self.program.body.len() {
                    return Err(format!("jump target out of range: {}", target));
                }
                if self.env.get(&var) == value {
                    self.pc = target;
                } else {
                    self.pc += 1;
                }
            }
        }
        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        let instruction = self.program.body.get(self.pc).map(|stmt| stmt.render());
        Snapshot {
            pc: self.pc,
            instruction,
            env: self.env.clone(),
            tape: self.tape.clone(),
        }
    }
}

fn validate_program(program: &Program) -> Result<(), String> {
    let len = program.body.len();
    for (idx, stmt) in program.body.iter().enumerate() {
        match stmt {
            Stmt::Jump(target) | Stmt::JumpIf { target, .. } => {
                if *target >= len {
                    return Err(format!(
                        "jump target out of range at {}: {}",
                        idx, target
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_alphabet(alphabet: &[Sign]) -> Result<Vec<Sign>, String> {
    if alphabet.is_empty() {
        return Err("alphabet must not be empty".to_string());
    }
    if alphabet.iter().any(|sign| *sign == Sign::blank()) {
        return Ok(alphabet.to_vec());
    }
    let mut extended = alphabet.to_vec();
    extended.push(Sign::blank());
    Ok(extended)
}

fn alphabet_set(alphabet: &[Sign]) -> HashSet<Sign> {
    alphabet.iter().cloned().collect()
}

fn validate_tape(tape: &Tape, allowed: &HashSet<Sign>) -> Result<(), String> {
    let (signs, _) = tape.into_vec();
    for sign in signs {
        if !allowed.contains(&sign) {
            return Err(format!("Unknown sign on tape: {}", sign.print()));
        }
    }
    Ok(())
}

fn validate_signs_in_program(program: &Program, allowed: &HashSet<Sign>) -> Result<(), String> {
    for stmt in &program.body {
        if let Stmt::JumpIf { value, .. } = stmt {
            if !allowed.contains(value) {
                return Err(format!("Unknown sign in jump if: {}", value.print()));
            }
        } else if let Stmt::ConstAssign(_, value) | Stmt::StorConst(value) = stmt {
            if !allowed.contains(value) {
                return Err(format!("Unknown sign in const: {}", value.print()));
            }
        }
    }
    Ok(())
}

fn collect_vars(stmts: &[Stmt]) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    for stmt in stmts {
        match stmt {
            Stmt::Read(var) | Stmt::Stor(var) | Stmt::JumpIf { var, .. } => {
                vars.insert(var.clone());
            }
            Stmt::Assign(dst, src) => {
                vars.insert(dst.clone());
                vars.insert(src.clone());
            }
            Stmt::ConstAssign(dst, _) => {
                vars.insert(dst.clone());
            }
            _ => {}
        }
    }
    vars
}
