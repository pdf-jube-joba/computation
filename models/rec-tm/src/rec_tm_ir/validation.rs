use std::collections::{HashMap, HashSet};
use turing_machine::machine::{Sign, Tape};
use utils::TextCodec;

use super::machine::{Block, Condition, Program, RValue, Stmt};

pub fn validate_no_recursion(program: &Program) -> Result<(), String> {
    if !program.functions.iter().any(|func| func.name == "main") {
        return Err("main() is not defined".to_string());
    }
    let mut state: HashMap<String, u8> = HashMap::new();
    let mut stack = Vec::new();
    for func in &program.functions {
        if state.get(&func.name).copied().unwrap_or(0) == 0 {
            dfs_validate(program, &func.name, &mut state, &mut stack)?;
        }
    }
    Ok(())
}

pub fn validate_alphabet(alphabet: &[Sign]) -> Result<Vec<Sign>, String> {
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

pub fn alphabet_set(alphabet: &[Sign]) -> HashSet<Sign> {
    alphabet.iter().cloned().collect()
}

pub fn validate_tape(tape: &Tape, allowed: &HashSet<Sign>) -> Result<(), String> {
    let (signs, _) = tape.into_vec();
    for sign in signs {
        if !allowed.contains(&sign) {
            return Err(format!("Unknown sign on tape: {}", sign.print()));
        }
    }
    Ok(())
}

pub fn validate_signs_in_program(program: &Program, allowed: &HashSet<Sign>) -> Result<(), String> {
    for func in &program.functions {
        validate_signs_in_blocks(&func.blocks, allowed)?;
        validate_jump_targets(&func.blocks)?;
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
        .iter()
        .find(|func| func.name == name)
        .ok_or_else(|| format!("Undefined function '{}'", name))?;
    let mut calls = HashSet::new();
    collect_calls(&func.blocks, &mut calls);
    for callee in calls {
        if !program.functions.iter().any(|func| func.name == callee) {
            return Err(format!("Undefined function '{}'", callee));
        }
        match state.get(&callee).copied().unwrap_or(0) {
            0 => dfs_validate(program, &callee, state, stack)?,
            1 => {
                stack.push(callee.clone());
                return Err(format!(
                    "Recursive call is not allowed: {}",
                    stack.join(" -> ")
                ));
            }
            _ => {}
        }
    }

    stack.pop();
    state.insert(name.to_string(), 2);
    Ok(())
}

fn collect_calls(blocks: &[Block], calls: &mut HashSet<String>) {
    for block in blocks {
        for stmt in &block.body {
            if let Stmt::Call { func } = stmt {
                calls.insert(func.name.clone());
            }
        }
    }
}

fn validate_signs_in_blocks(blocks: &[Block], allowed: &HashSet<Sign>) -> Result<(), String> {
    for block in blocks {
        validate_signs_in_stmts(&block.body, allowed)?;
    }
    Ok(())
}

fn validate_signs_in_stmts(stmts: &[Stmt], allowed: &HashSet<Sign>) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            Stmt::Assign { dst: _, src } => {
                validate_rvalue(src, allowed)?;
            }
            Stmt::Break { cond }
            | Stmt::Continue { cond }
            | Stmt::Jump { cond, .. }
            | Stmt::Return { cond } => {
                if let Some(cond) = cond {
                    validate_condition(cond, allowed)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_condition(cond: &Condition, allowed: &HashSet<Sign>) -> Result<(), String> {
    validate_rvalue(&cond.left, allowed)?;
    validate_rvalue(&cond.right, allowed)
}

fn validate_rvalue(value: &RValue, allowed: &HashSet<Sign>) -> Result<(), String> {
    if let RValue::Const(sign) = value {
        if !allowed.contains(sign) {
            return Err(format!("Unknown sign in const: {}", sign.print()));
        }
    }
    Ok(())
}

fn validate_jump_targets(blocks: &[Block]) -> Result<(), String> {
    let mut labels = HashSet::new();
    for block in blocks {
        if !labels.insert(block.label.clone()) {
            return Err(format!("label '{}' is duplicated", block.label));
        }
    }
    for block in blocks {
        for stmt in &block.body {
            match stmt {
                Stmt::Jump { label, .. } => {
                    if !labels.contains(label) {
                        return Err(format!("jump target '{}' not found", label));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
