use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{naive_builder_composition, GraphOfBuilder},
};

use super::*;

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

// [-]p- を [-]p-p- にする
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
            move_left(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("endB")),
            ((2, 4), state("end1")),
            ((2, 5), state("endbar")),
            ((3, 1), state("end")),
            ((4, 1), state("end")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(5),
    };
    naive_builder_composition(graph).unwrap()
}

// -p- を -p_1-...-p_n- ただし p_i = p にする
// n = 0 なら -p- を -- に、 n = 1 なら -p- を -p- にする
pub fn n_times_iter(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        annihilate()
    } else if n == 1 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: format!("copy_{n}"),
            init_state: state("start"),
            assign_vertex_to_builder: vec![
                vec![vec![copy(), move_right()]; n - 1]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<TuringMachineBuilder>>(),
                vec![move_lefts(n - 1)],
            ]
            .into_iter()
            .flatten()
            .collect(),
            assign_edge_to_state: series_edge_end_only(2 * (n - 1)),
            acceptable: accept_end_only(2 * (n - 1)),
        };
        naive_builder_composition(graph).unwrap()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_safe() {
        let _ = copy_aux_pre();
        let _ = copy_aux_this_1();
        let _ = copy_aux_this_1();
        let _ = copy();
        let _ = n_times_iter(0);
        let _ = n_times_iter(1);
        let _ = n_times_iter(2);
    }
    #[test]
    fn pre_copy_test() {
        let mut builder = copy_aux_pre();

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
    fn copy_n_times_test() {
        let mut builder = n_times_iter(2);
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
        builder_test(&mut builder, 500, tests);
    }
}
