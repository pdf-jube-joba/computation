use recursive_function::machine::NumberTuple;
use turing_machine::{
    machine::{Sign, State, TapeAsVec, TuringMachineSet},
    manipulation::{
        builder::TuringMachineBuilder,
        graph_compose::{naive_builder_composition, GraphOfBuilder},
    },
};

use super::{state, expand_aux_shift_R};
use super::{
    annihilate, bor1orbar, composition, copy, n_times_copy, copy_aux_this_1, copy_aux_this_b, id, move_left,
    move_right, move_rights, num_tape, copy_aux_pre, pre_move_this_1, pre_move_this_b,
    rotate_aux_move_this_tuple, pre_put_rotate, move_empty_case, rotate_aux_remove_first,
    remove_first_aux_remove_one, put1, putb, putbar, rotate, succ_builder, zero_builder,
};

fn sign(str: &str) -> Sign {
    Sign::try_from(str).unwrap()
}
fn vec_sign(vec: Vec<&str>) -> Vec<Sign> {
    vec.into_iter().map(|str| sign(str)).collect()
}
fn builder_test(
    builder: &mut TuringMachineBuilder,
    step: usize,
    tests: Vec<(TapeAsVec, TapeAsVec)>,
) {
    eprintln!("test start");
    for (input, result) in tests {
        let mut machine = builder.input(input).build().unwrap();
        eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }
        assert!(machine.is_accepted());
        assert!(machine.now_tape().eq(&result));
    }
}

#[test]
fn builder_safe() {
    let _ = zero_builder();
    let _ = succ_builder();
    let _ = id();
    let _ = move_right();
    let _ = move_rights(1);
    let _ = move_rights(2);
    let _ = move_left();
    let _ = bor1orbar();
    let _ = put1();
    let _ = putb();
    let _ = putbar();
    let _ = copy_aux_pre();
    let _ = copy_aux_this_1();
    let _ = copy_aux_this_1();
    let _ = copy();
    let _ = n_times_copy(0);
    let _ = n_times_copy(1);
    let _ = annihilate();
    let _ = pre_put_rotate(2);
    let _ = pre_move_this_1(2);
    let _ = pre_move_this_b(2);
    let _ = rotate_aux_move_this_tuple(2);
    let _ = move_empty_case();
    let _ = remove_first_aux_remove_one();
    let _ = remove_first_aux_remove_one();
    let _ = rotate(3);
    let _ = expand_aux_shift_R();
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
        // eprintln!("start: {} {:?}", machine.now_state(), machine.now_tape());
        loop {
            let _ = machine.step(1);
            // eprintln!("next: {} {:?}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![i + 1].into()))
    }
}
#[test]
fn move_const() {
    let vec: Vec<((usize, usize), State)> = vec![((0, 1), State::try_from("end").unwrap())];
    let graph = GraphOfBuilder {
        name: "move return".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![move_right(), move_left()],
        assign_edge_to_state: vec.into_iter().collect(),
        acceptable: vec![vec![], vec![State::try_from("end").unwrap()]],
    };
    let mut builder = naive_builder_composition(graph).unwrap();
    // eprintln!("code:");
    for entry in builder.get_code() {
        // eprintln!("    {:?}", entry);
    }
    // eprintln!("init: {:?}", builder.get_init_state());
    // eprintln!("accp: {:?}", builder.get_accepted_state());
    builder.input(num_tape::write(vec![1, 0].into()));

    let mut machine = builder.build().unwrap();
    // eprintln!("start: {} {:?}", machine.now_state(), machine.now_tape());
    for _ in 0..50 {
        let _ = machine.step(1);
        // eprintln!("next : {} {:?}", machine.now_state(), machine.now_tape());
        if machine.is_terminate() {
            break;
        }
    }
    let result = num_tape::read_right_one(machine.now_tape());
    assert_eq!(result, Ok(vec![1, 0].into()))
}
#[test]
fn pre_copy_test() {
    let mut builder = copy_aux_pre();

    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-", "-"]),
            },
        ),
    ];

    builder_test(&mut builder, 100, tests);
}
#[test]
fn copy_test() {
    let mut builder = copy();
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-", "", "1", "-"]),
            },
        ),
    ];
    builder_test(&mut builder, 100, tests);
}
#[test]
fn pre_move_test() {
    let mut builder = rotate_aux_move_this_tuple(2);

    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "", "-", "-", "", "1", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec![
                    "", "", "", "", "-", "", "1", "-", "", "1", "", "1", "-",
                ]),
            },
        ),
    ];
    builder_test(&mut builder, 400, tests);
}
#[test]
fn annihilate_test() {
    let mut builder = annihilate();
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
        ),
    ];
    builder_test(&mut builder, 500, tests);
}
#[test]
fn move_empty_case_test() {
    let mut builder = move_empty_case();
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "", "1", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "-"]),
            },
        ),
    ];
    builder_test(&mut builder, 100, tests);
}
#[test]
fn remove_first_aux_remove_one_pre_test() {
    let mut builder = remove_first_aux_remove_one();
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "", "1", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "", "", "-", "", "1", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "", "", "-"]),
            },
        ),
    ];
    builder_test(&mut builder, 500, tests)
}
#[test]
fn remove_first_aux_remove_one_test() {
    let mut builder = remove_first_aux_remove_one();
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "", "1", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "", "-", "", "1", "", "1", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "", "-"]),
            },
        ),
    ];
    builder_test(&mut builder, 500, tests)
}
#[test]
fn pre_remove_first_test() {
    let mut builder = rotate_aux_remove_first(2);
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "", "1", "", "1", "-", "", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "", "-", "", "1", "", "1", "-", "", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "-", "", "", "-"]),
            },
        ),
    ];
    builder_test(&mut builder, 500, tests)
}
#[test]
fn rotate_test() {
    let mut builder = rotate(3);
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "", "1", "", "1", "-", "", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "-", "-"]),
            },
        ),
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "", "-", "", "1", "", "1", "-", "", "-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "", "1", "-", "", "-", "", "", "-"]),
            },
        ),
    ];
    builder_test(&mut builder, 2000, tests);
}
#[test]
fn composition_test() {
    let mut builder = composition(vec![zero_builder()], succ_builder());
    let tests = vec![(
        TapeAsVec {
            left: vec![],
            head: sign("-"),
            right: vec_sign(vec!["-"]),
        },
        TapeAsVec {
            left: vec![],
            head: sign("-"),
            right: vec_sign(vec!["", "1", "-"]),
        },
    )];
    builder_test(&mut builder, 2000, tests);
}

#[test]
fn expand_aux_shift_R_test() {
    let mut builder = expand_aux_shift_R();
    let tests = vec![
        (
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"]),
            },
        )
    ];
    builder_test(&mut builder, 100, tests);
}