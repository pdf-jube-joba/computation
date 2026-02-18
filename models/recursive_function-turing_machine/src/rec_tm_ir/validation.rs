use std::collections::{HashMap, HashSet};
use turing_machine::machine::{Sign, Tape};
use utils::TextCodec;

use super::machine::{Program, Stmt};

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

pub fn validate_loops(program: &Program) -> Result<(), String> {
    for func in &program.functions {
        let mut loop_stack = Vec::new();
        validate_loops_in_stmts(&func.body, &mut loop_stack)?;
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
        validate_signs_in_stmts(&func.body, allowed)?;
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
    collect_calls(&func.body, &mut calls);
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

fn validate_loops_in_stmts(stmts: &[Stmt], loop_stack: &mut Vec<String>) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            Stmt::IfBreak { label, .. } => {
                if !loop_stack.iter().any(|name| name == label) {
                    return Err(format!("break target '{}' not found", label));
                }
            }
            Stmt::IfBreakHead { label, .. } => {
                if !loop_stack.iter().any(|name| name == label) {
                    return Err(format!("break target '{}' not found", label));
                }
            }
            Stmt::Loop { label, body } => {
                if loop_stack.iter().any(|name| name == label) {
                    return Err(format!("Loop label '{}' is duplicated", label));
                }
                loop_stack.push(label.clone());
                validate_loops_in_stmts(body, loop_stack)?;
                loop_stack.pop();
            }
            _ => {}
        }
    }
    Ok(())
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

fn validate_signs_in_stmts(stmts: &[Stmt], allowed: &HashSet<Sign>) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            Stmt::IfBreak { value, .. } => {
                if !allowed.contains(value) {
                    return Err(format!("Unknown sign in if: {}", value.print()));
                }
            }
            Stmt::IfBreakHead { value, .. } => {
                if !allowed.contains(value) {
                    return Err(format!("Unknown sign in if head: {}", value.print()));
                }
            }
            Stmt::ConstAssign(_, value) | Stmt::StorConst(value) => {
                if !allowed.contains(value) {
                    return Err(format!("Unknown sign in const: {}", value.print()));
                }
            }
            Stmt::Loop { body, .. } => validate_signs_in_stmts(body, allowed)?,
            _ => {}
        }
    }
    Ok(())
}
