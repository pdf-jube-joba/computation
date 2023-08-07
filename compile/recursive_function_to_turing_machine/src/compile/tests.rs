use crate::compile::projection::projection;

use super::*;
use recursive_function::machine::NumberTuple;

fn print_process(machine: &TuringMachineSet) {
    let state_str = machine.now_state().to_string();
    if state_str.contains("start") || state_str.contains("end") {
        eprintln!("{}\n   {}", machine.now_state(), machine.now_tape());
    }
}

#[test]
fn tuple_read_write() {
    fn assert_equal(tuple: NumberTuple) {
        let tape = num_tape::write(tuple.clone());
        let result = num_tape::read_right_one(tape);
        assert_eq!(Ok(tuple), result)
    }

    assert_equal(vec![].into());
    assert_equal(vec![0].into());
    assert_equal(vec![1].into());
    assert_equal(vec![2].into());
    assert_equal(vec![1, 1].into());
    assert_equal(vec![1, 2, 3].into());
}
#[test]

fn test_zero() {
    let mut zero_builder = zero_builder();
    zero_builder.input(num_tape::write(vec![].into()));
    let mut machine = zero_builder.build().unwrap();
    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }
    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![0].into()));
}
#[test]
fn succ_zero() {
    let mut succ_builder = succ_builder();

    for i in 0..5 {
        succ_builder.input(num_tape::write(vec![i].into()));
        let mut machine = succ_builder.build().unwrap();
        loop {
            let _ = machine.step(1);
            print_process(&machine);
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![i + 1].into()))
    }
}
#[test]
fn projection_test() {
    let mut builder = projection::projection(2, 0);
    let input: TapeAsVec = num_tape::write(vec![1, 2].into());
    builder.input(input);

    let mut machine = builder.build().unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![1].into()));

    let mut builder = projection::projection(3, 0);
    let input: TapeAsVec = num_tape::write(vec![1, 2, 3].into());
    builder.input(input);

    let mut machine = builder.build().unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![1].into()));

    let mut builder = projection::projection(3, 1);
    let input: TapeAsVec = num_tape::write(vec![1, 2, 3].into());
    builder.input(input);

    let mut machine = builder.build().unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![2].into()));

    let mut builder = projection::projection(3, 2);
    let input: TapeAsVec = num_tape::write(vec![1, 2, 3].into());
    builder.input(input);

    let mut machine = builder.build().unwrap();

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![3].into()));
}
#[test]
fn composition_test() {
    let mut builder = composition::composition(vec![zero_builder()], succ_builder());
    let input: TapeAsVec = num_tape::write(vec![].into());
    builder.input(input);

    let mut machine = builder.build().unwrap();

    loop {
        let _ = machine.step(1);
        if machine.is_terminate() {
            print_process(&machine);
            break;
        }
    }
    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![1].into()));

    let mut builder = composition::composition(
        vec![
            projection::projection(3, 2),
            projection::projection(3, 1),
            projection::projection(3, 0),
        ],
        projection(3, 0),
    );
    let input: TapeAsVec = num_tape::write(vec![1, 2, 3].into());
    builder.input(input);

    let mut machine = builder.build().unwrap();
    print_process(&machine);

    loop {
        let _ = machine.step(1);
        print_process(&machine);
        if machine.is_terminate() {
            break;
        }
    }
    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![3].into()));
}
#[test]
fn primitive_recursion_test() {
    let mut builder = primitive_recursion::primitive_recursion(
        zero_builder(),
        composition::composition(vec![projection::projection(2, 0)], succ_builder()),
    );
    for i in 0..5 {
        let input = num_tape::write(vec![i].into());
        let mut machine = builder.input(input).build().unwrap();

        loop {
            let _ = machine.step(1);
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![i].into()));
    }
}
#[test]
fn mu_recursion_test() {
    let mut builder = mu_recursion::mu_recursion(super::basic::id());
    let input = num_tape::write(vec![].into());
    let mut machine = builder.input(input).build().unwrap();

    loop {
        let _ = machine.step(1);
        if machine.is_terminate() {
            break;
        }
    }

    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![0].into()));
}

