use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

fn start_0() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "start_0".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(),
            basic::right_one(),
            basic::right_one(),
            basic::putbar(),
            basic::move_lefts(2),
        ],
        assign_edge_to_state: series_edge_end_only(4),
        acceptable: accept_end_only(4),
    };
    builder_composition(graph).unwrap()
}

// mu 再帰の作業領域を初期化する。
fn setting() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "setting".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_right(), // 0
            copy::copy(),
            basic::move_left(),
            rotate::rotate(3),
            basic::move_rights(2),
            copy::copy(),
            basic::move_lefts(2),
            rotate::rotate(4),
            rotate::rotate(4),
            rotate::rotate(4),
            basic::move_rights(2),
            basic::concat(),
            basic::id(),
        ],
        assign_edge_to_state: series_edge_end_only(12),
        acceptable: accept_end_only(12),
    };
    builder_composition(graph).unwrap()
}

// mu 再帰の候補値を 1 進める。
fn increment() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "increment".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::right_one(),
            basic::bor1orbar(),
            basic::putb(),
            basic::putb(),
            basic::move_left(),
            basic::put1(),
            basic::right_one(),
            basic::putbar(),
            basic::move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end1".parse_tc().unwrap()),
            ((1, 2), "endB".parse_tc().unwrap()),
            ((2, 0), "end".parse_tc().unwrap()),
            ((1, 3), "endbar".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 7), "end".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(8),
    };
    builder_composition(graph).unwrap()
}

// mu 再帰の候補値が 0 のときに削除する。
fn remove() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "remove".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            basic::move_lefts(2),
            rotate::rotate(3),
            basic::move_right(),
            basic::right_one(),
            basic::bor1orbar(),
            basic::putb(),
            basic::putb(),
            basic::right_one(),
            basic::bor1orbar(),
            basic::putb(),
            basic::putb(),
            basic::move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end1".parse_tc().unwrap()),
            ((4, 5), "endB".parse_tc().unwrap()),
            ((4, 6), "endbar".parse_tc().unwrap()),
            ((5, 3), "end".parse_tc().unwrap()),
            ((6, 7), "end".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
            ((8, 9), "end1".parse_tc().unwrap()),
            ((8, 9), "endB".parse_tc().unwrap()),
            ((8, 10), "endbar".parse_tc().unwrap()),
            ((9, 7), "end".parse_tc().unwrap()),
            ((10, 11), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(11),
    };
    builder_composition(graph).unwrap()
}

pub fn mu_recursion(builder: TuringMachineBuilder) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("mu_recursion_{}", builder.get_name()),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            start_0(),
            setting(),
            builder,
            basic::is_tuple_zero(),
            increment(),
            remove(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 5), "endT".parse_tc().unwrap()),
            ((3, 4), "endF".parse_tc().unwrap()),
            ((4, 1), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(5),
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sage() {
        let _ = start_0();
        let _ = setting();
        let _ = increment();
        let _ = remove();
    }

    #[test]
    fn start_0_test() {
        let mut builder = start_0();
        let tests = vec![(tape_from(&["x", "x"], 0), tape_from(&["x", "x", "-", "x"], 0))];
        builder_test(&mut builder, 100, tests);
    }

    #[test]
    fn setting_test() {
        let mut builder = setting();
        let tests = vec![
            (
                tape_from(&["x", "x", "-", "x"], 0),
                tape_from(&["-", "x", "x", "x", "-", "x"], 3),
            ),
            (
                tape_from(&["x", "x", "-", "l", "x"], 0),
                tape_from(&["l", "-", "x", "x", "x", "-", "l", "x"], 4),
            ),
            (
                tape_from(&["x", "-", "l", "x", "-", "x"], 0),
                tape_from(&["-", "x", "l", "-", "x", "x", "-", "l", "x"], 5),
            ),
        ];
        builder_test(&mut builder, 2000, tests);
    }

    #[test]
    fn increment_test() {
        let mut builder = increment();
        let tests = vec![
            (
                tape_from(&["-", "x", "x", "x", "x"], 3),
                tape_from(&["x", "x", "-", "l", "x"], 0),
            ),
            (
                tape_from(&["-", "x", "l", "-", "x", "x", "-", "l", "l", "x"], 5),
                tape_from(&["x", "-", "l", "x", "-", "l", "x"], 0),
            ),
            (
                tape_from(&["l", "l", "-", "x", "l", "-", "x", "x", "x"], 7),
                tape_from(&["x", "-", "l", "x", "-", "l", "l", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }

    #[test]
    fn remove_test() {
        let mut builder = remove();
        let tests = vec![
            (
                tape_from(&["-", "x", "x", "x", "x"], 3),
                tape_from(&["x", "-", "x"], 0),
            ),
            (
                tape_from(&["-", "x", "l", "-", "x", "x", "-", "l", "l", "x"], 5),
                tape_from(&["x", "-", "x"], 0),
            ),
            (
                tape_from(&["l", "l", "-", "x", "l", "-", "x", "x", "x"], 7),
                tape_from(&["x", "-", "l", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }
}
