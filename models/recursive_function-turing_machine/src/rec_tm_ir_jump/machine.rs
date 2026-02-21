use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde_json::json;
use turing_machine::machine::{Direction, Sign, Tape};
use utils::{Machine, TextCodec, json_text};

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
    Assign {
        dst: LValue,
        src: RValue,
    },
    Jump {
        target: usize,
        cond: Option<Condition>,
    },
}

#[derive(Debug, Clone)]
pub enum LValue {
    Var(String),
    Head,
}

#[derive(Debug, Clone)]
pub enum RValue {
    Var(String),
    Head,
    Const(Sign),
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub left: RValue,
    pub right: RValue,
}

#[macro_export]
macro_rules! lv_jr {
    (@) => {
        $crate::rec_tm_ir_jump::LValue::Head
    };
    ($x: literal) => {
        $crate::rec_tm_ir_jump::LValue::Var($x.into())
    };
}

#[macro_export]
macro_rules! rv_jr {
    (@) => {
        $crate::rec_tm_ir_jump::RValue::Head
    };
    ($x: literal) => {
        $crate::rec_tm_ir_jump::RValue::Var($x.into())
    };
    (const $x: expr) => {
        $crate::rec_tm_ir_jump::RValue::Const($x.into())
    };
}

#[macro_export]
macro_rules! cond_jr {
    ($x: expr, $y: expr) => {
        $crate::rec_tm_ir_jump::Condition {
            left: $x,
            right: $y,
        }
    };
}

#[macro_export]
macro_rules! assign_jr {
    ($l: expr, $r: expr) => {
        $crate::rec_tm_ir_jump::Stmt::Assign { dst: $l, src: $r }
    };
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
        self.values.get(var).cloned().unwrap_or_else(Sign::blank)
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
}

impl Machine for RecTmIrJumpMachine {
    type Code = Program;
    type AInput = Tape;
    type SnapShot = Snapshot;
    type RInput = ();
    type Output = Tape;

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
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        if self.pc >= self.program.body.len() {
            return Ok(Some(self.tape.clone()));
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
            Stmt::Assign { dst, src } => {
                let value = self.eval_rvalue(&src)?;
                self.assign_lvalue(&dst, value)?;
                self.pc += 1;
            }
            Stmt::Jump { target, cond } => {
                if target >= self.program.body.len() {
                    return Err(format!("jump target out of range: {}", target));
                }
                if self.eval_condition(&cond)? {
                    self.pc = target;
                } else {
                    self.pc += 1;
                }
            }
        }
        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        fn render_lvalue(value: &LValue) -> String {
            match value {
                LValue::Var(name) => name.clone(),
                LValue::Head => "@".to_string(),
            }
        }
        fn render_rvalue(value: &RValue) -> String {
            match value {
                RValue::Var(name) => name.clone(),
                RValue::Head => "@".to_string(),
                RValue::Const(sign) => format!("const {}", sign.print()),
            }
        }

        let instruction = self.program.body.get(self.pc).map(|stmt| match stmt {
            Stmt::Lt => "LT".to_string(),
            Stmt::Rt => "RT".to_string(),
            Stmt::Assign { dst, src } => {
                format!("{} := {}", render_lvalue(dst), render_rvalue(src))
            }
            Stmt::Jump { target, cond } => match cond {
                Some(cond) => format!(
                    "jump if {} == {} {}",
                    render_rvalue(&cond.left),
                    render_rvalue(&cond.right),
                    target
                ),
                None => format!("jump {}", target),
            },
        });
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
        if let Stmt::Jump { target, .. } = stmt
            && *target >= len
        {
            return Err(format!("jump target out of range at {}: {}", idx, target));
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
        match stmt {
            Stmt::Assign { dst: _, src } => validate_rvalue(src, allowed)?,
            Stmt::Jump {
                cond: Some(cond), ..
            } => {
                validate_rvalue(&cond.left, allowed)?;
                validate_rvalue(&cond.right, allowed)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_rvalue(value: &RValue, allowed: &HashSet<Sign>) -> Result<(), String> {
    if let RValue::Const(sign) = value
        && !allowed.contains(sign)
    {
        return Err(format!("Unknown sign in const: {}", sign.print()));
    }
    Ok(())
}

fn collect_vars(stmts: &[Stmt]) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    for stmt in stmts {
        match stmt {
            Stmt::Assign { dst, src } => {
                if let LValue::Var(name) = dst {
                    vars.insert(name.clone());
                }
                if let RValue::Var(name) = src {
                    vars.insert(name.clone());
                }
            }
            Stmt::Jump {
                cond: Some(cond), ..
            } => {
                if let RValue::Var(name) = &cond.left {
                    vars.insert(name.clone());
                }
                if let RValue::Var(name) = &cond.right {
                    vars.insert(name.clone());
                }
            }
            _ => {}
        }
    }
    vars
}

impl RecTmIrJumpMachine {
    fn eval_rvalue(&self, value: &RValue) -> Result<Sign, String> {
        let sign = match value {
            RValue::Var(name) => self.env.get(name),
            RValue::Head => self.tape.head_read().clone(),
            RValue::Const(sign) => sign.clone(),
        };
        Ok(sign)
    }

    fn assign_lvalue(&mut self, dst: &LValue, value: Sign) -> Result<(), String> {
        match dst {
            LValue::Var(name) => self.env.set(name, value),
            LValue::Head => self.tape.head_write(&value),
        }
        Ok(())
    }

    fn eval_condition(&self, cond: &Option<Condition>) -> Result<bool, String> {
        let Some(cond) = cond else {
            return Ok(true);
        };
        let left = self.eval_rvalue(&cond.left)?;
        let right = self.eval_rvalue(&cond.right)?;
        Ok(left == right)
    }
}
