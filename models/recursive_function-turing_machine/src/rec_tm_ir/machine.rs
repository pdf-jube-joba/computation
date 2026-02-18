use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde_json::json;
use turing_machine::machine::{Direction, Sign, Tape};
use utils::{Machine, TextCodec, json_text};

use super::parser::parse_identifier;
use super::validation::{
    alphabet_set, validate_alphabet, validate_loops, validate_no_recursion,
    validate_signs_in_program, validate_tape,
};

#[derive(Debug, Clone)]
pub struct Program {
    pub alphabet: Vec<Sign>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct CallArg {
    pub shared: bool,
    pub name: String,
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
    IfBreak {
        var: String,
        value: Sign,
        label: String,
    },
    IfBreakHead {
        value: Sign,
        label: String,
    },
    Loop {
        label: String,
        body: Vec<Stmt>,
    },
    Call {
        name: String,
        args: Vec<CallArg>,
    },
}

fn render_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Lt => "LT".to_string(),
        Stmt::Rt => "RT".to_string(),
        Stmt::Read(var) => format!("READ {}", var),
        Stmt::Stor(var) => format!("STOR {}", var),
        Stmt::StorConst(value) => format!("STOR const {}", value.print()),
        Stmt::Assign(dst, src) => format!("{} := {}", dst, src),
        Stmt::ConstAssign(dst, value) => format!("{} := const {}", dst, value.print()),
        Stmt::IfBreak { var, value, label } => {
            format!("if {} == {} break {}", var, value.print(), label)
        }
        Stmt::IfBreakHead { value, label } => {
            format!("if @ == {} break {}", value.print(), label)
        }
        Stmt::Loop { label, .. } => format!("loop {}", label),
        Stmt::Call { name, args } => {
            let rendered = args
                .iter()
                .map(|arg| {
                    if arg.shared {
                        format!("&{}", arg.name)
                    } else {
                        arg.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("call {}({})", name, rendered)
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
    function: String,
    pc: usize,
    instruction: Option<String>,
    env: Environment,
    tape: Tape,
    stack: Vec<String>,
}

impl From<Snapshot> for serde_json::Value {
    fn from(snapshot: Snapshot) -> Self {
        let fn_text = json_text!(snapshot.function, title: "function");
        let pc_text = json_text!(snapshot.pc.to_string(), title: "pc");
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

#[derive(Debug, Clone)]
struct BlockFrame {
    stmts: Vec<Stmt>,
    pc: usize,
    loop_label: Option<String>,
}

impl BlockFrame {
    fn new(stmts: Vec<Stmt>, loop_label: Option<String>) -> Self {
        BlockFrame {
            stmts,
            pc: 0,
            loop_label,
        }
    }
}

struct Frame {
    function: String,
    env: Environment,
    shared: HashMap<String, String>,
    caller_depth: Option<usize>,
    blocks: Vec<BlockFrame>,
}

pub struct RecTmIrMachine {
    program: Program,
    frame: Frame,
    stack: Vec<Frame>,
    tape: Tape,
    allowed: HashSet<Sign>,
}

impl Machine for RecTmIrMachine {
    type Code = Program;
    type AInput = Tape;
    type SnapShot = Snapshot;
    type RInput = ();
    type Output = Environment;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        validate_no_recursion(&code)?;
        validate_loops(&code)?;
        let alphabet = validate_alphabet(&code.alphabet)?;
        let main = code
            .functions
            .iter()
            .find(|func| func.name == "main")
            .ok_or_else(|| "main() is not defined".to_string())?;
        let env = Environment::new(collect_vars(&main.body));
        let allowed = alphabet_set(&alphabet);
        validate_signs_in_program(&code, &allowed)?;
        validate_tape(&ainput, &allowed)?;

        Ok(RecTmIrMachine {
            program: code.clone(),
            frame: Frame {
                function: "main".to_string(),
                env,
                shared: HashMap::new(),
                caller_depth: None,
                blocks: vec![BlockFrame::new(main.body.clone(), None)],
            },
            stack: Vec::new(),
            tape: ainput,
            allowed,
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let stmt = match self.next_stmt()? {
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
            Stmt::Read(var) => {
                let sign = self.tape.head_read().clone();
                if !self.allowed.contains(&sign) {
                    return Err(format!("Unknown sign on tape: {}", sign.print()));
                }
                self.set_var(&var, sign);
            }
            Stmt::Stor(var) => {
                let sign = self.get_var(&var);
                if !self.allowed.contains(&sign) {
                    return Err(format!("Unknown sign in env: {}", sign.print()));
                }
                self.tape.head_write(&sign);
            }
            Stmt::StorConst(value) => {
                if !self.allowed.contains(&value) {
                    return Err(format!("Unknown sign in const: {}", value.print()));
                }
                self.tape.head_write(&value);
            }
            Stmt::Assign(dest, src) => {
                let value = self.get_var(&src);
                self.set_var(&dest, value);
            }
            Stmt::ConstAssign(dest, value) => {
                if !self.allowed.contains(&value) {
                    return Err(format!("Unknown sign in const: {}", value.print()));
                }
                self.set_var(&dest, value);
            }
            Stmt::IfBreak { var, value, label } => {
                if self.get_var(&var) == value {
                    self.break_loop(&label)?;
                }
            }
            Stmt::IfBreakHead { value, label } => {
                if self.tape.head_read() == &value {
                    self.break_loop(&label)?;
                }
            }
            Stmt::Loop { label, body } => {
                self.frame.blocks.push(BlockFrame::new(body, Some(label)));
            }
            Stmt::Call { name, args } => {
                let callee = self
                    .program
                    .functions
                    .iter()
                    .find(|func| func.name == name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined function '{}'", name))?;
                if callee.params.len() != args.len() {
                    return Err(format!(
                        "Function '{}' expects {} args, got {}",
                        name,
                        callee.params.len(),
                        args.len()
                    ));
                }
                let mut next_env = Environment::new(collect_vars(&callee.body));
                let mut shared = HashMap::new();
                for (param, arg) in callee.params.clone().iter().zip(args.iter()) {
                    let value = self.get_var(&arg.name);
                    if arg.shared {
                        shared.insert(param.clone(), arg.name.clone());
                    }
                    next_env.set(param, value);
                }
                let caller = Frame {
                    function: self.frame.function.clone(),
                    env: self.frame.env.clone(),
                    shared: self.frame.shared.clone(),
                    caller_depth: self.frame.caller_depth,
                    blocks: self.frame.blocks.clone(),
                };
                self.stack.push(caller);
                let caller_depth = Some(self.stack.len() - 1);
                self.frame = Frame {
                    function: name,
                    env: next_env,
                    shared,
                    caller_depth,
                    blocks: vec![BlockFrame::new(callee.body.clone(), None)],
                };
            }
        }
        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        let instruction = self.peek_stmt().map(|stmt| render_stmt(&stmt));
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
    fn return_from_call(&mut self) -> Option<Environment> {
        if let Some(frame) = self.stack.pop() {
            self.frame = frame;
            None
        } else {
            Some(self.frame.env.clone())
        }
    }

    fn get_var(&mut self, var: &str) -> Sign {
        if let Some(target) = self.frame.shared.get(var).cloned() {
            let value = self.get_var_in_stack(self.frame.caller_depth, &target);
            self.frame.env.set(var, value.clone());
            value
        } else {
            self.frame.env.get(var)
        }
    }

    fn get_var_in_stack(&self, frame_idx: Option<usize>, var: &str) -> Sign {
        let Some(idx) = frame_idx else {
            return Sign::blank();
        };
        let frame = &self.stack[idx];
        if let Some(target) = frame.shared.get(var) {
            return self.get_var_in_stack(frame.caller_depth, target);
        }
        frame.env.get(var)
    }

    fn set_var(&mut self, var: &str, value: Sign) {
        if let Some(target) = self.frame.shared.get(var).cloned() {
            self.set_var_in_stack(self.frame.caller_depth, &target, value.clone());
        }
        self.frame.env.set(var, value);
    }

    fn set_var_in_stack(&mut self, frame_idx: Option<usize>, var: &str, value: Sign) {
        let Some(idx) = frame_idx else {
            return;
        };
        let shared_target = {
            let frame = &self.stack[idx];
            frame.shared.get(var).cloned()
        };
        if let Some(target) = shared_target {
            let caller_depth = self.stack[idx].caller_depth;
            self.set_var_in_stack(caller_depth, &target, value.clone());
        }
        if let Some(frame) = self.stack.get_mut(idx) {
            frame.env.set(var, value);
        }
    }

    fn current_pc(&self) -> usize {
        self.frame.blocks.last().map(|block| block.pc).unwrap_or(0)
    }

    fn peek_stmt(&self) -> Option<Stmt> {
        let block = self.frame.blocks.last()?;
        if block.pc < block.stmts.len() {
            return Some(block.stmts[block.pc].clone());
        }
        if block.loop_label.is_some() && !block.stmts.is_empty() {
            return Some(block.stmts[0].clone());
        }
        None
    }

    fn next_stmt(&mut self) -> Result<Option<Stmt>, String> {
        loop {
            let Some(block) = self.frame.blocks.last_mut() else {
                return Ok(None);
            };
            if block.pc < block.stmts.len() {
                let stmt = block.stmts[block.pc].clone();
                block.pc += 1;
                return Ok(Some(stmt));
            }
            if block.loop_label.is_some() {
                if block.stmts.is_empty() {
                    return Err("loop body must not be empty".to_string());
                }
                block.pc = 0;
                continue;
            }
            return Ok(None);
        }
    }

    fn break_loop(&mut self, label: &str) -> Result<(), String> {
        while let Some(block) = self.frame.blocks.pop() {
            if block.loop_label.as_deref() == Some(label) {
                return Ok(());
            }
        }
        Err(format!("break label not found: {}", label))
    }
}

fn collect_vars(stmts: &[Stmt]) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    collect_vars_inner(stmts, &mut vars);
    vars
}

fn collect_vars_inner(stmts: &[Stmt], vars: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Read(var) | Stmt::Stor(var) | Stmt::IfBreak { var, .. } => {
                vars.insert(var.clone());
            }
            Stmt::Assign(dst, src) => {
                vars.insert(dst.clone());
                vars.insert(src.clone());
            }
            Stmt::ConstAssign(dst, _) => {
                vars.insert(dst.clone());
            }
            Stmt::Loop { body, .. } => collect_vars_inner(body, vars),
            Stmt::Call { args, .. } => {
                for arg in args {
                    vars.insert(arg.name.clone());
                }
            }
            Stmt::Lt | Stmt::Rt => {}
            Stmt::StorConst(_) => {}
            Stmt::IfBreakHead { .. } => {}
        }
    }
}
