use serde_json::Value;
use std::collections::HashMap;
use turing_machine::machine::{Sign, Tape};
use utils::{Machine, StepResult, TextCodec, parse::ParseTextCodec};

use super::{
    auxiliary::{basic, copy, rotate},
    compile, read_right_one_usize, write_usize,
};
use crate::{
    rec_tm_ir::{Program, RecTmIrMachine},
    rec_to_ir::S,
};

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
    mut machine: RecTmIrMachine,
    limit: usize,
    print: bool,
    vars: &[&str],
) -> Result<crate::rec_tm_ir::Snapshot, String> {
    for _ in 0..limit {
        match machine.step(())? {
            StepResult::Continue { next, .. } => {
                if print {
                    let snapshot = next.current();
                    let tape = snapshot_tape(snapshot.clone());
                    let env = snapshot_env(snapshot);
                    let mut rendered = Vec::new();
                    for var in vars {
                        let value = env.get(*var).map(String::as_str).unwrap_or("?");
                        rendered.push(format!("{var}={value}"));
                    }
                    eprintln!("{:<30} {:>10}", tape.print(), rendered.join(","));
                }
                machine = next;
            }
            StepResult::Halt { snapshot, .. } => return Ok(snapshot),
        }
    }
    Err("step limit exceeded".to_string())
}

fn run_until_halt(
    machine: RecTmIrMachine,
    limit: usize,
    print: bool,
) -> Result<crate::rec_tm_ir::Snapshot, String> {
    run_until_halt_with_vars(machine, limit, print, &[])
}

fn wrap_function(function: crate::rec_tm_ir::Function) -> Program {
    super::wrap_function(function)
}

/*
=== Number intepretation ===
*/

#[test]
fn number_ip() {
    let tape = tape_from(&["x", "-", "l", "l", "x"], 0); // => number 2
    let res = read_right_one_usize(&tape).unwrap();
    assert_eq!(res, vec![2]);
    let tape = tape_from(&["x", "x", "-", "l", "l", "x"], 1); // => number 2
    let res = read_right_one_usize(&tape).unwrap();
    assert_eq!(res, vec![2]);
    let tape = tape_from(&["x", "-", "x", "-", "l", "l", "x", "-", "-"], 2); // => number 2
    let res = read_right_one_usize(&tape).unwrap();
    assert_eq!(res, vec![2]);
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

/*
=== basic function ===
*/

#[test]
fn move_right_till_x_works() {
    let program = wrap_function(basic::move_right_till_x_n_times(1));
    let machine = RecTmIrMachine::make(program, tape_from(&["-", "l", "x", "-"], 0)).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["-", "l", "x", "-"], 2);
    assert!(tape.eq(&expected));
}

