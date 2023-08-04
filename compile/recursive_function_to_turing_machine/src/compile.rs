use std::collections::{HashMap, HashSet};

use turing_machine::{
    machine::*,
    manipulation::code,
    manipulation::{
        builder::{self, TuringMachineBuilder},
        graph_compose::{naive_builder_composition, GraphOfBuilder},
    },
};

pub mod num_tape {
    use recursive_function::machine::{Number, NumberTuple};
    use turing_machine::machine::{Sign, TapeAsVec};

    fn partition() -> Sign {
        Sign::try_from("-").unwrap()
    }

    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }

    fn num_sings(num: Number) -> Vec<Sign> {
        (0..num.into()).map(|_| one()).collect()
    }

    pub fn write(tuple: NumberTuple) -> TapeAsVec {
        let vec: Vec<Number> = tuple.into();
        let mut signs: Vec<Sign> = vec
            .into_iter()
            .flat_map(|num: Number| {
                let mut vec = vec![Sign::blank()];
                vec.extend_from_slice(&num_sings(num));
                vec
            })
            .collect();
        signs.extend_from_slice(&vec![partition()]);
        TapeAsVec {
            left: vec![],
            head: partition(),
            right: signs,
        }
    }

    fn read_one(signs: Vec<Sign>) -> Result<NumberTuple, ()> {
        let v = signs
            .split(|char| *char == Sign::blank())
            .map(|vec| vec.len())
            .skip(1);
        Ok(v.collect::<Vec<_>>().into())
    }

    pub fn read_right_one(tape: TapeAsVec) -> Result<NumberTuple, ()> {
        if tape.head != partition() {
            return Err(());
        }
        let iter = tape
            .right
            .iter()
            .take_while(|sign| **sign == Sign::blank() || **sign == one())
            .cloned();
        read_one(iter.collect())
    }
}

fn state(str: &str) -> State {
    State::try_from(str).unwrap()
}

// 最後の edge の番号 = n
fn accept_end_only(n: usize) -> Vec<Vec<State>> {
    let mut v = vec![vec![]; n];
    v.push(vec![state("end")]);
    v
}

// 最後の edge の番号 = n
fn series_edge_end_only(n: usize) -> Vec<((usize, usize), State)> {
    (0..n).map(|i| ((i, i + 1), state("end"))).collect()
}

pub fn zero_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("zero_builder").unwrap();
    builder
        .from_source(include_str!("zero_builder.txt"))
        .unwrap();
    builder
}

pub fn succ_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("succ_adder").unwrap();
    builder
        .from_source(include_str!("succ_builder.txt"))
        .unwrap();
    builder
}

pub fn id() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("id").unwrap();
    builder.from_source(include_str!("id.txt")).unwrap();
    builder
}

pub fn move_right() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_right").unwrap();
    builder.from_source(include_str!("move_right.txt")).unwrap();
    builder
}

pub fn move_rights(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: format!("moveR_{n}"),
            init_state: state("start"),
            assign_vertex_to_builder: vec![move_right(); n],
            assign_edge_to_state: series_edge_end_only(n - 1),
            acceptable: accept_end_only(n - 1),
        };
        naive_builder_composition(graph).unwrap()
    }
}

pub fn move_left() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_left").unwrap();
    builder.from_source(include_str!("move_left.txt")).unwrap();
    builder
}

pub fn move_lefts(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: format!("moveL_{n}"),
            init_state: state("start"),
            assign_vertex_to_builder: vec![move_left(); n],
            assign_edge_to_state: series_edge_end_only(n - 1),
            acceptable: accept_end_only(n - 1),
        };
        naive_builder_composition(graph).unwrap()
    }
}

pub fn bor1orbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("bor1").unwrap();
    builder.from_source(include_str!("bor1orbar.txt")).unwrap();
    builder
}

pub fn putb() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putB").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start,  , end, C",
                " , start,  , end, C",
                "1, start,  , end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn put1() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("put1").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start, 1, end, C",
                " , start, 1, end, C",
                "1, start, 1, end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn putbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putbar").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![" , start, -, end, C", "1, start, -, end, C"]
                .into_iter()
                .map(|str| str.try_into().unwrap())
                .collect(),
        );
    builder
}

pub fn right_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("rightone").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start,  , end, R",
                "1, start, 1, end, R",
                "-, start, -, end, R",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

fn annihilate() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "annihilate".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(),
            putb(),
            left_one(),
            bor1orbar(),
            putb(),
            right_one(),
            putbar(),
            left_one(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end1")),
            ((3, 4), state("endB")),
            ((3, 5), state("endbar")),
            ((4, 2), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("end")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn left_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("leftone").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start,  , end, L",
                "1, start, 1, end, L",
                "-, start, -, end, L",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

fn pre_copy() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_procedure_copy".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(),
            right_one(),
            putbar(),
            move_left(),
            move_left(),
        ],
        assign_edge_to_state: series_edge_end_only(4),
        acceptable: accept_end_only(4),
    };
    naive_builder_composition(graph).unwrap()
}

