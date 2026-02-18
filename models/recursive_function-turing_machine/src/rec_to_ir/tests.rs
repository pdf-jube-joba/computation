use serde_json::Value;
use std::collections::HashMap;
use turing_machine::machine::{Sign, Tape};
use utils::{Machine, TextCodec};

use super::{
    auxiliary::{basic, copy, rotate},
    compile, read_right_one_usize, write_usize,
};
use crate::rec_tm_ir::{Program, RecTmIrMachine};

fn tape_from(symbols: &[&str], head: usize) -> Tape {
    let signs = symbols
        .iter()
        .map(|s| <Sign as TextCodec>::parse(s).unwrap())
        .collect::<Vec<_>>();
    Tape::from_vec(signs, head).unwrap()
}

fn snapshot_tape(snapshot: crate::rec_tm_ir::Snapshot) -> Tape {
    let value: Value = snapshot.into();
    let arr = value.as_array().unwrap();
    let tape = arr.last().unwrap().as_object().unwrap();
    let children = tape.get("children").unwrap().as_array().unwrap();
    let mut head = None;
    let mut signs = Vec::new();
    for (idx, child) in children.iter().enumerate() {
        let obj = child.as_object().unwrap();
        let text = obj.get("text").unwrap().as_str().unwrap();
        signs.push(<Sign as TextCodec>::parse(text).unwrap());
        if obj.get("className").and_then(|v| v.as_str()) == Some("highlight") {
            head = Some(idx);
        }
    }
    Tape::from_vec(signs, head.unwrap_or(0)).unwrap()
}

fn snapshot_env(snapshot: crate::rec_tm_ir::Snapshot) -> HashMap<String, String> {
    let value: Value = snapshot.into();
    let arr = value.as_array().unwrap();
    let env_table = arr
        .iter()
        .find(|entry| entry.get("title").and_then(|t| t.as_str()) == Some("env"));
    let mut out = HashMap::new();
    let Some(env_table) = env_table else {
        return out;
    };
    let tmp = vec![];
    let rows = env_table
        .get("rows")
        .and_then(|v| v.as_array())
        .unwrap_or(&tmp);
    for row in rows {
        let cells = row.get("cells").and_then(|v| v.as_array());
        let Some(cells) = cells else { continue };
        if cells.len() < 2 {
            continue;
        }
        let var = cells[0].get("text").and_then(|v| v.as_str());
        let value = cells[1].get("text").and_then(|v| v.as_str());
        let (Some(var), Some(value)) = (var, value) else {
            continue;
        };
        out.insert(var.to_string(), value.to_string());
    }
    out
}

fn run_until_halt_with_vars(
    machine: &mut RecTmIrMachine,
    limit: usize,
    print: bool,
    vars: &[&str],
) -> Result<(), String> {
    for _ in 0..limit {
        if machine.step(())?.is_some() {
            return Ok(());
        }
        if print {
            let snapshot = machine.current();
            let tape = snapshot_tape(snapshot.clone());
            let env = snapshot_env(snapshot);
            let mut rendered = Vec::new();
            for var in vars {
                let value = env.get(*var).map(String::as_str).unwrap_or("?");
                rendered.push(format!("{var}={value}"));
            }
            eprintln!("{:<30} {:>10}", tape.print(), rendered.join(","));
        }
    }
    Err("step limit exceeded".to_string())
}

fn run_until_halt(machine: &mut RecTmIrMachine, limit: usize, print: bool) -> Result<(), String> {
    run_until_halt_with_vars(machine, limit, print, &[])
}

fn wrap_function(function: crate::rec_tm_ir::Function) -> Program {
    super::wrap_function(function)
}

#[test]
fn number_tape_roundtrip() {
    let cases = vec![vec![], vec![0], vec![1], vec![2], vec![1, 1], vec![1, 2, 3]];
    for case in cases {
        let tape = write_usize(case.clone());
        let back = read_right_one_usize(&tape);
        assert_eq!(back, Some(case));
    }
}

