use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

// -l- はタプルとしては現れないのでそれをシグネチャとし、判定する
// -l- が左にあると T そうじゃないと F を返す
fn is_left_sig() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "is_left_sig".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::left_one(), // 0
            basic::check_current(),
            basic::right_one(),
            basic::left_one(),
            basic::check_current(),
            basic::right_one(), //5
            basic::right_one(),
            basic::right_one(), //7
            basic::right_one(),
            basic::id_end("endF"),
            basic::id_end("endT"),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end-".parse_tc().unwrap()),
            ((1, 2), "endx".parse_tc().unwrap()),
            ((2, 9), "end".parse_tc().unwrap()),
            ((1, 3), "endl".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end-".parse_tc().unwrap()),
            ((4, 5), "endl".parse_tc().unwrap()),
            ((4, 7), "endx".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 9), "end".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
            ((8, 10), "end".parse_tc().unwrap()),
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
            vec!["endF".parse_tc().unwrap()],
            vec!["endT".parse_tc().unwrap()],
        ],
    };
    builder_composition(graph).unwrap()
}

// 先頭区画の l を 1 つ削る。空/0 の場合はエラー。
fn expand_aux_shrink() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shrink".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::putb(),
            basic::left_one(),
            basic::check_current(),
            basic::putx(), // 4
            basic::left_one(),
            basic::check_current(),
            basic::putb(),
            basic::putb(),
            basic::putx(),
            basic::putx(), // 10
            basic::left_one(),
            basic::check_current(),
            basic::putl(),
            basic::putl(),
            basic::putx(),
            basic::right_one(), // 16
            basic::putb(),
            basic::left_one(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end-".parse_tc().unwrap()),
            ((3, 10), "endl".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()), // blank-case
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 7), "end-".parse_tc().unwrap()),
            ((6, 8), "endl".parse_tc().unwrap()),
            ((6, 9), "endx".parse_tc().unwrap()),
            ((7, 5), "end".parse_tc().unwrap()),
            ((8, 11), "end".parse_tc().unwrap()),
            ((9, 16), "end".parse_tc().unwrap()),
            ((10, 11), "end".parse_tc().unwrap()), // one-case
            ((11, 12), "end".parse_tc().unwrap()),
            ((12, 13), "end-".parse_tc().unwrap()),
            ((12, 14), "endl".parse_tc().unwrap()),
            ((12, 15), "endx".parse_tc().unwrap()),
            ((13, 5), "end".parse_tc().unwrap()),
            ((14, 11), "end".parse_tc().unwrap()),
            ((15, 16), "end".parse_tc().unwrap()),
            ((16, 17), "end".parse_tc().unwrap()),
            ((17, 18), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(18),
    };
    builder_composition(graph).unwrap()
}

// 先頭区画が空(=0)なら区画を削る。
fn expand_aux_remove_zero() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shrink".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::putb(),
            basic::left_one(),
            basic::check_current(),
            basic::putx(), // 4
            basic::left_one(),
            basic::check_current(),
            basic::putb(),
            basic::putb(),
            basic::putx(), // 9
            basic::left_one(),
            basic::check_current(),
            basic::putl(),
            basic::putl(),
            basic::id(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end-".parse_tc().unwrap()),
            ((3, 9), "endl".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 7), "end-".parse_tc().unwrap()),
            ((6, 8), "endl".parse_tc().unwrap()),
            ((6, 14), "endx".parse_tc().unwrap()),
            ((7, 5), "end".parse_tc().unwrap()),
            ((8, 10), "end".parse_tc().unwrap()),
            ((9, 10), "end".parse_tc().unwrap()),
            ((10, 11), "end".parse_tc().unwrap()),
            ((11, 12), "end-".parse_tc().unwrap()),
            ((11, 13), "endl".parse_tc().unwrap()),
            ((12, 5), "end".parse_tc().unwrap()),
            ((13, 10), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(14),
    };
    builder_composition(graph).unwrap()
}

fn expand_aux_shift_right() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shift_right".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::right_one(),
            basic::check_current(),
            basic::putx(),
            basic::right_one(),
            basic::check_current(),
            basic::putb(),
            basic::putb(),
            basic::putb(),
            basic::putx(),
            basic::right_one(),
            basic::check_current(),
            basic::putl(),
            basic::putl(),
            basic::putl(),
            basic::putx(),
            basic::right_one(),
            basic::putx(),
            basic::move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end-".parse_tc().unwrap()),
            ((1, 8), "endl".parse_tc().unwrap()),
            ((1, 14), "endx".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end-".parse_tc().unwrap()),
            ((4, 6), "endl".parse_tc().unwrap()),
            ((4, 7), "endx".parse_tc().unwrap()),
            ((5, 3), "end".parse_tc().unwrap()),
            ((6, 9), "end".parse_tc().unwrap()),
            ((7, 15), "end".parse_tc().unwrap()),
            ((8, 9), "end".parse_tc().unwrap()),
            ((9, 10), "end".parse_tc().unwrap()),
            ((10, 11), "end-".parse_tc().unwrap()),
            ((10, 12), "endl".parse_tc().unwrap()),
            ((10, 13), "endx".parse_tc().unwrap()),
            ((11, 3), "end".parse_tc().unwrap()),
            ((12, 9), "end".parse_tc().unwrap()),
            ((13, 15), "end".parse_tc().unwrap()),
            ((14, 15), "end".parse_tc().unwrap()),
            ((15, 16), "end".parse_tc().unwrap()),
            ((16, 17), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(17),
    };
    builder_composition(graph).unwrap()
}