#[test]
fn move_left_till_x_works() {
    let program = wrap_function(basic::move_left_till_x_n_times(1));
    let machine = RecTmIrMachine::make(program, tape_from(&["x", "-", "l", "-"], 3)).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "l", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn move_till_x_n_times_works() {
    let symbols = vec![
        "-", "l", "x", "-", "x", "x", "-", "-", "x", "l", "x", "l", "-",
    ];

    let program = wrap_function(basic::move_right_till_x_n_times(3));
    let machine = RecTmIrMachine::make(program, tape_from(&symbols, 0)).unwrap();
    let snapshot = run_until_halt(machine, 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&symbols, 5);
    assert!(tape.eq(&expected));

    let program = wrap_function(basic::move_right_till_x_n_times(2));
    let machine = RecTmIrMachine::make(program, tape_from(&symbols, 2)).unwrap();
    let snapshot = run_until_halt(machine, 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&symbols, 5);
    assert!(tape.eq(&expected));

    let program = wrap_function(basic::move_left_till_x_n_times(2));
    let machine = RecTmIrMachine::make(program, tape_from(&symbols, 11)).unwrap();
    let snapshot = run_until_halt(machine, 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&symbols, 8);
    assert!(tape.eq(&expected));

    let program = wrap_function(basic::move_left_till_x_n_times(3));
    let machine = RecTmIrMachine::make(program, tape_from(&symbols, 10)).unwrap();
    let snapshot = run_until_halt(machine, 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&symbols, 4);
    assert!(tape.eq(&expected));
}

#[test]
fn shift_left() {
    // case for S::B
    let program = wrap_function(basic::shift_left_x(S::B));

    let tape: Tape = "l, x, -,-,- |x|".parse_tc().unwrap();
    let expd: Tape = "l |-| -,-,x, -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let tape: Tape = "l, x |x|".parse_tc().unwrap();
    let expd: Tape = "l |x| -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let tape: Tape = "l, x, x |x|".parse_tc().unwrap();
    let expd: Tape = "l, x |x| -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let tape: Tape = "l,x,l, x, -,-,- |x|".parse_tc().unwrap();
    let expd: Tape = "l,x,l |-| -,-,x, -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    // case for S::X
    let program = wrap_function(basic::shift_left_x(S::X));

    let tape: Tape = "l, x, -,-,- |x|".parse_tc().unwrap();
    let expd: Tape = "l |-| -,-,x, x".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let tape: Tape = "l, x |x|".parse_tc().unwrap();
    let expd: Tape = "l |x| x".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let tape: Tape = "l,x,l, x, -,-,- |x|".parse_tc().unwrap();
    let expd: Tape = "l,x,l |-| -,-,x, x".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64 * 2, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));
}

#[test]
fn shift_left_n_times() {
    let program = wrap_function(basic::shift_left_x_n_times(1));

    let tape: Tape = "l |x| -, -, -, x".parse_tc().unwrap();
    let expd: Tape = "l |-| -, -, x, -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt_with_vars(machine, 64 * 2, true, &["where"]).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let program = wrap_function(basic::shift_left_x_n_times(2));

    let tape: Tape = "l |x| -, l, -, x, l, x".parse_tc().unwrap();
    let expd: Tape = "l |-| l, -, x, l, x, -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt_with_vars(machine, 64 * 2, true, &["where"]).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));

    let tape: Tape = "l |x| x, x".parse_tc().unwrap();
    let expd: Tape = "l |x| x, -".parse_tc().unwrap();
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt_with_vars(machine, 64 * 2, true, &["where"]).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&expd));
}

/*
=== rotate function ===
*/

