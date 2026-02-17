use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde_json::json;
use turing_machine::machine::{Direction, Sign, Tape};
use utils::{json_text, Machine, TextCodec};

use crate::parser::parse_identifier;

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Lt,
    Rt,
    Read(String),
    Stor(String),
    Assign(String, String),
    IfBreak {
        var: String,
        value: Sign,
        label: String,
    },
    Loop {
        label: String,
        body: Vec<Stmt>,
    },
    Call {
        name: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone)]
enum Instr {
    Lt,
    Rt,
    Read(String),
    Stor(String),
    Assign(String, String),
    IfEqJump {
        var: String,
        value: Sign,
        target: usize,
    },
    Jump(usize),
    Call {
        name: String,
        args: Vec<String>,
    },
}

impl Instr {
    fn render(&self) -> String {
        match self {
            Instr::Lt => "LT".to_string(),
            Instr::Rt => "RT".to_string(),
            Instr::Read(var) => format!("READ {}", var),
            Instr::Stor(var) => format!("STOR {}", var),
            Instr::Assign(dst, src) => format!("{} := {}", dst, src),
            Instr::IfEqJump { var, value, target } => {
                format!("if {} == {} break @{}", var, value.print(), target)
            }
            Instr::Jump(target) => format!("jump @{}", target),
            Instr::Call { name, args } => {
                format!("call {}({})", name, args.join(", "))
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
        let mut stack_children: Vec<serde_json::Value> = snapshot
            .stack
            .iter()
            .map(|name| json_text!(name))
            .collect();
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

        json!([fn_text, pc_text, instruction_text, stack_container, env_table, tape_container])
    }
}

struct CompiledFunction {
    params: Vec<String>,
    vars: BTreeSet<String>,
    code: Vec<Instr>,
}

struct CompiledProgram {
    functions: HashMap<String, CompiledFunction>,
}

struct Frame {
    function: String,
    pc: usize,
    env: Environment,
}

pub struct RecTmIrMachine {
    program: CompiledProgram,
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
        let program = compile_program(&code)?;
        let main = program
            .functions
            .get("main")
            .ok_or_else(|| "main() is not defined".to_string())?;
        let env = Environment::new(main.vars.iter().cloned());
        let allowed = collect_signs_program(&code, &ainput);

        Ok(RecTmIrMachine {
            program,
            frame: Frame {
                function: "main".to_string(),
                pc: 0,
                env,
            },
            stack: Vec::new(),
            tape: ainput,
            allowed,
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        if self.frame.pc >= self.current_function().code.len() {
            return Ok(self.return_from_call());
        }

        let instr = self.current_function().code[self.frame.pc].clone();
        match instr {
            Instr::Lt => {
                self.tape.move_to(&Direction::Left);
                self.frame.pc += 1;
            }
            Instr::Rt => {
                self.tape.move_to(&Direction::Right);
                self.frame.pc += 1;
            }
            Instr::Read(var) => {
                let sign = self.tape.head_read().clone();
                if !self.allowed.contains(&sign) {
                    return Err(format!("Unknown sign on tape: {}", sign.print()));
                }
                self.frame.env.set(&var, sign);
                self.frame.pc += 1;
            }
            Instr::Stor(var) => {
                let sign = self.frame.env.get(&var);
                if !self.allowed.contains(&sign) {
                    return Err(format!("Unknown sign in env: {}", sign.print()));
                }
                self.tape.head_write(&sign);
                self.frame.pc += 1;
            }
            Instr::Assign(dest, src) => {
                let value = self.frame.env.get(&src);
                self.frame.env.set(&dest, value);
                self.frame.pc += 1;
            }
            Instr::IfEqJump { var, value, target } => {
                if self.frame.env.get(&var) == value {
                    self.frame.pc = target;
                } else {
                    self.frame.pc += 1;
                }
            }
            Instr::Jump(target) => {
                self.frame.pc = target;
            }
            Instr::Call { name, args } => {
                let callee = self
                    .program
                    .functions
                    .get(&name)
                    .ok_or_else(|| format!("Undefined function '{}'", name))?;
                if callee.params.len() != args.len() {
                    return Err(format!(
                        "Function '{}' expects {} args, got {}",
                        name,
                        callee.params.len(),
                        args.len()
                    ));
                }
                let mut next_env = Environment::new(callee.vars.iter().cloned());
                for (param, arg) in callee.params.iter().zip(args.iter()) {
                    let value = self.frame.env.get(arg);
                    next_env.set(param, value);
                }
                let caller = Frame {
                    function: self.frame.function.clone(),
                    pc: self.frame.pc + 1,
                    env: self.frame.env.clone(),
                };
                self.stack.push(caller);
                self.frame = Frame {
                    function: name,
                    pc: 0,
                    env: next_env,
                };
            }
        }
        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        let instruction = self
            .current_function()
            .code
            .get(self.frame.pc)
            .map(|instr| instr.render());
        let stack = self
            .stack
            .iter()
            .map(|frame| frame.function.clone())
            .collect();
        Snapshot {
            function: self.frame.function.clone(),
            pc: self.frame.pc,
            instruction,
            env: self.frame.env.clone(),
            tape: self.tape.clone(),
            stack,
        }
    }
}

impl RecTmIrMachine {
    fn current_function(&self) -> &CompiledFunction {
        self.program
            .functions
            .get(&self.frame.function)
            .expect("Current function missing")
    }

    fn return_from_call(&mut self) -> Option<Environment> {
        if let Some(frame) = self.stack.pop() {
            self.frame = frame;
            None
        } else {
            Some(self.frame.env.clone())
        }
    }
}

pub fn validate_no_recursion(program: &Program) -> Result<(), String> {
    if !program.functions.contains_key("main") {
        return Err("main() is not defined".to_string());
    }
    let mut state: HashMap<String, u8> = HashMap::new();
    let mut stack = Vec::new();
    for name in program.functions.keys() {
        if state.get(name).copied().unwrap_or(0) == 0 {
            dfs_validate(program, name, &mut state, &mut stack)?;
        }
    }
    Ok(())
}

fn dfs_validate(
    program: &Program,
    name: &str,
    state: &mut HashMap<String, u8>,
    stack: &mut Vec<String>,
) -> Result<(), String> {
    state.insert(name.to_string(), 1);
    stack.push(name.to_string());

    let func = program
        .functions
        .get(name)
        .ok_or_else(|| format!("Undefined function '{}'", name))?;
    let mut calls = HashSet::new();
    collect_calls(&func.body, &mut calls);
    for callee in calls {
        if !program.functions.contains_key(&callee) {
            return Err(format!("Undefined function '{}'", callee));
        }
        match state.get(&callee).copied().unwrap_or(0) {
            0 => dfs_validate(program, &callee, state, stack)?,
            1 => {
                stack.push(callee.clone());
                return Err(format!("Recursive call is not allowed: {}", stack.join(" -> ")));
            }
            _ => {}
        }
    }

    stack.pop();
    state.insert(name.to_string(), 2);
    Ok(())
}

fn compile_program(program: &Program) -> Result<CompiledProgram, String> {
    let mut functions = HashMap::new();
    for (name, func) in &program.functions {
        let code = compile_stmts(&func.body)?;
        let vars = collect_vars(&func.body);
        functions.insert(
            name.clone(),
            CompiledFunction {
                params: func.params.clone(),
                vars,
                code,
            },
        );
    }
    Ok(CompiledProgram { functions })
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
            Stmt::Loop { body, .. } => collect_vars_inner(body, vars),
            Stmt::Call { args, .. } => {
                for arg in args {
                    vars.insert(arg.clone());
                }
            }
            Stmt::Lt | Stmt::Rt => {}
        }
    }
}

fn collect_calls(stmts: &[Stmt], calls: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Call { name, .. } => {
                calls.insert(name.clone());
            }
            Stmt::Loop { body, .. } => collect_calls(body, calls),
            _ => {}
        }
    }
}

