/*
use turing_machine::manipulation::builder::TuringMachineBuilder;

use super::basic::*;
use crate::*;

fn copy_aux_pre() -> TuringMachineBuilder {
    chain_builders(
        "pre_procedure_copy",
        vec![move_right_till_x(), right_one(), putx(), move_left_till_x(), move_left_till_x()],
    )
}

fn copy_aux_this_b() -> TuringMachineBuilder {
    chain_builders(
        "copy_this_b",
        vec![
            putx(),
            move_right_till_x(),
            move_right_till_x(),
            putb(),
            right_one(),
            putx(),
            move_left_till_x(),
            move_left_till_x(),
            putb(),
        ],
    )
}

fn copy_aux_this_1() -> TuringMachineBuilder {
    chain_builders(
        "copy_this_1",
        vec![
            putx(),
            move_right_till_x(),
            move_right_till_x(),
            put1(),
            right_one(),
            putx(),
            move_left_till_x(),
            move_left_till_x(),
            put1(),
        ],
    )
}

// [-]x- を [-]x-x- にする
pub fn copy() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "copy".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            copy_aux_pre(),
            right_one(),
            check_branch(),
            copy_aux_this_b(),
            copy_aux_this_1(),
            move_left_till_x(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "endb".parse_tc().unwrap()),
            ((2, 4), "endl".parse_tc().unwrap()),
            ((2, 5), "endx".parse_tc().unwrap()),
            ((3, 1), "end".parse_tc().unwrap()),
            ((4, 1), "end".parse_tc().unwrap()),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(5),
    };
    builder_composition(graph).unwrap()
}

// -x- を -x_1-...-x_n- ただし x_i = x にする
// n = 0 なら -x- を -- に、 n = 1 なら -x- を -x- にする
pub fn n_times_iter(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        annihilate()
    } else if n == 1 {
        id()
    } else {
        chain_builders(
            format!("copy_{n}"),
            vec![
                vec![vec![copy(), move_right_till_x()]; n - 1]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<TuringMachineBuilder>>(),
                vec![move_lefts(n - 1)],
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
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
                tape_from(&["x", "x"], 0),
                tape_from(&["x", "x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "x", "x"], 0),
            ),
        ];

        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn copy_test() {
        let mut builder = copy();
        let tests = vec![
            (
                tape_from(&["x", "x"], 0),
                tape_from(&["x", "x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "x", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn copy_n_times_test() {
        let mut builder = n_times_iter(2);
        let tests = vec![
            (
                tape_from(&["x", "x"], 0),
                tape_from(&["x", "x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "x", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
}
*/
