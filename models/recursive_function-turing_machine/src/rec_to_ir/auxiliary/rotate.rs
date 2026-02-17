/*
use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use super::basic::*;
use crate::symbols;
use crate::*;

// [-]X1-...-Xn- を [x]X2-...-Xn-X1- にする
pub fn rotate(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("rotate_{n}"),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            move_rights(n),
            right_one(),
            putx(),
            move_lefts(n + 1),
            right_one(), // 4
            check_branch(),
            move_rights(n + 1),
            shift_left_to_rights(symbols::S::B.into(), n + 1),
            putx(),
            move_rights(n + 1),
            shift_left_to_rights(symbols::S::L.into(), n + 1),
            putx(),
            move_rights(n),
            shift_left_to_rights(symbols::S::X.into(), n + 1),
            move_rights(n + 1),
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()),
            ((5, 6), "endb".parse_tc().unwrap()),
            ((5, 9), "endl".parse_tc().unwrap()),
            ((5, 12), "endx".parse_tc().unwrap()),
            ((6, 7), "end".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
            ((8, 4), "end".parse_tc().unwrap()),
            ((9, 10), "end".parse_tc().unwrap()),
            ((10, 11), "end".parse_tc().unwrap()),
            ((11, 4), "end".parse_tc().unwrap()),
            ((12, 13), "end".parse_tc().unwrap()),
            ((13, 14), "end".parse_tc().unwrap()),
            ((14, 15), "end".parse_tc().unwrap()),
            ((15, 16), "end".parse_tc().unwrap()),
        ],
        acceptable: accept_end_only(16),
    };
    builder_composition(graph).unwrap()
}

// [-]X1-...-Xn- を [-]Xn-X1-...-X{n-1}- にする
pub fn rotate_back(n: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "rotate_back".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            shift_right_to_lefts(symbols::S::X.into(), n),
            right_one(),
            putx(),
            move_lefts(n + 1),
            move_rights(n + 1), //4
            left_one(),
            check_branch(),    // 6
            move_lefts(n + 1), // 7
            shift_right_to_lefts(symbols::S::B.into(), n + 1),
            putx(),
            move_lefts(n + 1), // 10
            shift_right_to_lefts(symbols::S::L.into(), n + 1),
            putx(),
            right_one(), // 13
            putb(),
            move_lefts(n + 1),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 7), "endb".parse_tc().unwrap()),
            ((6, 10), "endl".parse_tc().unwrap()),
            ((6, 13), "endx".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
            ((8, 9), "end".parse_tc().unwrap()),
            ((9, 5), "end".parse_tc().unwrap()),
            ((10, 11), "end".parse_tc().unwrap()),
            ((11, 12), "end".parse_tc().unwrap()),
            ((12, 5), "end".parse_tc().unwrap()),
            ((13, 14), "end".parse_tc().unwrap()),
            ((14, 15), "end".parse_tc().unwrap()),
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
                tape_from(&["x", "x", "x"], 0),
                tape_from(&["x", "x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "x", "-", "x"], 0),
                tape_from(&["x", "-", "x", "-", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 600, tests);
    }
    #[test]
    fn rotate_back_test() {
        let mut builder = rotate_back(3);
        let tests = vec![
            (
                tape_from(&["x", "x", "x", "x"], 0),
                tape_from(&["x", "x", "x", "x"], 0),
            ),
            (
                tape_from(&["x", "x", "-", "l", "-", "l", "x", "-", "x"], 0),
                tape_from(&["x", "-", "x", "x", "-", "l", "-", "l", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "-", "x", "-", "l", "-", "l", "x", "-", "x"], 0),
                tape_from(&["x", "-", "x", "-", "-", "x", "-", "l", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 300, tests);
    }
}
*/
