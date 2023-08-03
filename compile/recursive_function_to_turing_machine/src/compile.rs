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

fn accept_end_only(n: usize) -> Vec<Vec<State>> {
    let mut v = vec![vec![]; n-1];
    v.push(vec![state("end")]);
    v
}

pub fn zero_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("zero_builder").unwrap();
    builder.from_source(include_str!("zero_builder.txt")).unwrap();
    builder
}

pub fn succ_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("succ_adder").unwrap();
    builder.from_source(include_str!("succ_builder.txt")).unwrap();  
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
    let graph = GraphOfBuilder {
        name: format!("moveR_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![move_right(); n],
        assign_edge_to_state: (0..n-1).map(|i| ((i, i+1), state("end"))).collect(),
        acceptable: accept_end_only(n),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn move_left() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_left").unwrap();
    builder.from_source(include_str!("move_left.txt")).unwrap();
    builder
}

pub fn move_lefts(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("moveL_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![move_left(); n],
        assign_edge_to_state: (0..n-1).map(|i| ((i, i+1), state("end"))).collect(),
        acceptable: accept_end_only(n),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn bor1orbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("bor1").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(vec![
            " , start,  , endB, C",
            "1, start, 1, end1, C",
            "-, start, -, endbar, C",
        ].into_iter().map(|str| str.try_into().unwrap()).collect());
    builder
}

pub fn putb() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putB").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(vec![
            "-, start,  , end, C",
        ].into_iter().map(|str| str.try_into().unwrap()).collect());
    builder
}

pub fn put1() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("put1").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(vec![
            "-, start, 1, end, C",
        ].into_iter().map(|str| str.try_into().unwrap()).collect());
    builder
}

pub fn putbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putbar").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(vec![
            " , start, -, end, C",
            "1, start, -, end, C",
        ].into_iter().map(|str| str.try_into().unwrap()).collect());
    builder
}

pub fn right_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("rightone").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(vec![
            " , start,  , end, R",
            "1, start, 1, end, R",
            "-, start, -, end, R",
        ].into_iter().map(|str| str.try_into().unwrap()).collect());
    builder
}

fn pre_copy() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_procedure_copy".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![move_right(), right_one(), putbar(), move_left(), move_left()],
        assign_edge_to_state: vec![
            ((0,1), state("end")),
            ((1,2), state("end")),
            ((2,3), state("end")),
            ((3,4), state("end")),
        ].into_iter().collect(),
        acceptable: vec![vec![], vec![], vec![], vec![], vec![state("end")]],
    };
    naive_builder_composition(graph).unwrap()
}

fn copy_this_b() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy_this_b".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![putbar(), move_right(), move_right(), putb(), right_one(), putbar(), move_left(), move_left(), putb()],
        assign_edge_to_state: vec![
            ((0,1), state("end")),
            ((1,2), state("end")),
            ((2,3), state("end")),
            ((3,4), state("end")),
            ((4,5), state("end")),
            ((5,6), state("end")),
            ((6,7), state("end")),
            ((7,8), state("end")),
        ].into_iter().collect(),
        acceptable: accept_end_only(9),
    };
    naive_builder_composition(graph).unwrap()
}

fn copy_this_1() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy_this_1".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![putbar(), move_right(), move_right(), put1(), right_one(), putbar(), move_left(), move_left(), put1()],
        assign_edge_to_state: vec![
            ((0,1), state("end")),
            ((1,2), state("end")),
            ((2,3), state("end")),
            ((3,4), state("end")),
            ((4,5), state("end")),
            ((5,6), state("end")),
            ((6,7), state("end")),
            ((7,8), state("end")),
        ].into_iter().collect(),
        acceptable: accept_end_only(9),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn copy() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![pre_copy(), right_one(), bor1orbar(), copy_this_b(), copy_this_1(), right_one(), move_left()],
        assign_edge_to_state: vec![
            ((0,1), state("end")),
            ((1,2), state("end")),
            ((2,3), state("endB")),
            ((2,4), state("end1")),
            ((2,6), state("endbar")),
            ((3,5), state("end")),
            ((4,5), state("end")),
            ((5,2), state("end")),
        ].into_iter().collect(),
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn copy_n(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: "a".to_string(),
            init_state: state("start"),
            assign_vertex_to_builder: vec![copy(), move_right(), copy_n(n - 1), move_left()],
            assign_edge_to_state: HashMap::new(),
            acceptable: vec![vec![], vec![State::try_from("end").unwrap()]],
        };
        naive_builder_composition(graph).unwrap()
    }
}

// -p_1- ... -p_n- を -p_1- ... -p_n-- にする
fn pre_put_rotate(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_put".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![move_rights(n), right_one(), putbar(), move_lefts(n+1)],
        assign_edge_to_state: (0..3).map(|i| ((i, i+1), state("end"))).collect(),
        acceptable: accept_end_only(4),
    };
    naive_builder_composition(graph).unwrap()
}