#[test]
fn swap_shorter_b() {
    let program = wrap_function(rotate::swap_tuple());
    let input = tape_from(&["x", "-", "l", "l", "x", "-", "l", "x"], 0);
    let machine = RecTmIrMachine::make(program, input).unwrap();
    let snapshot = run_until_halt_with_vars(machine, 64 * 64, true, &["put", "where", "where2"]).unwrap();
    let tape = snapshot_tape(snapshot);
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "-", "l", "x", "-", "l", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn swap_longer_b() {
    let program = wrap_function(rotate::swap_tuple());
    let input = tape_from(&["x", "-", "l", "x", "-", "l", "l", "x"], 0);
    let machine = RecTmIrMachine::make(program, input).unwrap();
    let snapshot = run_until_halt(machine, 64 * 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn swap_nullable() {
    let program = wrap_function(rotate::swap_tuple());

    let input = tape_from(&["x", "x", "x"], 0);
    let machine = RecTmIrMachine::make(program.clone(), input).unwrap();
    let snapshot = run_until_halt(machine, 64 * 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "x", "x"], 0);
    assert!(tape.eq(&expected));

    let input = tape_from(&["x", "x", "-", "x"], 0);
    let machine = RecTmIrMachine::make(program.clone(), input).unwrap();
    let snapshot = run_until_halt(machine, 64 * 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "-", "x", "x"], 0);
    assert!(tape.eq(&expected));

    let input = tape_from(&["x", "-", "x", "x"], 0);
    let machine = RecTmIrMachine::make(program.clone(), input).unwrap();
    let snapshot = run_until_halt(machine, 64 * 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "x", "-", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn rotate() {
    let program = wrap_function(rotate::rotate(3));

    let input = tape_from(&["x", "-", "x", "x", "l", "x"], 0);
    let machine = RecTmIrMachine::make(program.clone(), input).unwrap();
    let snapshot = run_until_halt(machine, 64 * 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    eprintln!("{}", tape.print());
    let expected = tape_from(&["x", "x", "l", "x", "-", "x"], 0);
    assert!(tape.eq(&expected));
}

/*
=== copy function ===
*/

#[test]
fn copy_to_end_works() {
    let symbols = vec!["x", "-", "l", "l", "x", "l", "-", "l", "x"];
    let program = wrap_function(copy::copy_to_end(1));
    let machine = RecTmIrMachine::make(program, tape_from(&symbols, 0)).unwrap();
    let snapshot = run_until_halt(machine, 64 * 8, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(
        &[
            "x", "-", "l", "l", "x", "l", "-", "l", "x", "-", "l", "l", "x",
        ],
        0,
    );
    assert!(tape.eq(&expected));
}

#[test]
fn copy_only_l() {
    let program = wrap_function(copy::copy_to_end(0));

    let tape: Tape = "|x| x".parse_tc().unwrap();
    let expd: Tape = "- |x| x, x, -".parse_tc().unwrap();

    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);

    assert_eq!(rslt, expd);

    let tape: Tape = "|x| l, l, x".parse_tc().unwrap();
    let expd: Tape = "- |x| l, l, x, l, l, x, -".parse_tc().unwrap();

    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64 * 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);

    assert_eq!(rslt, expd);
}

#[test]
fn copy_empty_tuple() {
    let program = wrap_function(copy::copy_to_end(0));
    let machine = RecTmIrMachine::make(program, tape_from(&["x", "x"], 0)).unwrap();
    let snapshot = run_until_halt(machine, 128, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "x", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_single_tuple_roundtrip() {
    let program = wrap_function(copy::copy_to_end(0));
    let machine = RecTmIrMachine::make(program, write_usize(vec![2])).unwrap();
    let snapshot = run_until_halt(machine, 256, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "l", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_n_times_zero_is_noop() {
    let program = wrap_function(copy::copy_n_times(0));
    let input = tape_from(&["x", "-", "l", "l", "x"], 0);
    let machine = RecTmIrMachine::make(program, input.clone()).unwrap();
    let snapshot = run_until_halt(machine, 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    assert!(tape.eq(&input));
}

#[test]
fn copy_n_times_one_matches_copy() {
    let program = wrap_function(copy::copy_n_times(1));
    let machine = RecTmIrMachine::make(program, write_usize(vec![2])).unwrap();
    let snapshot = run_until_halt(machine, 256, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "l", "l", "x"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn copy_n_times_two_appends_twice() {
    let program = wrap_function(copy::copy_n_times(2));
    let machine = RecTmIrMachine::make(program, write_usize(vec![2])).unwrap();
    let snapshot = run_until_halt(machine, 512, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(
        &[
            "x", "-", "l", "l", "x", "-", "l", "l", "x", "-", "l", "l", "x",
        ],
        0,
    );
    assert!(tape.eq(&expected));
}

/*
=== zero function and succ function ===
*/

#[test]
fn zero_succ_number_roundtrip() {
    let zero_program = wrap_function(compile::zero_function());
    let zero_machine = RecTmIrMachine::make(zero_program, write_usize(vec![])).unwrap();
    let snapshot = run_until_halt(zero_machine, 64, false).unwrap();
    let zero_tape = snapshot_tape(snapshot);
    assert_eq!(read_right_one_usize(&zero_tape), Some(vec![0]));

    let succ_program = wrap_function(compile::succ_function());
    let succ_machine = RecTmIrMachine::make(succ_program, write_usize(vec![2])).unwrap();
    let snapshot = run_until_halt(succ_machine, 64, false).unwrap();
    let succ_tape = snapshot_tape(snapshot);
    assert_eq!(read_right_one_usize(&succ_tape), Some(vec![3]));
}

#[test]
fn zero_function_works() {
    let program = wrap_function(compile::zero_function());
    let machine = RecTmIrMachine::make(program, tape_from(&["x", "x", "-"], 0)).unwrap();
    let snapshot = run_until_halt(machine, 64, false).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "x", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn succ_function_works() {
    let program = wrap_function(compile::succ_function());
    let machine =
        RecTmIrMachine::make(program, tape_from(&["x", "-", "l", "l", "x", "-"], 0)).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "l", "l", "l", "x", "-"], 0);
    assert!(tape.eq(&expected));
}

/*
=== projection function ===
*/

#[test]
fn projection_aux_test() {
    let tape_first = tape_from(&["x", "-", "l", "l", "-", "l", "l", "-", "x"], 0);

    let program = wrap_function(compile::projection::aux_projection_init(3, 0));
    let machine = RecTmIrMachine::make(program, tape_first.clone()).unwrap();
    let snapshot = run_until_halt(machine, 1024, true).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "l", "l", "x", "-", "-", "-", "-"], 4);
    assert!(tape.eq(&expected));

    let program = wrap_function(compile::projection::aux_projection_init(3, 1));
    let machine = RecTmIrMachine::make(program, tape_first.clone()).unwrap();
    let snapshot = run_until_halt(machine, 1024, true).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "-", "-", "-", "l", "l", "x", "-"], 7);
    assert!(tape.eq(&expected));

    let program = wrap_function(compile::projection::aux_projection_init(3, 2));
    let machine = RecTmIrMachine::make(program, tape_first).unwrap();
    let snapshot = run_until_halt(machine, 1024, true).unwrap();
    let tape = snapshot_tape(snapshot);
    let expected = tape_from(&["x", "-", "-", "-", "-", "-", "-", "-", "x"], 8);

    assert!(tape.eq(&expected));
}

#[test]
fn projection_first_element() {
    let program = wrap_function(compile::projection(3, 0));
    let machine = RecTmIrMachine::make(program, write_usize(vec![2, 0, 3])).unwrap();
    let snapshot = run_until_halt(machine, 1024, true).unwrap();
    let tape = snapshot_tape(snapshot);
    assert_eq!(read_right_one_usize(&tape), Some(vec![2]));
}

#[test]
fn projection_middle_zero() {
    let program = wrap_function(compile::projection(3, 1));
    let machine = RecTmIrMachine::make(program, write_usize(vec![4, 0, 1])).unwrap();
    let snapshot = run_until_halt(machine, 1024, false).unwrap();
    let tape = snapshot_tape(snapshot);
    assert_eq!(read_right_one_usize(&tape), Some(vec![0]));
}

#[test]
fn projection_single_tuple_zero() {
    let program = wrap_function(compile::projection(1, 0));
    let machine = RecTmIrMachine::make(program, write_usize(vec![0])).unwrap();
    let snapshot = run_until_halt(machine, 1024, false).unwrap();
    let tape = snapshot_tape(snapshot);
    assert_eq!(read_right_one_usize(&tape), Some(vec![0]));
}

/*
=== primitive recursive function ===
*/

#[test]
fn test_pred_tuple() {
    let program = wrap_function(compile::primitive_recursion::pred_tuple());

    let tape = write_usize(vec![1, 1]);
    let expd = write_usize(vec![0, 1]);
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);
    assert_eq!(rslt, expd);

    let tape = write_usize(vec![1, 0]);
    let expd = write_usize(vec![0, 0]);
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);
    assert_eq!(rslt, expd);

    let tape = write_usize(vec![1]);
    let expd = write_usize(vec![0]);
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);
    assert_eq!(rslt, expd);

    let tape = write_usize(vec![0, 2]);
    let expd = write_usize(vec![2]);
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);
    assert_eq!(rslt, expd);

    let tape = write_usize(vec![0]);
    let expd = write_usize(vec![]);
    let machine = RecTmIrMachine::make(program.clone(), tape).unwrap();
    let snapshot = run_until_halt(machine, 64, true).unwrap();
    let rslt = snapshot_tape(snapshot);
    assert_eq!(rslt, expd);
}
