use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

// -1-はタプルとしては現れないのでそれをシグネチャとし、判定する
// -1-が左にあると T そうじゃないと F を返す
fn is_left_sig() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "is_left_sig".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            basic::left_one(), // 0
            basic::bor1orbar(),
            basic::right_one(),
            basic::left_one(),
            basic::bor1orbar(),
            basic::right_one(), //5
            basic::right_one(),
            basic::right_one(), //7
            basic::right_one(),
            basic::id_end("endF"),
            basic::id_end("endT"),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "endB".parse().unwrap()),
            ((1, 2), "endbar".parse().unwrap()),
            ((2, 9), "end".parse().unwrap()),
            ((1, 3), "end1".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "endB".parse().unwrap()),
            ((4, 5), "end1".parse().unwrap()),
            ((4, 7), "endbar".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 9), "end".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
            ((8, 10), "end".parse().unwrap()),
        ],
        acceptable: vec![
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec!["endF".parse().unwrap()],
            vec!["endT".parse().unwrap()],
        ],
    };
    builder_composition(graph).unwrap()
}

// -b(x)p- を -b(x-1)p- にする
// -- や -bp- はエラー
fn expand_aux_shrink() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shrink".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::putb(),
            basic::left_one(),
            basic::bor1orbar(),
            basic::putbar(), // 4
            basic::left_one(),
            basic::bor1orbar(),
            basic::putb(),
            basic::putb(),
            basic::putbar(),
            basic::putbar(), // 10
            basic::left_one(),
            basic::bor1orbar(),
            basic::put1(),
            basic::put1(),
            basic::putbar(),
            basic::right_one(), // 16
            basic::putb(),
            basic::left_one(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "endB".parse().unwrap()),
            ((3, 10), "end1".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()), // b-case
            ((5, 6), "end".parse().unwrap()),
            ((6, 7), "endB".parse().unwrap()),
            ((6, 8), "end1".parse().unwrap()),
            ((6, 9), "endbar".parse().unwrap()),
            ((7, 5), "end".parse().unwrap()),
            ((8, 11), "end".parse().unwrap()),
            ((9, 16), "end".parse().unwrap()),
            ((10, 11), "end".parse().unwrap()), // 1-case
            ((11, 12), "end".parse().unwrap()),
            ((12, 13), "endB".parse().unwrap()),
            ((12, 14), "end1".parse().unwrap()),
            ((12, 15), "endbar".parse().unwrap()),
            ((13, 5), "end".parse().unwrap()),
            ((14, 11), "end".parse().unwrap()),
            ((15, 16), "end".parse().unwrap()),
            ((16, 17), "end".parse().unwrap()),
            ((17, 18), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(18),
    };
    builder_composition(graph).unwrap()
}

// -bp- を -p- にする
fn expand_aux_remove_zero() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shrink".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::putb(),
            basic::left_one(),
            basic::bor1orbar(),
            basic::putbar(), // 4
            basic::left_one(),
            basic::bor1orbar(),
            basic::putb(),
            basic::putb(),
            basic::putbar(), // 9
            basic::left_one(),
            basic::bor1orbar(),
            basic::put1(),
            basic::put1(),
            basic::id(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "endB".parse().unwrap()),
            ((3, 9), "end1".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 7), "endB".parse().unwrap()),
            ((6, 8), "end1".parse().unwrap()),
            ((6, 14), "endbar".parse().unwrap()),
            ((7, 5), "end".parse().unwrap()),
            ((8, 10), "end".parse().unwrap()),
            ((9, 10), "end".parse().unwrap()),
            ((10, 11), "end".parse().unwrap()),
            ((11, 12), "endB".parse().unwrap()),
            ((11, 13), "end1".parse().unwrap()),
            ((12, 5), "end".parse().unwrap()),
            ((13, 10), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(14),
    };
    builder_composition(graph).unwrap()
}

fn expand_aux_shift_right() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shift_right".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            basic::right_one(),
            basic::bor1orbar(),
            basic::putbar(),
            basic::right_one(),
            basic::bor1orbar(),
            basic::putb(),
            basic::putb(),
            basic::putb(),
            basic::putbar(),
            basic::right_one(),
            basic::bor1orbar(),
            basic::put1(),
            basic::put1(),
            basic::put1(),
            basic::putbar(),
            basic::right_one(),
            basic::putbar(),
            basic::move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "endB".parse().unwrap()),
            ((1, 8), "end1".parse().unwrap()),
            ((1, 14), "endbar".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "endB".parse().unwrap()),
            ((4, 6), "end1".parse().unwrap()),
            ((4, 7), "endbar".parse().unwrap()),
            ((5, 3), "end".parse().unwrap()),
            ((6, 9), "end".parse().unwrap()),
            ((7, 15), "end".parse().unwrap()),
            ((8, 9), "end".parse().unwrap()),
            ((9, 10), "end".parse().unwrap()),
            ((10, 11), "endB".parse().unwrap()),
            ((10, 12), "end1".parse().unwrap()),
            ((10, 13), "endbar".parse().unwrap()),
            ((11, 3), "end".parse().unwrap()),
            ((12, 9), "end".parse().unwrap()),
            ((13, 15), "end".parse().unwrap()),
            ((14, 15), "end".parse().unwrap()),
            ((15, 16), "end".parse().unwrap()),
            ((16, 17), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(17),
    };
    builder_composition(graph).unwrap()
}