fn copy_this_b() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy_this_b".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            putbar(),
            move_right(),
            move_right(),
            putb(),
            right_one(),
            putbar(),
            move_left(),
            move_left(),
            putb(),
        ],
        assign_edge_to_state: series_edge_end_only(8),
        acceptable: accept_end_only(8),
    };
    naive_builder_composition(graph).unwrap()
}

fn copy_this_1() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy_this_1".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            putbar(),
            move_right(),
            move_right(),
            put1(),
            right_one(),
            putbar(),
            move_left(),
            move_left(),
            put1(),
        ],
        assign_edge_to_state: series_edge_end_only(8),
        acceptable: accept_end_only(8),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn copy() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            pre_copy(),
            right_one(),
            bor1orbar(),
            copy_this_b(),
            copy_this_1(),
            right_one(),
            move_left(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("endB")),
            ((2, 4), state("end1")),
            ((2, 6), state("endbar")),
            ((3, 5), state("end")),
            ((4, 5), state("end")),
            ((5, 2), state("end")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(6),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn copy_n(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: format!("copy_{n}"),
            init_state: state("start"),
            assign_vertex_to_builder: vec![
                vec![vec![copy(), move_right()]; n]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<TuringMachineBuilder>>(),
                vec![move_lefts(n - 1)],
            ]
            .into_iter()
            .flatten()
            .collect(),
            assign_edge_to_state: series_edge_end_only(2 * n),
            acceptable: accept_end_only(2 * n),
        };
        naive_builder_composition(graph).unwrap()
    }
}

// -p_1- ... -p_n- を -p_1- ... -p_n-- にする
fn pre_put_rotate(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_put".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![move_rights(n), right_one(), putbar(), move_lefts(n + 1)],
        assign_edge_to_state: series_edge_end_only(3),
        acceptable: accept_end_only(3),
    };
    naive_builder_composition(graph).unwrap()
}

// -...1...-p_2-...-p_n-...- を -...B...-p_2-...-p_n-...1- にする
fn pre_move_this_1(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_move_this_1".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            putbar(),
            move_rights(n + 1),
            put1(),
            right_one(),
            putbar(),
            move_lefts(n + 1),
            putb(),
        ],
        assign_edge_to_state: series_edge_end_only(6),
        acceptable: accept_end_only(6),
    };
    naive_builder_composition(graph).unwrap()
}

// -...B...-p_2-...-p_n-...- を -...B...-p_2-...-p_n-...1- にする
fn pre_move_this_b(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_move_this_b".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            putbar(),
            move_rights(n + 1),
            putb(),
            right_one(),
            putbar(),
            move_lefts(n + 1),
            putb(),
        ],
        assign_edge_to_state: series_edge_end_only(6),
        acceptable: accept_end_only(6),
    };
    naive_builder_composition(graph).unwrap()
}

// -p_1-p_2-...-p_n- を -Bs-p_2-...p_n-p_1- にする
fn pre_move_this_tuple(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("pre_move_this_tuple_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            pre_put_rotate(n),
            right_one(),
            bor1orbar(),
            pre_move_this_b(n),
            pre_move_this_1(n),
            right_one(),
            move_left(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("endB")),
            ((2, 4), state("end1")),
            ((2, 6), state("endbar")),
            ((3, 5), state("end")),
            ((4, 5), state("end")),
            ((5, 2), state("end")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(6),
    };
    naive_builder_composition(graph).unwrap()
}

// --p- を -p-- にする
fn pre_remove_empty_case() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "remove_empty_case".to_owned(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_rights(2),
            left_one(),
            bor1orbar(), // 0,1,2
            putbar(),
            left_one(),
            bor1orbar(),
            put1(),
            put1(),
            put1(), // 3, 4, 5, 6, 7, 8
            putbar(),
            left_one(),
            bor1orbar(),
            putb(),
            putb(),
            putb(), // 9,10,11,12,13,14
            id(),   //15
            left_one(),
            id(), //16,17
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end1")),
            ((2, 9), state("endB")),
            ((2, 15), state("endbar")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("end1")),
            ((5, 7), state("endB")),
            ((5, 8), state("endbar")),
            ((6, 4), state("end")),
            ((7, 10), state("end")),
            ((8, 16), state("end")),
            ((9, 10), state("end")),
            ((10, 11), state("end")),
            ((11, 12), state("end1")),
            ((11, 13), state("endB")),
            ((11, 14), state("endbar")),
            ((12, 4), state("end")),
            ((13, 10), state("end")),
            ((14, 16), state("end")),
            ((15, 16), state("end")),
            ((16, 17), state("end")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(17),
    };
    naive_builder_composition(graph).unwrap()
}

fn is_empty() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("is_empty").unwrap();
    builder.from_source(include_str!("is_empty.txt")).unwrap();
    builder
}

