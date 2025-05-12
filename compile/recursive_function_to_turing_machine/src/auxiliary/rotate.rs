use turing_machine_core::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use super::basic::*;
use crate::*;

// [-]X1-...-Xn- を [x]X2-...-Xn-X1- にする
pub fn rotate(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("rotate_{n}"),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            move_rights(n),
            right_one(),
            putbar(),
            move_lefts(n + 1),
            right_one(), // 4
            bor1orbar(),
            move_rights(n + 1),
            shift_left_to_rights("".parse().unwrap(), n + 1),
            putbar(),
            move_rights(n + 1),
            shift_left_to_rights("1".parse().unwrap(), n + 1),
            putbar(),
            move_rights(n),
            shift_left_to_rights("-".parse().unwrap(), n + 1),
            move_rights(n + 1),
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()),
            ((5, 6), "endB".parse().unwrap()),
            ((5, 9), "end1".parse().unwrap()),
            ((5, 12), "endbar".parse().unwrap()),
            ((6, 7), "end".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
            ((8, 4), "end".parse().unwrap()),
            ((9, 10), "end".parse().unwrap()),
            ((10, 11), "end".parse().unwrap()),
            ((11, 4), "end".parse().unwrap()),
            ((12, 13), "end".parse().unwrap()),
            ((13, 14), "end".parse().unwrap()),
            ((14, 15), "end".parse().unwrap()),
            ((15, 16), "end".parse().unwrap()),
        ],
        acceptable: accept_end_only(16),
    };
    builder_composition(graph).unwrap()
}

// [-]X1-...-Xn- を [-]Xn-X1-...-X{n-1}- にする
pub fn rotate_back(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "rotate_back".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            shift_right_to_lefts("-".parse().unwrap(), n),
            right_one(),
            putbar(),
            move_lefts(n + 1),
            move_rights(n + 1), //4
            left_one(),
            bor1orbar(),       // 6
            move_lefts(n + 1), // 7
            shift_right_to_lefts("".parse().unwrap(), n + 1),
            putbar(),
            move_lefts(n + 1), // 10
            shift_right_to_lefts("1".parse().unwrap(), n + 1),
            putbar(),
            right_one(), // 13
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 7), "endB".parse().unwrap()),
            ((6, 10), "end1".parse().unwrap()),
            ((6, 13), "endbar".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
            ((8, 9), "end".parse().unwrap()),
            ((9, 5), "end".parse().unwrap()),
            ((10, 11), "end".parse().unwrap()),
            ((11, 12), "end".parse().unwrap()),
            ((12, 5), "end".parse().unwrap()),
            ((13, 14), "end".parse().unwrap()),
            ((14, 15), "end".parse().unwrap()),
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
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "-"]),
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
                    right: vec_sign(vec!["", "-", "", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-", "", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }
    #[test]
    fn rotate_back_test() {
        let mut builder = rotate_back(3);
        let tests = vec![
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "-", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "-", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["-", "", "1", "", "1", "-", "", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-", "-", "", "1", "", "1", "-"]),
                },
            ),
            (
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "", "-", "", "1", "", "1", "-", "", "-"]),
                },
                Tape {
                    left: vec![],
                    head: "-".parse().unwrap(),
                    right: vec_sign(vec!["", "-", "", "", "-", "", "1", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 300, tests);
    }
}