// -...1...-p_2-...-p_n-...- を -...B...-p_2-...-p_n-...1- にする
fn pre_move_this_1(n :usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_move_this_1".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![putbar(), move_rights(n+1), put1(), right_one(), putbar(), move_lefts(n+1), putb()],
        assign_edge_to_state: vec![].into_iter().collect(),
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

// -...B...-p_2-...-p_n-...- を -...B...-p_2-...-p_n-...1- にする
fn pre_move_this_b(n :usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_move_this_1".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![putbar(), move_rights(n+1), putb(), right_one(), putbar(), move_lefts(n+1), putb()],
        assign_edge_to_state: vec![].into_iter().collect(),
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

fn pre_move_this_tuple(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("pre_move_this_tuple_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![pre_put_rotate(n), right_one(), bor1orbar(), pre_move_this_b(n), pre_move_this_1(n), right_one(), move_left()],
        assign_edge_to_state: vec![
            ((0,1), state("end")),
            ((1,2), state("end")),
            ((2,3), state("endB")),
            ((2,4), state("end1")),
            ((2,6), state("endbar")),
            ((3,5), state("end")),
            ((4,5), state("end")),
            ((5,2), state("end")),
        ].into_iter().collect(),
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn rotate() -> TuringMachineBuilder {
    let code = code::parse_code(include_str!("rotate.txt")).unwrap();
    let mut builder = TuringMachineBuilder::new("bin_adder").unwrap();
    builder
        .code_new(code)
        .init_state(state("start"))
        .accepted_state(vec![State::try_from("end").unwrap()]);
    builder
}

pub fn composition(
    inner_builder: Vec<TuringMachineBuilder>,
    outer_builder: TuringMachineBuilder,
) -> TuringMachineBuilder {
    unimplemented!()
}

#[cfg(test)]
mod test {

    use recursive_function::machine::NumberTuple;
    use turing_machine::{
        machine::{State, TapeAsVec, Sign, TuringMachineSet},
        manipulation::graph_compose::{naive_builder_composition, GraphOfBuilder},
    };

    use super::{move_left, move_right, num_tape, succ_builder, zero_builder, bor1orbar, id, put1, putb, putbar, pre_copy, copy_this_b, copy_this_1, copy, pre_move_this_tuple, pre_put_rotate, pre_move_this_1, pre_move_this_b, move_rights};
    use super::state;

    fn sign(str: &str) -> Sign {
        Sign::try_from(str).unwrap()
    }
    fn vec_sign(vec: Vec<&str>) -> Vec<Sign> {
        vec.into_iter().map(|str| sign(str)).collect()
    }
    fn view_step(machine: &mut TuringMachineSet, step: usize) {
        eprintln!("start");
        eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
        for _ in 0..step {
            let _ = machine.step(1);
            eprintln!("{:?}\n    {}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }
    }
    // #[test]
    // fn view_tape() {
    //     let tape = TapeAsVec {
    //         left: vec![],
    //         head: sign("-"),
    //         right: vec_sign(vec!["a", "b", "-"]),
    //     };
    //     eprintln!("{tape}");
    // }

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
        let _ = pre_put_rotate(2);
        let _ = pre_move_this_1(2);
        let _ = pre_move_this_b(2);
        let _ = pre_move_this_tuple(2);
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
            eprintln!("start: {} {:?}", machine.now_state(), machine.now_tape());
            loop {
                let _ = machine.step(1);
                eprintln!("next: {} {:?}", machine.now_state(), machine.now_tape());
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
        eprintln!("code:");
        for entry in builder.get_code() {
            eprintln!("    {:?}", entry);
        }
        eprintln!("init: {:?}", builder.get_init_state());
        eprintln!("accp: {:?}", builder.get_accepted_state());
        builder.input(num_tape::write(vec![1, 0].into()));

        let mut machine = builder.build().unwrap();
        eprintln!("start: {} {:?}", machine.now_state(), machine.now_tape());
        for _ in 0..50 {
            let _ = machine.step(1);
            eprintln!("next : {} {:?}", machine.now_state(), machine.now_tape());
            if machine.is_terminate() {
                break;
            }
        }
        let result = num_tape::read_right_one(machine.now_tape());
        assert_eq!(result, Ok(vec![1, 0].into()))
    }
    #[test]
    fn pre_copy_test() {
        let mut pre = pre_copy();
        // eprintln!("{:?} {:?} {:?}", pre.get_init_state(), pre.get_accepted_state(), pre.get_code());

        let tests = vec![
            (TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            }, TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "-"])
            }),

            (TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-"]),
            }, TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-", "-"]),
            }),
        ];
        for (input, expect) in tests {
            let mut machine = pre.input(input).build().unwrap();
            view_step(&mut machine, 100);
            // assert!(machine.is_accepted());
            assert_eq!(machine.now_tape(), expect);
        }
    }
    #[test]
    fn copy_test() {
        let mut copy = copy();
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
                }
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
                }
            ),
        ];
        for (input, expect) in tests {
            let mut machine = copy.input(input).build().unwrap();
            view_step(&mut machine, 100);
            // assert!(machine.is_accepted());
            assert_eq!(machine.now_tape(), expect);
        }
    }
    #[test]
    fn pre_move_test() {
        let mut pre = pre_move_this_tuple(2);
        // eprintln!("{:?} {:?} {:?}", pre.get_init_state(), pre.get_accepted_state(), pre.get_code());

        let tests = vec![
            (TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-", "-"]),
            }, TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["", "1", "-", "-", "", "1", "-"]),
            }),
        ];
        for (input, expect) in tests {
            let mut machine = pre.input(input).build().unwrap();
            view_step(&mut machine, 300);
            assert!(machine.is_accepted());
            assert_eq!(machine.now_tape(), expect);
        }
    }
}
