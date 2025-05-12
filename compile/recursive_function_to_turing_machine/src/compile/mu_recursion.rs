use turing_machine_core::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

fn start_0() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "start_0".to_string(),
        init_state: "start".parse().unwrap(),
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

// [-]p-bx- を -p-bx[-]bxp- にする
fn setting() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "setting".to_string(),
        init_state: "start".parse().unwrap(),
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

// -p-bx[-]X- => [-]p-b{x+1}-
fn increment() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "increment".to_string(),
        init_state: "start".parse().unwrap(),
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
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end1".parse().unwrap()),
            ((1, 2), "endB".parse().unwrap()),
            ((2, 0), "end".parse().unwrap()),
            ((1, 3), "endbar".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 7), "end".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(8),
    };
    builder_composition(graph).unwrap()
}

// -p-bx[-]X- X=0 => [-]bx-
fn remove() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "remove".to_string(),
        init_state: "start".parse().unwrap(),
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
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end1".parse().unwrap()),
            ((4, 5), "endB".parse().unwrap()),
            ((4, 6), "endbar".parse().unwrap()),
            ((5, 3), "end".parse().unwrap()),
            ((6, 7), "end".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
            ((8, 9), "end1".parse().unwrap()),
            ((8, 9), "endB".parse().unwrap()),
            ((8, 10), "endbar".parse().unwrap()),
            ((9, 7), "end".parse().unwrap()),
            ((10, 11), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(11),
    };
    builder_composition(graph).unwrap()
}

pub fn mu_recursion(builder: TuringMachineBuilder) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("mu_recursion_{}", builder.get_name()),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            start_0(),
            setting(),
            builder,
            basic::is_tuple_zero(),
            increment(),
            remove(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 5), "endT".parse().unwrap()),
            ((3, 4), "endF".parse().unwrap()),
            ((4, 1), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(5),
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use turing_machine_core::machine::Tape;
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
            Tape {
                left: vec![],
                head: "-".parse().unwrap(),
                right: vec_sign(vec!["-"]),
            },
            Tape {
                left: vec![],
                head: "-".parse().unwrap(),
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
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "", "-"]),
                },
                Tape {
                    left: vec_sign(vec!["", "-", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "", "1", "-"]),
                },
                Tape {
                    left: vec_sign(vec!["1", "", "-", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "-", "", "-"]),
                },
                Tape {
                    left: vec_sign(vec!["", "-", "1", "", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
            ),
            // (
            //     Tape {
            //         left: vec![],
            //         head: "-".parse().unwrap(),
            //         right: vec_sign(vec!["", "1", "-", "", "1", "1", "-"]),
            //     },
            //     Tape {
            //         left: vec![],
            //         head: "-".parse().unwrap(),
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
                Tape {
                    left: vec_sign(vec!["", "-", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "", "1", "-"]),
                },
            ),
            (
                Tape {
                    left: vec_sign(vec!["", "-", "1", "", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "-", "", "1", "-"]),
                },
            ),
            (
                Tape {
                    left: vec_sign(vec!["1", "1", "", "-", "1", "", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
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
                Tape {
                    left: vec_sign(vec!["", "-", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                Tape {
                    left: vec_sign(vec!["", "-", "1", "", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                Tape {
                    left: vec_sign(vec!["1", "1", "", "-", "1", "", "-"]),
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }
}
