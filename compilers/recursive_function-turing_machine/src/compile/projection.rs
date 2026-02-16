use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};
use turing_machine::parse::parse_one_code_entry;

use crate::auxiliary::basic;
use crate::*;

// [-]l...l- を --...-[-] にする
fn flat_1_till_end() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("flat_1_till_b").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "-, start, -, next, R",
                "l,  next, -, next, R",
                "-,  next, -,  end, C",
                "x,  next, x,  end,  C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// [-]l...l- を --...-[-] にする
fn move_1_till_end() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_1_till_b").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "-, start, -, next, R",
                "l,  next, l, next, R",
                "-,  next, -,  end, C",
                "x,  next, x,  end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// ...-l...l-...-[-] を ...-l...l[-] にする
// ただし、 --...-[-] の場合は -[-] にする
fn shrink_bar_right_till_1_or_bar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("shrink_bar_right_till_1").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, -, next, L",
                "-,  next, -, next, L",
                "l,  next, l, put0, R",
                "-,  put0, x,  end, C",
                "x,  next, x, put1, R",
                "-,  put1, x,  end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// [-]-...-l...l- を -l...l-...-[-] にする
fn move_1_left_in_bar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_1_left_in_bar").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, x, next, R",
                "-,  next, -, next, R",
                "l,  next, -, stre, L",
                "-,  stre, -, stre, L",
                "l,  stre, l,  put, R",
                "x,  stre, x,  put, R",
                "-,   put, l, next, R",
                "x,  next, x,  end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// [-]l...l- を -xl...l[-] にする
pub fn format() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_1_left_in_bar").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, x, next, R",
                "x,  next, -, post, R",
                "l,  next, -,  put, R",
                "l,   put, l,  put, R",
                "x,   put, l, post, R",
                "-,  post, x,  end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

pub fn projection(n: usize, i: usize) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!("projection_{n}_{i}"),
        init_state: "start".parse_tc().unwrap(),
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
    builder_composition(graph).unwrap()
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
            (tape_from(&["-", "-"], 0), tape_from(&["-", "-"], 1)),
            (tape_from(&["-", "l"], 0), tape_from(&["-", "-"], 1)),
            (
                tape_from(&["-", "l", "l", "l", "-"], 0),
                tape_from(&["-", "-"], 1),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn move_1_till_end_test() {
        let mut builder = move_1_till_end();
        let tests = vec![
            (tape_from(&["-", "-"], 0), tape_from(&["-", "-"], 1)),
            (tape_from(&["-", "l"], 0), tape_from(&["l", "-"], 1)),
            (
                tape_from(&["-", "l", "l", "l", "-"], 0),
                tape_from(&["l", "l", "l", "-"], 3),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn shrink_bar_right_till_1_or_bar_test() {
        let mut builder = shrink_bar_right_till_1_or_bar();
        let tests = vec![
            (tape_from(&["x", "x"], 1), tape_from(&["x", "x"], 1)),
            (
                tape_from(&["l", "x", "x"], 2),
                tape_from(&["l", "x", "x"], 2),
            ),
            (
                tape_from(&["-", "l", "l", "l", "x", "x"], 5),
                tape_from(&["l", "l", "l", "x", "x"], 4),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn move_1_left_in_bar_test() {
        let mut builder = move_1_left_in_bar();
        let tests = vec![
            (tape_from(&["x", "x"], 0), tape_from(&["x", "x"], 1)),
            (
                tape_from(&["x", "l", "x"], 0),
                tape_from(&["l", "x", "x"], 2),
            ),
            (
                tape_from(&["x", "-", "-", "l", "l", "x"], 0),
                tape_from(&["-", "-", "l", "l", "x", "x"], 5),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
    #[test]
    fn format_test() {
        let mut builder = format();
        let tests = vec![
            (tape_from(&["x", "x"], 0), tape_from(&["-", "x", "x"], 2)),
            (
                tape_from(&["x", "l", "x"], 0),
                tape_from(&["l", "-", "x", "x"], 3),
            ),
            (
                tape_from(&["x", "l", "l", "l", "l", "x"], 0),
                tape_from(&["l", "l", "l", "l", "-", "x", "x"], 6),
            ),
        ];
        builder_test(&mut builder, 100, tests);
    }
}
