use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use super::basic::*;
use crate::*;

// [-]X1-...-Xn- を [x]X2-...-Xn-X1- にする
pub fn rotate(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("rotate_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_rights(n),
            right_one(),
            putbar(),
            move_lefts(n + 1),
            right_one(), // 4
            bor1orbar(),
            move_rights(n + 1),
            shift_left_to_rights(sign(""), n + 1),
            putbar(),
            move_rights(n + 1),
            shift_left_to_rights(sign("1"), n + 1),
            putbar(),
            move_rights(n),
            shift_left_to_rights(sign("-"), n + 1),
            move_rights(n + 1),
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("endB")),
            ((5, 9), state("end1")),
            ((5, 12), state("endbar")),
            ((6, 7), state("end")),
            ((7, 8), state("end")),
            ((8, 4), state("end")),
            ((9, 10), state("end")),
            ((10, 11), state("end")),
            ((11, 4), state("end")),
            ((12, 13), state("end")),
            ((13, 14), state("end")),
            ((14, 15), state("end")),
            ((15, 16), state("end")),
        ],
        acceptable: accept_end_only(16),
    };
    builder_composition(graph).unwrap()
}

// [-]X1-...-Xn- を [-]Xn-X1-...-X{n-1}- にする
pub fn rotate_back(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "rotate_back".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            shift_right_to_lefts(sign("-"), n),
            right_one(),
            putbar(),
            move_lefts(n + 1),
            move_rights(n + 1), //4
            left_one(),
            bor1orbar(),       // 6
            move_lefts(n + 1), // 7
            shift_right_to_lefts(sign(""), n + 1),
            putbar(),
            move_lefts(n + 1), // 10
            shift_right_to_lefts(sign("1"), n + 1),
            putbar(),
            right_one(), // 13
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("endB")),
            ((6, 10), state("end1")),
            ((6, 13), state("endbar")),
            ((7, 8), state("end")),
            ((8, 9), state("end")),
            ((9, 5), state("end")),
            ((10, 11), state("end")),
            ((11, 12), state("end")),
            ((12, 5), state("end")),
            ((13, 14), state("end")),
            ((14, 15), state("end")),
        ],
        acceptable: accept_end_only(15),
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_safe() {
        let _ = rotate(2);
    }
    #[test]
    fn rotate_test() {
        let mut builder = rotate(2);
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
                    right: vec_sign(vec!["", "-", "", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-", "", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 600, tests);

        // let mut builder = rotate(3);
        // let tests = vec![
        //     (
        //         TapeAsVec {
        //             left: vec![],
        //             head: sign("-"),
        //             right: vec_sign(vec!["-", "-", "-"]),
        //         },
        //         TapeAsVec {
        //             left: vec![],
        //             head: sign("-"),
        //             right: vec_sign(vec!["-", "-", "-"]),
        //         },
        //     ),
        //     (
        //         TapeAsVec {
        //             left: vec![],
        //             head: sign("-"),
        //             right: vec_sign(vec!["-", "", "1", "", "1", "-", "", "-"]),
        //         },
        //         TapeAsVec {
        //             left: vec![],
        //             head: sign("-"),
        //             right: vec_sign(vec!["", "1", "", "1", "-", "", "-", "-"]),
        //         },
        //     ),
        //     (
        //         TapeAsVec {
        //             left: vec![],
        //             head: sign("-"),
        //             right: vec_sign(vec!["", "", "-", "", "1", "", "1", "-", "", "-"]),
        //         },
        //         TapeAsVec {
        //             left: vec![],
        //             head: sign("-"),
        //             right: vec_sign(vec!["", "1", "", "1", "-", "", "-", "", "", "-"]),
        //         },
        //     ),
        // ];
        // builder_test(&mut builder, 600, tests);
    }
    #[test]
    fn rotate_back_test() {
        let mut builder = rotate_back(3);
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
                    right: vec_sign(vec!["", "-", "-", "", "1", "", "1", "-"]),
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
                    right: vec_sign(vec!["", "-", "", "", "-", "", "1", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 300, tests);
    }
}