// -b(x)p- を -1-b(x-1)p-...-b(1)p-p- にする
// ただし、展開後は -b(1)p[-]p- の位置にセットする。
fn expand() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "expand".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            expand_aux_shift_right(),
            basic::right_one(),
            expand_aux_shift_right(),
            basic::put1(),
            basic::right_one(),
            basic::is_tuple_zero(), // 5
            expand_aux_shrink(),
            copy::copy(),
            basic::move_right(),
            expand_aux_remove_zero(), //9
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()),
            ((5, 9), "endT".parse().unwrap()),
            ((5, 6), "endF".parse().unwrap()),
            ((6, 7), "end".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
            ((8, 5), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(9),
    };
    builder_composition(graph).unwrap()
}

// -1[-]p- を [-]p- にする
fn format() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "format".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::shift_left_to_right_fill("-".parse().unwrap()),
            basic::putbar(),
            basic::move_rights(2),
            basic::putb(),
            basic::left_one(),
            basic::shift_left_to_right_fill("-".parse().unwrap()),
            basic::move_rights(2),
            basic::putb(),
            basic::move_lefts(2),
        ],
        assign_edge_to_state: series_edge_end_only(9),
        acceptable: accept_end_only(9),
    };
    builder_composition(graph).unwrap()
}

pub fn primitive_recursion(
    zero_case: TuringMachineBuilder,
    succ_case: TuringMachineBuilder,
) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!(
            "primitive_recursion_{}_{}",
            zero_case.get_name(),
            succ_builder().get_name()
        ),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            expand(), // 0
            zero_case,
            is_left_sig(),
            basic::move_left(),
            rotate::rotate(2),
            basic::concat(),
            succ_case,
            format(), // 7
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 7), "endT".parse().unwrap()),
            ((2, 3), "endF".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 2), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(7),
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use turing_machine::manipulation::tape::from_vec_and_position;

    use super::*;

    #[test]
    fn builder_safe() {
        let _ = is_left_sig();
        let _ = expand_aux_shrink();
        let _ = expand_aux_shift_right();
        let _ = expand_aux_shrink();
        let _ = expand();
    }
    #[test]
    fn is_left_sig_test() {
        let mut builder = is_left_sig();
        let tests = vec![
            (
                Tape {
                    left: vec_sign(vec!["-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                "endF".parse().unwrap(),
            ),
            (
                Tape {
                    left: vec_sign(vec!["1", "", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                "endF".parse().unwrap(),
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                "endF".parse().unwrap(),
            ),
            (
                Tape {
                    left: vec_sign(vec!["1", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                "endT".parse().unwrap(),
            ),
        ];
        builder_test_predicate(&mut builder, 100, tests);
    }
    #[test]
    fn expand_aux_shrink_test() {
        let mut builder = expand_aux_shrink();
        let tests = vec![
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn expand_aux_remove_zero_test() {
        let mut builder = expand_aux_remove_zero();
        let tests = vec![
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "1", "", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn expand_aux_shift_right_test() {
        let mut builder = expand_aux_shift_right();
        let tests = vec![
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn expand_test() {
        let mut builder = expand();
        let tests = vec![
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
                Tape {
                    left: vec_sign(vec!["1", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                from_vec_and_position(
                    vec![
                        "-", "1", "-", "", "1", "", "1", "-", "", "", "1", "-", "", "1", "-",
                    ]
                    .into_iter()
                    .map(|s| s.parse().unwrap())
                    .collect(),
                    11,
                ),
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "1", "", "1", "-"]),
                },
                from_vec_and_position(
                    vec![
                        "-", "1", "-", "", "1", "1", "", "1", "-", "", "1", "", "1", "-", "", "",
                        "1", "-", "", "1", "-",
                    ]
                    .into_iter()
                    .map(|s| s.parse().unwrap())
                    .collect(),
                    17,
                ),
            ),
        ];
        builder_test(&mut builder, 1000, tests);
    }
    #[test]
    fn format_test() {
        let mut builder = format();
        let tests = vec![
            (
                Tape {
                    left: vec_sign(vec!["1", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                Tape {
                    left: vec_sign(vec![]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                Tape {
                    left: vec_sign(vec!["1", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
                Tape {
                    left: vec_sign(vec![]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                Tape {
                    left: vec_sign(vec!["1", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "", "-"]),
                },
                Tape {
                    left: vec_sign(vec![]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
}
