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
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("endB")),
            ((1, 2), state("endbar")),
            ((2, 9), state("end")),
            ((1, 3), state("end1")),
            ((3, 4), state("end")),
            ((4, 5), state("endB")),
            ((4, 5), state("end1")),
            ((4, 7), state("endbar")),
            ((5, 6), state("end")),
            ((6, 9), state("end")),
            ((7, 8), state("end")),
            ((8, 10), state("end")),
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
            vec![state("endF")],
            vec![state("endT")],
        ],
    };
    builder_composition(graph).unwrap()
}

// -b(x)p- を -b(x-1)p- にする
// -- や -bp- はエラー
fn expand_aux_shrink() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shrink".to_string(),
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("endB")),
            ((3, 10), state("end1")),
            ((4, 5), state("end")), // b-case
            ((5, 6), state("end")),
            ((6, 7), state("endB")),
            ((6, 8), state("end1")),
            ((6, 9), state("endbar")),
            ((7, 5), state("end")),
            ((8, 11), state("end")),
            ((9, 16), state("end")),
            ((10, 11), state("end")), // 1-case
            ((11, 12), state("end")),
            ((12, 13), state("endB")),
            ((12, 14), state("end1")),
            ((12, 15), state("endbar")),
            ((13, 5), state("end")),
            ((14, 11), state("end")),
            ((15, 16), state("end")),
            ((16, 17), state("end")),
            ((17, 18), state("end")),
        ],
        acceptable: accept_end_only(18),
    };
    builder_composition(graph).unwrap()
}

// -bp- を -p- にする
fn expand_aux_remove_zero() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shrink".to_string(),
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("endB")),
            ((3, 9), state("end1")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("endB")),
            ((6, 8), state("end1")),
            ((6, 14), state("endbar")),
            ((7, 5), state("end")),
            ((8, 10), state("end")),
            ((9, 10), state("end")),
            ((10, 11), state("end")),
            ((11, 12), state("endB")),
            ((11, 13), state("end1")),
            ((12, 5), state("end")),
            ((13, 10), state("end")),
        ],
        acceptable: accept_end_only(14),
    };
    builder_composition(graph).unwrap()
}

fn expand_aux_shift_right() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shift_right".to_string(),
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("endB")),
            ((1, 8), state("end1")),
            ((1, 14), state("endbar")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("endB")),
            ((4, 6), state("end1")),
            ((4, 7), state("endbar")),
            ((5, 3), state("end")),
            ((6, 9), state("end")),
            ((7, 15), state("end")),
            ((8, 9), state("end")),
            ((9, 10), state("end")),
            ((10, 11), state("endB")),
            ((10, 12), state("end1")),
            ((10, 13), state("endbar")),
            ((11, 3), state("end")),
            ((12, 9), state("end")),
            ((13, 15), state("end")),
            ((14, 15), state("end")),
            ((15, 16), state("end")),
            ((16, 17), state("end")),
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
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 9), state("endT")),
            ((5, 6), state("endF")),
            ((6, 7), state("end")),
            ((7, 8), state("end")),
            ((8, 5), state("end")),
        ],
        acceptable: accept_end_only(9),
    };
    builder_composition(graph).unwrap()
}

// -1[-]p- を [-]p- にする
fn format() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "format".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::shift_left_to_right_fill(sign("-")),
            basic::putbar(),
            basic::move_rights(2),
            basic::putb(),
            basic::left_one(),
            basic::shift_left_to_right_fill(sign("-")),
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
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 7), state("endT")),
            ((2, 3), state("endF")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 2), state("end")),
        ],
        acceptable: accept_end_only(7),
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
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
                TapeAsVec {
                    left: vec_sign(vec!["-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                state("endF"),
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                state("endF"),
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                state("endF"),
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                state("endT"),
            ),
        ];
        builder_test_predicate(&mut builder, 100, tests);
    }
    #[test]
    fn expand_aux_shrink_test() {
        let mut builder = expand_aux_shrink();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
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
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
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
                    right: vec_sign(vec!["", "", "1", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
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
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
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
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                (
                    vec![
                        "-", "1", "-", "", "1", "", "1", "-", "", "", "1", "-", "", "1", "-",
                    ],
                    11,
                )
                    .try_into()
                    .unwrap(),
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "1", "", "1", "-"]),
                },
                (
                    vec![
                        "-", "1", "-", "", "1", "1", "", "1", "-", "", "1", "", "1", "-", "", "",
                        "1", "-", "", "1", "-",
                    ],
                    17,
                )
                    .try_into()
                    .unwrap(),
            ),
        ];
        builder_test(&mut builder, 1000, tests);
    }
    #[test]
    fn format_test() {
        let mut builder = format();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec![]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec![]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec![]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
}