fn pre_remove_one_pre() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_remove_one_pre".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(),
            left_one(),
            bor1orbar(),
            putbar(),
            pre_remove_empty_case(),
            pre_remove_empty_case(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("endB")),
            ((3, 4), state("end")),
            ((4, 1), state("end")),
            ((2, 5), state("endbar")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(5),
    };
    naive_builder_composition(graph).unwrap()
}

fn pre_remove_one() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "remove_one".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            is_empty(),
            pre_remove_empty_case(),
            move_right(),
            left_one(),
            putbar(),
            pre_remove_empty_case(),
            move_rights(2),
            putb(),
            left_one(),
            move_lefts(2),
            pre_remove_one_pre(),
            move_right(),
            right_one(),
            bor1orbar(),
            putb(),
            putbar(),
            move_lefts(2),
            id(),
        ],
        assign_edge_to_state: vec![
            vec![
                ((0, 1), state("endT")),
                ((1, 17), state("end")),
                ((0, 2), state("endF")),
            ],
            (2..=11).map(|i| ((i, i + 1), state("end"))).collect(),
            vec![
                ((12, 13), state("end")),
                ((13, 14), state("endbar")),
                ((13, 15), state("endB")),
                ((14, 12), state("end")),
                ((15, 16), state("end")),
                ((16, 17), state("end")),
            ],
        ]
        .into_iter()
        .flatten()
        .collect(),
        acceptable: accept_end_only(17),
    };
    naive_builder_composition(graph).unwrap()
}

// -B-p_1-...-p_n- を -p_1-...-p_n-B- にする
fn pre_remove_first(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("pre_remove_first_this_tuple_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: {
            let mut v = Vec::new();
            v.extend(
                vec![vec![pre_remove_one(), move_right()]; n - 1]
                    .into_iter()
                    .flatten(),
            );
            v.push(pre_remove_one());
            v.push(move_lefts(n - 1));
            v
        },
        assign_edge_to_state: series_edge_end_only(2 * n - 1),
        acceptable: accept_end_only(2 * n - 1),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn rotate(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "rotate".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            pre_move_this_tuple(n),
            pre_remove_first(n),
            move_rights(n + 1),
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: series_edge_end_only(4),
        acceptable: accept_end_only(4),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn composition(
    inner_builder: Vec<TuringMachineBuilder>,
    outer_builder: TuringMachineBuilder,
) -> TuringMachineBuilder {
    let num = inner_builder.len();
    if num == 0 {}
    let graph = GraphOfBuilder {
        name: "compose".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            vec![copy_n(num - 1)],
            inner_builder
                .into_iter()
                .map(|builder| {
                    vec![
                        move_rights(num - 1),
                        builder,
                        move_lefts(num - 1),
                        rotate(num),
                    ]
                })
                .flatten()
                .collect(),
            vec![outer_builder],
        ]
        .into_iter()
        .flatten()
        .collect(),
        assign_edge_to_state: series_edge_end_only(4 * num + 1),
        acceptable: accept_end_only(4 * num + 1),
    };
    naive_builder_composition(graph).unwrap()
}

#[cfg(test)]
mod test {

    use recursive_function::machine::NumberTuple;
    use turing_machine::{
        machine::{Sign, State, TapeAsVec, TuringMachineSet},
        manipulation::{
            builder::TuringMachineBuilder,
            graph_compose::{naive_builder_composition, GraphOfBuilder},
        },
    };

    use super::state;
    use super::{
        annihilate, bor1orbar, composition, copy, copy_n, copy_this_1, copy_this_b, id, move_left,
        move_right, move_rights, num_tape, pre_copy, pre_move_this_1, pre_move_this_b,
        pre_move_this_tuple, pre_put_rotate, pre_remove_empty_case, pre_remove_first,
        pre_remove_one, pre_remove_one_pre, put1, putb, putbar, rotate, succ_builder, zero_builder,
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
        let _ = pre_copy();
        let _ = copy_this_b();
        let _ = copy_this_1();
        let _ = copy();
        let _ = copy_n(0);
        let _ = copy_n(1);
        let _ = annihilate();
        let _ = pre_put_rotate(2);
        let _ = pre_move_this_1(2);
        let _ = pre_move_this_b(2);
        let _ = pre_move_this_tuple(2);
        let _ = pre_remove_empty_case();
        let _ = pre_remove_one_pre();
        let _ = pre_remove_one();
        let _ = rotate(3);
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
        let mut builder = pre_copy();

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
        let mut builder = pre_move_this_tuple(2);

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
    fn pre_remove_empty_case_test() {
        let mut builder = pre_remove_empty_case();
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
    fn pre_remove_one_pre_test() {
        let mut builder = pre_remove_one_pre();
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
                    right: vec_sign(vec!["", "1", "", "1", "-", "-", "-", "-", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 500, tests)
    }
    #[test]
    fn pre_remove_one_test() {
        let mut builder = pre_remove_one();
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
        let mut builder = pre_remove_first(2);
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
}
