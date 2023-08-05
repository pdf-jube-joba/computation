use turing_machine::{
    machine::*,
    manipulation::code,
    manipulation::{
        builder::{self, TuringMachineBuilder},
        graph_compose::{naive_builder_composition, GraphOfBuilder},
    },
};

use super::*;

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
            bor1orbar(),
            putbar(),
            left_one(),
            bor1orbar(),
            put1(),
            put1(),
            put1(),
            putbar(),
            left_one(),
            bor1orbar(),
            putb(),
            putb(),
            putb(),
            id(),
            left_one(),
            id(),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_safe() {
        let _ = remove_first_aux_remove_one();
        let _ = remove_one_aux_pre();
        let _ = rotate_aux_remove_first(1);
        let _ = rotate_aux_remove_first(2);
        let _ = rotate_aux_move_this_tuple(1);
        let _ = rotate_aux_move_this_tuple(2);
        let _ = rotate(2);
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
}
