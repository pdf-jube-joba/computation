use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use serde_json::json;
use turing_machine::machine::{Direction, Sign, Tape};
use utils::{Machine, TextCodec, json_text};

use super::parser::{parse_identifier, render_text};
use super::validation::{
    alphabet_set, validate_alphabet, validate_no_recursion, validate_signs_in_program,
    validate_tape,
};

#[derive(Debug, Clone)]
pub struct Program {
    pub alphabet: Vec<Sign>,
    pub functions: Vec<Rc<Function>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub label: String,
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
    Break {
        cond: Option<Condition>,
    },
    Continue {
        cond: Option<Condition>,
    },
    Jump {
        label: String,
        cond: Option<Condition>,
    },
    Return {
        cond: Option<Condition>,
    },
    Call {
        func: Rc<Function>,
    },
}

impl PartialEq for Stmt {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Stmt::Lt, Stmt::Lt) => true,
            (Stmt::Rt, Stmt::Rt) => true,
            (
                Stmt::Assign { dst, src },
                Stmt::Assign {
                    dst: o_dst,
                    src: o_src,
                },
            ) => dst == o_dst && src == o_src,
            (Stmt::Break { cond }, Stmt::Break { cond: o_cond }) => cond == o_cond,
            (Stmt::Continue { cond }, Stmt::Continue { cond: o_cond }) => cond == o_cond,
            (
                Stmt::Jump { label, cond },
                Stmt::Jump {
                    label: o_label,
                    cond: o_cond,
                },
            ) => label == o_label && cond == o_cond,
            (Stmt::Return { cond }, Stmt::Return { cond: o_cond }) => cond == o_cond,
            (Stmt::Call { func }, Stmt::Call { func: o_func }) => Rc::ptr_eq(func, o_func),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LValue {
    Var(String),
    Head,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RValue {
    Var(String),
    Head,
    Const(Sign),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub left: RValue,
    pub right: RValue,
}

#[macro_export]
macro_rules! lv {
    (@) => {
        $crate::rec_tm_ir::LValue::Head
    };
    ($x: literal) => {
        $crate::rec_tm_ir::LValue::Var($x.into())
    };
}

#[macro_export]
macro_rules! rv {
    (@) => {
        $crate::rec_tm_ir::RValue::Head
    };
    ($x: literal) => {
        $crate::rec_tm_ir::RValue::Var($x.into())
    };
    (const $x: expr) => {
        $crate::rec_tm_ir::RValue::Const($x.into())
    };
}

#[macro_export]
macro_rules! cond {
    ($l: expr, $r: expr) => {
        Some($crate::rec_tm_ir::Condition {
            left: $l,
            right: $r,
        })
    };
}

#[macro_export]
macro_rules! assign {
    ($l: expr, $r: expr) => {
        $crate::rec_tm_ir::Stmt::Assign { dst: $l, src: $r }
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
    pub function: String,
    pub pc: (usize, usize),
    pub instruction: Option<String>,
    pub env: Environment,
    pub tape: Tape,
    pub stack: Vec<String>,
}

impl From<Snapshot> for serde_json::Value {
    fn from(snapshot: Snapshot) -> Self {
        let fn_text = json_text!(snapshot.function, title: "function");
        let pc_text = json_text!(
            format!("{}:{}", snapshot.pc.0, snapshot.pc.1),
            title: "pc"
        );
        let instruction_text = json_text!(
            snapshot
                .instruction
                .unwrap_or_else(|| "halt".to_string()),
            title: "next"
        );
        let mut stack_children: Vec<serde_json::Value> =
            snapshot.stack.iter().map(|name| json_text!(name)).collect();
        let mut current_block = json_text!(snapshot.function);
        if let Some(map) = current_block.as_object_mut() {
            map.insert("className".to_string(), json!("highlight"));
        }
        stack_children.push(current_block);
        let stack_container = json!({
            "kind": "container",
            "title": "stack",
            "orientation": "horizontal",
            "display": "block",
            "children": stack_children
        });

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

        json!([
            fn_text,
            pc_text,
            instruction_text,
            stack_container,
            env_table,
            tape_container
        ])
    }
}

struct Frame {
    function: String,
    env: Environment,
    blocks: Vec<Block>,
    pc: (usize, usize),
}

pub struct RecTmIrMachine {
    program: Program,
    frame: Frame,
    stack: Vec<Frame>,
    tape: Tape,
}

fn build_frame_for_function(func: &Function) -> Frame {
    let env = Environment::new(collect_vars(&func.blocks));
    Frame {
        function: func.name.clone(),
        env,
        blocks: func.blocks.clone(),
        pc: (0, 0),
    }
}

impl Machine for RecTmIrMachine {
    type Code = Program;
    type AInput = Tape;
    type SnapShot = Snapshot;
    type RInput = ();
    type Output = Tape;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        validate_no_recursion(&code)?;
        let alphabet = validate_alphabet(&code.alphabet)?;
        let main = code
            .functions
            .iter()
            .find(|func| func.name == "main")
            .expect("main() is not defined");
        let allowed = alphabet_set(&alphabet);
        validate_signs_in_program(&code, &allowed)?;
        validate_tape(&ainput, &allowed)?;

        let frame = build_frame_for_function(main.as_ref());

        Ok(RecTmIrMachine {
            program: code.clone(),
            frame,
            stack: Vec::new(),
            tape: ainput,
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let (stmt, block_idx) = match self.next_stmt()? {
            Some(stmt) => stmt,
            None => return Ok(self.return_from_call()),
        };
        match stmt {
            Stmt::Lt => {
                self.tape.move_to(&Direction::Left);
            }
            Stmt::Rt => {
                self.tape.move_to(&Direction::Right);
            }
            Stmt::Assign { dst, src } => {
                let value = self.eval_rvalue(&src);
                self.assign_lvalue(&dst, value);
            }
            Stmt::Break { cond } => {
                if self.eval_condition(&cond) {
                    let next_block = block_idx + 1;
                    self.frame.pc = (next_block, 0);
                }
            }
            Stmt::Continue { cond } => {
                if self.eval_condition(&cond) {
                    self.frame.pc = (block_idx, 0);
                }
            }
            Stmt::Jump { label, cond } => {
                if self.eval_condition(&cond) {
                    self.jump_to(&label)?;
                }
            }
            Stmt::Return { cond } => {
                if self.eval_condition(&cond) {
                    return Ok(self.return_from_call());
                }
            }
            Stmt::Call { func } => {
                let callee = func;
                let caller = Frame {
                    function: self.frame.function.clone(),
                    env: self.frame.env.clone(),
                    blocks: self.frame.blocks.clone(),
                    pc: self.frame.pc,
                };
                self.stack.push(caller);
                self.frame = build_frame_for_function(callee.as_ref());
            }
        }
        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        let instruction = self.peek_stmt().map(|stmt| render_text(&stmt));
        let stack = self
            .stack
            .iter()
            .map(|frame| frame.function.clone())
            .collect();
        Snapshot {
            function: self.frame.function.clone(),
            pc: self.current_pc(),
            instruction,
            env: self.frame.env.clone(),
            tape: self.tape.clone(),
            stack,
        }
    }
}

impl RecTmIrMachine {
    fn return_from_call(&mut self) -> Option<Tape> {
        if let Some(frame) = self.stack.pop() {
            self.frame = frame;
            None
        } else {
            Some(self.tape.clone())
        }
    }

    fn current_pc(&self) -> (usize, usize) {
        self.normalized_pc()
    }

    fn peek_stmt(&self) -> Option<Stmt> {
        let (block_idx, line_idx) = self.normalized_pc();
        let blocks = &self.frame.blocks;
        if block_idx >= blocks.len() {
            return None;
        }
        blocks
            .get(block_idx)
            .and_then(|block| block.body.get(line_idx).cloned())
    }

    fn next_stmt(&mut self) -> Result<Option<(Stmt, usize)>, String> {
        let (block_idx, line_idx) = self.normalized_pc();
        let blocks = &self.frame.blocks;
        if block_idx >= blocks.len() {
            return Ok(None);
        }
        let stmt = blocks[block_idx].body[line_idx].clone();
        self.frame.pc = (block_idx, line_idx + 1);
        Ok(Some((stmt, block_idx)))
    }

    fn jump_to(&mut self, label: &str) -> Result<(), String> {
        let idx = self
            .frame
            .blocks
            .iter()
            .position(|block| block.label == label)
            .expect("jump label not found");
        self.frame.pc = (idx, 0);
        Ok(())
    }

    fn eval_rvalue(&self, value: &RValue) -> Sign {
        match value {
            RValue::Var(var) => self.frame.env.get(var),
            RValue::Head => self.tape.head_read().clone(),
            RValue::Const(sign) => sign.clone(),
        }
    }

    fn assign_lvalue(&mut self, target: &LValue, value: Sign) {
        match target {
            LValue::Var(var) => self.frame.env.set(var, value),
            LValue::Head => self.tape.head_write(&value),
        }
    }

    fn eval_condition(&self, cond: &Option<Condition>) -> bool {
        let Some(cond) = cond else {
            return true;
        };
        self.eval_rvalue(&cond.left) == self.eval_rvalue(&cond.right)
    }

    fn normalized_pc(&self) -> (usize, usize) {
        let mut block_idx = self.frame.pc.0;
        let mut line_idx = self.frame.pc.1;
        let blocks = &self.frame.blocks;
        loop {
            if block_idx >= blocks.len() {
                return (block_idx, line_idx);
            }
            let block = &blocks[block_idx];
            if line_idx >= block.body.len() {
                block_idx += 1;
                line_idx = 0;
                continue;
            }
            return (block_idx, line_idx);
        }
    }
}

fn collect_vars(blocks: &[Block]) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    for block in blocks {
        collect_vars_inner(&block.body, &mut vars);
    }
    vars
}

fn collect_vars_inner(stmts: &[Stmt], vars: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Assign { dst, src } => {
                if let LValue::Var(var) = dst {
                    vars.insert(var.clone());
                }
                collect_vars_rvalue(src, vars);
            }
            Stmt::Break { cond }
            | Stmt::Continue { cond }
            | Stmt::Jump { cond, .. }
            | Stmt::Return { cond } => {
                if let Some(cond) = cond {
                    collect_vars_rvalue(&cond.left, vars);
                    collect_vars_rvalue(&cond.right, vars);
                }
            }
            Stmt::Call { .. } => {}
            Stmt::Lt | Stmt::Rt => {}
        }
    }
}

fn collect_vars_rvalue(value: &RValue, vars: &mut BTreeSet<String>) {
    if let RValue::Var(var) = value {
        vars.insert(var.clone());
    }
}
