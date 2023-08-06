use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{naive_builder_composition, GraphOfBuilder},
};

use super::{
    basic::{copy::copy, rotate::rotate},
    *,
};

fn start_0() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "start_0".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(),
            right_one(),
            right_one(),
            putbar(),
            move_lefts(2),
        ],
        assign_edge_to_state: series_edge_end_only(4),
        acceptable: accept_end_only(4),
    };
    naive_builder_composition(graph).unwrap()
}

// [-]p-bx- を -p-bx[-]bxp- にする
fn setting() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "setting".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(), // 0
            copy::copy(),
            move_left(),
            rotate::rotate(3),
            move_rights(2),
            copy(),
            move_lefts(2),
            rotate::rotate(4),
            rotate::rotate(4),
            rotate::rotate(4),
            move_rights(2),
            concat(),
            id(),
        ],
        assign_edge_to_state: series_edge_end_only(12),
        acceptable: accept_end_only(12),
    };
    naive_builder_composition(graph).unwrap()
}

// -p-bx[-]X- => [-]p-b{x+1}-
fn increment() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "increment".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            right_one(),
            bor1orbar(),
            putb(),
            putb(),
            move_left(),
            put1(),
            right_one(),
            putbar(),
            move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end1")),
            ((1, 2), state("endB")),
            ((2, 0), state("end")),
            ((1, 3), state("endbar")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("end")),
            ((7, 8), state("end")),
        ],
        acceptable: accept_end_only(8),
    };
    naive_builder_composition(graph).unwrap()
}

// -p-bx[-]X- X=0 => [-]bx-
fn remove() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "remove".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_lefts(2),
            rotate(3),
            move_right(),
            right_one(),
            bor1orbar(),
            putb(),
            putb(),
            right_one(),
            bor1orbar(),
            putb(),
            putb(),
            move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("end1")),
            ((4, 5), state("endB")),
            ((4, 6), state("endbar")),
            ((5, 3), state("end")),
            ((6, 7), state("end")),
            ((7, 8), state("end")),
            ((8, 9), state("end1")),
            ((8, 9), state("endB")),
            ((8, 10), state("endbar")),
            ((9, 7), state("end")),
            ((10, 11), state("end")),
        ],
        acceptable: accept_end_only(11),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn mu_recursion(builder: TuringMachineBuilder) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("mu_recursion_{}", builder.get_name()),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            start_0(),
            setting(),
            builder,
            is_tuple_zero(),
            increment(),
            remove(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 5), state("endT")),
            ((3, 4), state("endF")),
            ((4, 1), state("end")),
        ],
        acceptable: accept_end_only(5),
    };
    naive_builder_composition(graph).unwrap()
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
        let tests = vec![(
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-"]),
            },
            TapeAsVec {
                left: vec![],
                head: sign("-"),
                right: vec_sign(vec!["-", "", "-"]),
            },
        )];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn setting_test() {
        let mut builder = setting();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "", "-", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "-", "", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "1", "", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
            ),
            // (
            //     TapeAsVec {
            //         left: vec![],
            //         head: sign("-"),
            //         right: vec_sign(vec!["", "1", "-", "", "1", "1", "-"]),
            //     },
            //     TapeAsVec {
            //         left: vec![],
            //         head: sign("-"),
            //         right: vec_sign(vec![
            //             "", "1", "-", "", "1", "1", "-", "", "1", "1", "", "1", "-",
            //         ]),
            //     },
            // ),
        ];
        builder_test(&mut builder, 2000, tests);
    }
    #[test]
    fn increment_test() {
        let mut builder = increment();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "", "1", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "1", "", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "-", "", "1", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "1", "", "-", "1", "", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "-", "", "1", "1", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }

    #[test]
    fn remove_test() {
        let mut builder = remove();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "1", "", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "1", "", "-", "1", "", "-"]),
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }
}