fn add() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        RecursiveFunctions::projection(1, 0).unwrap(),
        RecursiveFunctions::composition(
            3,
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
            RecursiveFunctions::succ(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn any_to_zero() -> RecursiveFunctions {
    RecursiveFunctions::composition(1, vec![], RecursiveFunctions::zero()).unwrap()
}

fn mul() -> RecursiveFunctions {
    RecursiveFunctions::primitive_recursion(
        any_to_zero(),
        RecursiveFunctions::composition(
            3,
            vec![
                RecursiveFunctions::projection(3, 0).unwrap(),
                RecursiveFunctions::projection(3, 2).unwrap(),
            ],
            add(),
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
            3,
            vec![RecursiveFunctions::projection(3, 0).unwrap()],
            pred(),
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

fn func_test(fun: &RecursiveFunctions, tests: Vec<(NumberTuple, NumberTuple)>) {
    let mut builder = compile(&fun);
    for (input, expect) in tests {
        let mut machine = builder.input(num_tape::write(input)).build().unwrap();
        let mut loop_num = 0;
        loop {
            let _ = machine.step(1);
            if machine.is_accepted() {
                break;
            }
            loop_num = loop_num + 1;
            if loop_num > 1000 && loop_num % 100 == 0 {
                eprint!("{loop_num}:");
                print_process(&machine);
            }
        }

        let result = num_tape::read_right_one(machine.now_tape()).unwrap();
        assert_eq!(expect, result);
    }
}

#[test]
fn compile_test_add() {
    let tests: Vec<(NumberTuple, NumberTuple)> = vec![
        (vec![0, 0].into(), vec![0].into()),
        (vec![0, 1].into(), vec![1].into()),
        (vec![0, 2].into(), vec![2].into()),
        (vec![1, 0].into(), vec![1].into()),
        (vec![1, 1].into(), vec![2].into()),
        (vec![1, 2].into(), vec![3].into()),
    ];

    func_test(&add(), tests);
}

#[test]
fn compile_test_any_to_zero() {
    let tests: Vec<(NumberTuple, NumberTuple)> = vec![
        (vec![0].into(), vec![0].into()),
        (vec![1].into(), vec![0].into()),
        (vec![2].into(), vec![0].into()),
    ];

    func_test(&any_to_zero(), tests);
}

#[test]
fn compile_test_mul() {
    let tests: Vec<(NumberTuple, NumberTuple)> = vec![
        (vec![0, 0].into(), vec![0].into()),
        (vec![0, 1].into(), vec![0].into()),
        (vec![0, 2].into(), vec![0].into()),
        (vec![1, 0].into(), vec![0].into()),
        (vec![1, 1].into(), vec![1].into()),
        (vec![1, 2].into(), vec![2].into()),
        (vec![2, 0].into(), vec![0].into()),
        (vec![2, 1].into(), vec![2].into()),
        (vec![2, 2].into(), vec![4].into()),
    ];

    func_test(&mul(), tests);
}

#[test]
fn compile_test_zero_from_mul() {
    let tests: Vec<(NumberTuple, NumberTuple)> = vec![
        (vec![0].into(), vec![0].into()),
        (vec![1].into(), vec![0].into()),
        (vec![2].into(), vec![0].into()),
    ];

    func_test(&zero_from_mul(), tests);
}

#[test]
fn compile_test_id_from_inv_monus() {
    let tests: Vec<(NumberTuple, NumberTuple)> = vec![
        (vec![0].into(), vec![0].into()),
        (vec![1].into(), vec![1].into()),
        (vec![2].into(), vec![2].into()),
    ];

    func_test(&id_from_inv_monus(), tests);
}