fn collect_signs_program(program: &Program, tape: &Tape) -> HashSet<Sign> {
    let mut signs = HashSet::new();
    signs.insert(Sign::blank());
    for func in program.functions.values() {
        collect_signs(&func.body, &mut signs);
    }
    let (tape_signs, _) = tape.into_vec();
    signs.extend(tape_signs);
    signs
}

fn collect_signs(stmts: &[Stmt], signs: &mut HashSet<Sign>) {
    for stmt in stmts {
        match stmt {
            Stmt::IfBreak { value, .. } => {
                signs.insert(value.clone());
            }
            Stmt::Loop { body, .. } => collect_signs(body, signs),
            _ => {}
        }
    }
}

fn compile_stmts(stmts: &[Stmt]) -> Result<Vec<Instr>, String> {
    let mut instrs = Vec::new();
    let mut loop_stack = Vec::new();
    compile_block(stmts, &mut instrs, &mut loop_stack)?;
    if !loop_stack.is_empty() {
        return Err("Loop stack not empty after compilation".to_string());
    }
    Ok(instrs)
}

struct LoopContext {
    label: String,
    start: usize,
    break_fixups: Vec<usize>,
}

fn compile_block(
    stmts: &[Stmt],
    instrs: &mut Vec<Instr>,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            Stmt::Lt => instrs.push(Instr::Lt),
            Stmt::Rt => instrs.push(Instr::Rt),
            Stmt::Read(var) => instrs.push(Instr::Read(var.clone())),
            Stmt::Stor(var) => instrs.push(Instr::Stor(var.clone())),
            Stmt::Assign(dst, src) => instrs.push(Instr::Assign(dst.clone(), src.clone())),
            Stmt::IfBreak { var, value, label } => {
                let mut found = None;
                for (idx, ctx) in loop_stack.iter().enumerate().rev() {
                    if ctx.label == *label {
                        found = Some(idx);
                        break;
                    }
                }
                let Some(loop_index) = found else {
                    return Err(format!("break target '{}' not found", label));
                };
                let index = instrs.len();
                instrs.push(Instr::IfEqJump {
                    var: var.clone(),
                    value: value.clone(),
                    target: 0,
                });
                loop_stack[loop_index].break_fixups.push(index);
            }
            Stmt::Loop { label, body } => {
                if loop_stack.iter().any(|ctx| ctx.label == *label) {
                    return Err(format!("Loop label '{}' is duplicated", label));
                }
                let start = instrs.len();
                loop_stack.push(LoopContext {
                    label: label.clone(),
                    start,
                    break_fixups: Vec::new(),
                });
                compile_block(body, instrs, loop_stack)?;
                instrs.push(Instr::Jump(start));
                let end = instrs.len();
                if let Some(ctx) = loop_stack.pop() {
                    for fixup in ctx.break_fixups {
                        if let Instr::IfEqJump { target, .. } = &mut instrs[fixup] {
                            *target = end;
                        }
                    }
                }
            }
            Stmt::Call { name, args } => instrs.push(Instr::Call {
                name: name.clone(),
                args: args.clone(),
            }),
        }
    }
    Ok(())
}
