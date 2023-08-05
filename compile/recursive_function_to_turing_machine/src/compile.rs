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

fn copy_aux_pre() -> TuringMachineBuilder {
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

fn copy_aux_this_b() -> TuringMachineBuilder {
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

fn copy_aux_this_1() -> TuringMachineBuilder {
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
            copy_aux_pre(),
            right_one(),
            bor1orbar(),
            copy_aux_this_b(),
            copy_aux_this_1(),
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

pub fn n_times_copy(n: usize) -> TuringMachineBuilder {
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
fn rotate_aux_move_this_tuple(n: usize) -> TuringMachineBuilder {
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
fn move_empty_case() -> TuringMachineBuilder {
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

fn remove_one_aux_pre() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "pre_remove_one_pre".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(),
            left_one(),
            bor1orbar(),
            putbar(),
            move_empty_case(),
            move_empty_case(),
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

// -B-p- を -p-B- にする
fn remove_first_aux_remove_one() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "remove_one".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            is_empty(),
            move_empty_case(),
            move_right(),
            left_one(),
            putbar(),
            move_empty_case(),
            move_rights(2),
            putb(),
            left_one(),
            move_lefts(2),
            remove_one_aux_pre(),
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
fn rotate_aux_remove_first(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("pre_remove_first_this_tuple_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: {
            let mut v = Vec::new();
            v.extend(
                vec![vec![remove_first_aux_remove_one(), move_right()]; n - 1]
                    .into_iter()
                    .flatten(),
            );
            v.push(remove_first_aux_remove_one());
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
            rotate_aux_move_this_tuple(n),
            rotate_aux_remove_first(n),
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
            vec![n_times_copy(num - 1)],
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
mod tests;