#[test]
fn zero_function_works() {
    let program = wrap_function(compile::zero_builder());
    let mut machine = RecTmIrMachine::make(program, tape_from(&["x", "x", "-"], 0)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "x", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn succ_function_works() {
    let program = wrap_function(compile::succ_builder());
    let mut machine =
        RecTmIrMachine::make(program, tape_from(&["x", "-", "l", "l", "x", "-"], 0)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "l", "l", "l", "x", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn move_right_till_x_works() {
    let program = wrap_function(basic::move_right_till_x_n_times(1));
    let mut machine = RecTmIrMachine::make(program, tape_from(&["-", "l", "x", "-"], 0)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["-", "l", "x", "-"], 2);
    assert!(tape.eq(&expected));
}

#[test]
fn move_left_till_x_works() {
    let program = wrap_function(basic::move_left_till_x_n_times(1));
    let mut machine = RecTmIrMachine::make(program, tape_from(&["x", "-", "l", "-"], 3)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "l", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn swap_shorter_b() {
    let program = wrap_function(rotate::swap_tuple());
    let input = tape_from(&["x", "-", "l", "l", "x", "-", "l", "x"], 0);
    let mut machine = RecTmIrMachine::make(program, input).unwrap();
    run_until_halt_with_vars(&mut machine, 64 * 3, true, &["put"]).unwrap();
    let tape = snapshot_tape(machine.current());
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "-", "l", "x", "-", "l", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn swap_longer_b() {
    let program = wrap_function(rotate::swap_tuple());
    let input = tape_from(&["x", "-", "l", "x", "-", "l", "l", "x"], 0);
    let mut machine = RecTmIrMachine::make(program, input).unwrap();
    run_until_halt(&mut machine, 64 * 8, false).unwrap();
    let tape = snapshot_tape(machine.current());
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn move_till_x_n_times_works() {
    let symbols = vec![
        "-", "l", "x", "-", "x", "x", "-", "-", "x", "l", "x", "l", "-",
    ];

    let program = wrap_function(basic::move_right_till_x_n_times(3));
    let mut machine = RecTmIrMachine::make(program, tape_from(&symbols, 0)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&symbols, 5);
    assert!(tape.eq(&expected));

    let program = wrap_function(basic::move_right_till_x_n_times(2));
    let mut machine = RecTmIrMachine::make(program, tape_from(&symbols, 2)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&symbols, 5);
    assert!(tape.eq(&expected));

    let program = wrap_function(basic::move_left_till_x_n_times(2));
    let mut machine = RecTmIrMachine::make(program, tape_from(&symbols, 11)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&symbols, 8);
    assert!(tape.eq(&expected));

    let program = wrap_function(basic::move_left_till_x_n_times(3));
    let mut machine = RecTmIrMachine::make(program, tape_from(&symbols, 10)).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&symbols, 4);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_to_end_works() {
    let symbols = vec!["x", "-", "l", "l", "x", "l", "-", "l", "x"];
    let program = wrap_function(copy::copy_to_end(1));
    let mut machine = RecTmIrMachine::make(program, tape_from(&symbols, 0)).unwrap();
    run_until_halt(&mut machine, 64 * 8, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(
        &[
            "x", "-", "l", "l", "x", "l", "-", "l", "x", "-", "l", "l", "x",
        ],
        0,
    );
    assert!(tape.eq(&expected));
}

#[test]
fn zero_succ_number_roundtrip() {
    let zero_program = wrap_function(compile::zero_builder());
    let mut zero_machine = RecTmIrMachine::make(zero_program, write_usize(vec![])).unwrap();
    run_until_halt(&mut zero_machine, 64, false).unwrap();
    let zero_tape = snapshot_tape(zero_machine.current());
    assert_eq!(read_right_one_usize(&zero_tape), Some(vec![0]));

    let succ_program = wrap_function(compile::succ_builder());
    let mut succ_machine = RecTmIrMachine::make(succ_program, write_usize(vec![2])).unwrap();
    run_until_halt(&mut succ_machine, 64, false).unwrap();
    let succ_tape = snapshot_tape(succ_machine.current());
    assert_eq!(read_right_one_usize(&succ_tape), Some(vec![3]));
}

#[test]
fn copy_empty_tuple() {
    let program = wrap_function(copy::copy_to_end(0));
    let mut machine = RecTmIrMachine::make(program, tape_from(&["x", "x"], 0)).unwrap();
    run_until_halt(&mut machine, 128, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "x", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_single_tuple_roundtrip() {
    let program = wrap_function(copy::copy_to_end(0));
    let mut machine = RecTmIrMachine::make(program, write_usize(vec![2])).unwrap();
    run_until_halt(&mut machine, 256, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "l", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_n_times_zero_is_noop() {
    let program = wrap_function(copy::copy_n_times(0));
    let input = tape_from(&["x", "-", "l", "l", "x"], 0);
    let mut machine = RecTmIrMachine::make(program, input.clone()).unwrap();
    run_until_halt(&mut machine, 64, false).unwrap();
    let tape = snapshot_tape(machine.current());
    assert!(tape.eq(&input));
}

#[test]
fn copy_n_times_one_matches_copy() {
    let program = wrap_function(copy::copy_n_times(1));
    let mut machine = RecTmIrMachine::make(program, write_usize(vec![2])).unwrap();
    run_until_halt(&mut machine, 256, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "l", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_n_times_two_appends_twice() {
    let program = wrap_function(copy::copy_n_times(2));
    let mut machine = RecTmIrMachine::make(program, write_usize(vec![2])).unwrap();
    run_until_halt(&mut machine, 512, false).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(
        &[
            "x", "-", "l", "l", "x", "-", "l", "l", "x", "-", "l", "l", "x",
        ],
        0,
    );
    assert!(tape.eq(&expected));
}