// 先頭区画を展開して複製用の区画を並べる。終了時は先頭区画に戻す。
fn expand() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "expand".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            expand_aux_shift_right(),
            basic::right_one(),
            expand_aux_shift_right(),
            basic::putl(),
            basic::right_one(),
            basic::is_tuple_zero(), // 5
            expand_aux_shrink(),
            copy::copy(),
            basic::move_right(),
            expand_aux_remove_zero(), //9
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()),
            ((5, 9), "endT".parse_tc().unwrap()),
            ((5, 6), "endF".parse_tc().unwrap()),
            ((6, 7), "end".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
            ((8, 5), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(9),
    };
    builder_composition(graph).unwrap()
}

// シグネチャを取り除いて通常フォーマットに戻す。
fn format() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "format".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::shift_l2r_fill(crate::symbols::partition_sign()),
            basic::putx(),
            basic::move_rights(2),
            basic::putb(),
            basic::left_one(),
            basic::shift_l2r_fill(crate::symbols::partition_sign()),
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
            crate::compile::succ_builder().get_name()
        ),
        init_state: "start".parse_tc().unwrap(),
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
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 7), "endT".parse_tc().unwrap()),
            ((2, 3), "endF".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 2), "end".parse_tc().unwrap()),
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
            (tape_from(&["x", "x", "x"], 1), "endF".parse_tc().unwrap()),
            (
                tape_from(&["l", "-", "x", "x", "-", "l", "x"], 3),
                "endF".parse_tc().unwrap(),
            ),
            (
                tape_from(&["x", "-", "l", "l", "-", "l", "x"], 0),
                "endF".parse_tc().unwrap(),
            ),
            (
                tape_from(&["l", "x", "x", "-", "-", "l", "x"], 2),
                "endT".parse_tc().unwrap(),
            ),
        ];
        builder_test_predicate(&mut builder, 100, tests);
    }

    #[test]
    fn expand_aux_shrink_test() {
        let mut builder = expand_aux_shrink();
        let tests = vec![
            (
                tape_from(&["x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "-", "l", "x"], 0),
                tape_from(&["x", "-", "-", "l", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "l", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }

    #[test]
    fn expand_aux_remove_zero_test() {
        let mut builder = expand_aux_remove_zero();
        let tests = vec![
            (tape_from(&["x", "-", "x"], 0), tape_from(&["x", "x"], 0)),
            (
                tape_from(&["x", "-", "-", "l", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }

    #[test]
    fn expand_aux_shift_right_test() {
        let mut builder = expand_aux_shift_right();
        let tests = vec![
            (tape_from(&["x", "x"], 0), tape_from(&["x", "x", "x"], 0)),
            (
                tape_from(&["x", "-", "-", "l", "x"], 0),
                tape_from(&["x", "x", "-", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }

    #[test]
    fn expand_test() {
        let mut builder = expand();
        let tests = vec![
            (
                tape_from(&["x", "-", "x"], 0),
                tape_from(&["l", "x", "x", "x"], 2),
            ),
            (
                tape_from(&["x", "-", "l", "l", "-", "l", "x"], 0),
                tape_from(
                    &[
                        "x", "l", "x", "-", "l", "-", "l", "x", "-", "-", "l", "x", "-", "l", "x",
                    ],
                    11,
                ),
            ),
            (
                tape_from(&["x", "-", "l", "l", "l", "-", "l", "x"], 0),
                tape_from(
                    &[
                        "x", "l", "x", "-", "l", "l", "-", "l", "x", "-", "l", "-", "l", "x", "-",
                        "-", "l", "x", "-", "l", "x",
                    ],
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
                tape_from(&["l", "x", "x", "x"], 2),
                tape_from(&["x", "x"], 0),
            ),
            (
                tape_from(&["l", "x", "x", "-", "x"], 2),
                tape_from(&["x", "-", "x"], 0),
            ),
            (
                tape_from(&["l", "x", "x", "-", "l", "l", "-", "x"], 2),
                tape_from(&["x", "-", "l", "l", "-", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
}
