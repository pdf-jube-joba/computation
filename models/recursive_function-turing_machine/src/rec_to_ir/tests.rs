use serde_json::Value;
use turing_machine::machine::{Sign, Tape};
use utils::{Machine, TextCodec};

use super::{auxiliary::basic, compile, read_right_one_usize, write_usize};
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

fn run_until_halt(machine: &mut RecTmIrMachine, limit: usize) -> Result<(), String> {
    for _ in 0..limit {
        if machine.step(())?.is_some() {
            return Ok(());
        }
    }
    Err("step limit exceeded".to_string())
}

fn wrap_function(function: crate::rec_tm_ir::Function) -> Program {
    super::wrap_function(function)
}

#[test]
fn number_tape_roundtrip() {
    let cases = vec![
        vec![],
        vec![0],
        vec![1],
        vec![2],
        vec![1, 1],
        vec![1, 2, 3],
    ];
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
    run_until_halt(&mut machine, 64).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "x", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn succ_function_works() {
    let program = wrap_function(compile::succ_builder());
    let mut machine =
        RecTmIrMachine::make(program, tape_from(&["x", "-", "l", "l", "x", "-"], 0)).unwrap();
    run_until_halt(&mut machine, 64).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "l", "l", "l", "x", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn move_right_till_x_works() {
    let program = wrap_function(basic::move_right_till_x());
    let mut machine = RecTmIrMachine::make(program, tape_from(&["-", "l", "x", "-"], 0)).unwrap();
    run_until_halt(&mut machine, 64).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["-", "l", "x", "-"], 2);
    assert!(tape.eq(&expected));
}

#[test]
fn move_left_till_x_works() {
    let program = wrap_function(basic::move_left_till_x());
    let mut machine = RecTmIrMachine::make(program, tape_from(&["x", "-", "l", "-"], 3)).unwrap();
    run_until_halt(&mut machine, 64).unwrap();
    let tape = snapshot_tape(machine.current());
    let expected = tape_from(&["x", "-", "l", "-"], 0);
    assert!(tape.eq(&expected));
}

#[test]
fn zero_succ_number_roundtrip() {
    let zero_program = wrap_function(compile::zero_builder());
    let mut zero_machine = RecTmIrMachine::make(zero_program, write_usize(vec![])).unwrap();
    run_until_halt(&mut zero_machine, 64).unwrap();
    let zero_tape = snapshot_tape(zero_machine.current());
    eprintln!("zero tape: {}", zero_tape.print());
    assert_eq!(read_right_one_usize(&zero_tape), Some(vec![0]));

    let succ_program = wrap_function(compile::succ_builder());
    let mut succ_machine = RecTmIrMachine::make(succ_program, write_usize(vec![2])).unwrap();
    run_until_halt(&mut succ_machine, 64).unwrap();
    let succ_tape = snapshot_tape(succ_machine.current());
    assert_eq!(read_right_one_usize(&succ_tape), Some(vec![3]));
}

/*
use turing_machine::machine::*;
use utils::number::*;
use utils::TextCodec;

use crate::compile::projection::projection;

use crate::auxiliary::basic;
use crate::compile::*;

fn print_process(machine: &TuringMachine) {
    let state_str = machine.now_state().print();
    if state_str.contains("start") || state_str.contains("end") {
        eprintln!(
            "{}\n   {}",
            machine.now_state().print(),
            machine.now_tape().print()
        );
    }
}

fn map_tuple_to_numbertuple(tuple: Vec<usize>) -> Vec<Number> {
    tuple.into_iter().map(|x| x.into()).collect()
}

#[test]
fn tuple_read_write() {
    fn assert_equal(tuple: Vec<usize>) {
        let tape = num_tape::write(map_tuple_to_numbertuple(tuple.clone()));
        let result = num_tape::read_right_one(&tape);
        assert_eq!(Some(map_tuple_to_numbertuple(tuple)), result)
    }

    assert_equal(Vec::<usize>::new());
    assert_equal(vec![0]);
    assert_equal(vec![1]);
    assert_equal(vec![2]);
    assert_equal(vec![1, 1]);
    assert_equal(vec![1, 2, 3]);
}

#[test]

fn test_zero() {
    let zero_builder = zero_builder();
    let mut machine = zero_builder.build(num_tape::write_usize(vec![])).unwrap();
    loop {
        let _ = machine.step(1);
        eprintln!("{}", machine.now_tape().print());
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }
    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![0]));
}
#[test]
fn succ_zero() {
    let succ_builder = succ_builder();

    for i in 0..5 {
        let mut machine = succ_builder.build(num_tape::write_usize(vec![i])).unwrap();
        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one_usize(machine.now_tape());
        assert_eq!(result, Some(vec![i + 1]));
    }
}
#[test]
fn projection_test() {
    let builder = projection::projection(2, 0);
    let input: Tape = num_tape::write_usize(vec![1, 2]);

    let mut machine = builder.build(input).unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![1]));

    let builder = projection::projection(3, 0);
    let input: Tape = num_tape::write_usize(vec![1, 2, 3]);

    let mut machine = builder.build(input).unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![1]));

    let builder = projection::projection(3, 1);
    let input: Tape = num_tape::write_usize(vec![1, 2, 3]);

    let mut machine = builder.build(input).unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![2]));

    let builder = projection::projection(3, 2);
    let input: Tape = num_tape::write_usize(vec![1, 2, 3]);

    let mut machine = builder.build(input).unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![3]));
}
#[test]
fn composition_test() {
    let builder = composition::composition(vec![zero_builder()], succ_builder());
    let input: Tape = num_tape::write_usize(Vec::<usize>::new());

    let mut machine = builder.build(input).unwrap();

    loop {
        let _ = machine.step(1);
        if machine.is_terminate() {
            print_process(&machine);
            break;
        }
    }
    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![1]));

    let builder = composition::composition(
        vec![
            projection::projection(3, 2),
            projection::projection(3, 1),
            projection::projection(3, 0),
        ],
        projection(3, 0),
    );
    let input: Tape = num_tape::write_usize(vec![1, 2, 3]);

    let mut machine = builder.build(input).unwrap();
    print_process(&machine);

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }
    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![3]));
}
#[test]
fn primitive_recursion_test() {
    let builder = primitive_recursion::primitive_recursion(
        zero_builder(),
        composition::composition(vec![projection::projection(2, 0)], succ_builder()),
    );
    for i in 0..5 {
        let input = num_tape::write_usize(vec![i]);
        let mut machine = builder.build(input).unwrap();

        loop {
            let _ = machine.step(1);
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one_usize(machine.now_tape());
        assert_eq!(result, Some(vec![i]));
    }
}
#[test]
fn mu_recursion_test() {
    let builder = mu_recursion::mu_recursion(basic::id());
    let input = num_tape::write_usize(Vec::<usize>::new());
    let mut machine = builder.build(input).unwrap();

    loop {
        let _ = machine.step(1);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one_usize(machine.now_tape());
    assert_eq!(result, Some(vec![0]));
}

fn add() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        RecursiveFunctions::projection(1, 0).unwrap(),
        RecursiveFunctions::composition(
            RecursiveFunctions::succ(),
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
        )
        .unwrap(),
    )
    .unwrap()
}

fn any_to_zero() -> RecursiveFunctions {
    RecursiveFunctions::composition(RecursiveFunctions::zero(), vec![]).unwrap()
}

fn mul() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        any_to_zero(),
        RecursiveFunctions::composition(
            add(),
            vec![
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 2).unwrap(),
            ],
        )
        .unwrap(),
    )
    .unwrap()
}

fn pred() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        RecursiveFunctions::zero(),
        RecursiveFunctions::projection(2, 1).unwrap(),
    )
    .unwrap()
}

fn inv_monus() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        RecursiveFunctions::projection(1, 0).unwrap(),
        RecursiveFunctions::composition(
            pred(),
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
        )
        .unwrap(),
    )
    .unwrap()
}

fn id_from_inv_monus() -> RecursiveFunctions {
    RecursiveFunctions::muoperator(inv_monus()).unwrap()
}

fn zero_from_mul() -> RecursiveFunctions {
    RecursiveFunctions::muoperator(mul()).unwrap()
}

fn func_test(fun: &RecursiveFunctions, tests: Vec<(Vec<usize>, Vec<usize>)>) {
    let builder = compile(fun);
    for (input, expect) in tests {
        let mut machine = builder.build(num_tape::write_usize(input)).unwrap();
        let mut loop_num = 0;
        loop {
            let _ = machine.step(1);
            if machine.is_accepted() {
                break;
            }
            loop_num += 1;
            if loop_num > 1000 && loop_num % 100 == 0 {
                eprint!("{loop_num}:");
                print_process(&machine);
            }
        }

        let result = num_tape::read_right_one_usize(machine.now_tape()).unwrap();
        assert_eq!(expect, result);
    }
}

#[test]
fn compile_test_add() {
    let tests: Vec<(Vec<usize>, Vec<usize>)> = vec![
        (vec![0, 0], vec![0]),
        (vec![0, 1], vec![1]),
        (vec![0, 2], vec![2]),
        (vec![1, 0], vec![1]),
        (vec![1, 1], vec![2]),
        (vec![1, 2], vec![3]),
    ];

    func_test(&add(), tests);
}

#[test]
fn compile_test_any_to_zero() {
    let tests: Vec<(Vec<usize>, Vec<usize>)> =
        vec![(vec![0], vec![0]), (vec![1], vec![0]), (vec![2], vec![0])];

    func_test(&any_to_zero(), tests);
}

#[test]
fn compile_test_mul() {
    let tests: Vec<(Vec<usize>, Vec<usize>)> = vec![
        (vec![0, 0], vec![0]),
        (vec![0, 1], vec![0]),
        (vec![0, 2], vec![0]),
        (vec![1, 0], vec![0]),
        (vec![1, 1], vec![1]),
        (vec![1, 2], vec![2]),
        (vec![2, 0], vec![0]),
        (vec![2, 1], vec![2]),
        (vec![2, 2], vec![4]),
    ];

    func_test(&mul(), tests);
}

#[test]
fn compile_test_zero_from_mul() {
    let tests: Vec<(Vec<usize>, Vec<usize>)> =
        vec![(vec![0], vec![0]), (vec![1], vec![0]), (vec![2], vec![0])];

    func_test(&zero_from_mul(), tests);
}

#[test]
fn compile_test_id_from_inv_monus() {
    let tests: Vec<(Vec<usize>, Vec<usize>)> =
        vec![(vec![0], vec![0]), (vec![1], vec![1]), (vec![2], vec![2])];

    func_test(&id_from_inv_monus(), tests);
}
*/
