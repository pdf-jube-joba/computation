use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{naive_builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

// [b]1...1b を bb...b[b] にする
fn flat_1_till_end() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("flat_1_till_b").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start,  , next, R",
                "1,  next,  , next, R",
                " ,  next,  ,  end, C",
                "-,  next, -,  end,  C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

// [b]1...1b を bb...b[b] にする
fn move_1_till_end() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_1_till_b").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start,  , next, R",
                "1,  next, 1, next, R",
                " ,  next,  ,  end, C",
                "-,  next, -,  end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

// ...b1...1b...b[-] を　...b1...1[-] にする
// ただし、 -b...b[-] の場合は -[-] にする
fn shrink_bar_right_till_1_or_bar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("shrink_bar_right_till_1").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start,  , next, L",
                " ,  next,  , next, L",
                "1,  next, 1, put0, R",
                " ,  put0, -,  end, C",
                "-,  next, -, put1, R",
                " ,  put1, -,  end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

// [-]b...b1...1- を -1...1b...b[-] にする
fn move_1_left_in_bar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_1_left_in_bar").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start, -, next, R",
                " ,  next,  , next, R",
                "1,  next,  , stre, L",
                " ,  stre,  , stre, L",
                "1,  stre, 1,  put, R",
                "-,  stre, -,  put, R",
                " ,   put, 1, next, R",
                "-,  next, -,  end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

// [-]1...1- を -b1...1[-] にする
pub fn format() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_1_left_in_bar").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start, -, next, R",
                "-,  next,  , post, R",
                "1,  next,  ,  put, R",
                "1,   put, 1,  put, R",
                "-,   put, 1, post, R",
                " ,  post, -,  end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn projection(n: usize, i: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("projection_{n}_{i}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            vec![basic::right_one()],
            vec![flat_1_till_end(); i],
            vec![move_1_till_end()],
            vec![flat_1_till_end(); n - i - 1],
            vec![
                shrink_bar_right_till_1_or_bar(),
                basic::move_left(),
                move_1_left_in_bar(),
                shrink_bar_right_till_1_or_bar(),
                basic::move_left(),
                format(),
                basic::move_left(),
            ],
        ]
        .into_iter()
        .flatten()
        .collect(),
        assign_edge_to_state: series_edge_end_only(n + 7),
        acceptable: accept_end_only(n + 7),
    };
    naive_builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_safe() {
        let _ = flat_1_till_end();
        let _ = move_1_till_end();
        let _ = shrink_bar_right_till_1_or_bar();
        let _ = move_1_left_in_bar();
        let _ = format();
    }

    #[test]
    fn flat_1_till_end_test() {
        let mut builder = flat_1_till_end();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec![""]),
                },
                TapeAsVec {
                    left: vec_sign(vec![""]),
                    head: sign(""),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec!["1"]),
                },
                TapeAsVec {
                    left: vec_sign(vec![""]),
                    head: sign(""),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec!["1", "1", "1", ""]),
                },
                TapeAsVec {
                    left: vec_sign(vec![""]),
                    head: sign(""),
                    right: vec![],
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn move_1_till_end_test() {
        let mut builder = move_1_till_end();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec![""]),
                },
                TapeAsVec {
                    left: vec_sign(vec![""]),
                    head: sign(""),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec!["1"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1"]),
                    head: sign(""),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec!["1", "1", "1", ""]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "1", "1"]),
                    head: sign(""),
                    right: vec![],
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn shrink_bar_right_till_1_or_bar_test() {
        let mut builder = shrink_bar_right_till_1_or_bar();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec_sign(vec!["-"]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec_sign(vec!["-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "1", "1", "1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "1", "1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn move_1_left_in_bar_test() {
        let mut builder = move_1_left_in_bar();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["1", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "", "1", "1", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["", "", "1", "1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn format_test() {
        let mut builder = format();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["1", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["1", "1", "1", "1", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["1", "1", "1", "1", "", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
}
