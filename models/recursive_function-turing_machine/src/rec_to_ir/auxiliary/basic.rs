/*
use turing_machine::{
    machine::*,
    manipulation::{
        builder::TuringMachineBuilder,
        graph_compose::{builder_composition, GraphOfBuilder},
    },
    parse::parse_one_code_entry,
};
use utils::TextCodec;

use crate::symbols;
use crate::*;

pub fn id() -> TuringMachineBuilder {
    Builder {
        name: "id".to_string(),
        code: vec!["x, start, x, end, C"],
    }
    .into()
}

pub fn id_end(str: &str) -> TuringMachineBuilder {
    Builder {
        name: format!("id_{str}"),
        code: vec![
            &format!("x, start, x, {str}, C"),
            &format!("-, start, -, {str}, C"),
            &format!("l, start, l, {str}, C"),
        ],
    }
    .into()
}

// ... [?_1] ?_2 ... ?_n ...
// ... ?_1 ?_2 ... [?_n] ...
// where
//  - ?_1 in {'-', 'l', 'x'}
//  - ?_i in {'-', 'l'} for 1 < i < n
//  - ?_n == 'x'
pub fn move_right_till_x() -> TuringMachineBuilder {
    Builder {
        name: "move_right".to_string(),
        code: vec![
            "x, start, x, till, R",
            "-, start, -, till, R",
            "l, start, l, till, R",
            "-,  till, -, till, R",
            "l,  till, l, till, R",
            "x,  till, x,  end, C",
        ],
    }
    .into()
}

pub fn move_rights(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        chain_builders(format!("moveR_{n}"), vec![move_right_till_x(); n])
    }
}

// ... ?_1 ?_2 ... [?_n] ...
// ... [?_1] ?_2 ... ?_n ...
// where
//  - ?_n in {'-', 'l', 'x'}
//  - ?_i in {'-', 'l'} for 1 < i < n
//  - ?_1 == 'x'
pub fn move_left_till_x() -> TuringMachineBuilder {
    Builder {
        name: "move_left".to_string(),
        code: vec![
            "x, start, x, till, L",
            "-, start, -, till, L",
            "l, start, l, till, L",
            "-,  till, -, till, L",
            "l,  till, l, till, L",
            "x,  till, x,  end, C",
        ],
    }
    .into()
}

pub fn move_lefts(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        chain_builders(format!("moveL_{n}"), vec![move_left_till_x(); n])
    }
}

// blank        ('-') -> endb
// flag         ('l') -> endl
// partition    ('x') -> endx
pub fn check_branch() -> TuringMachineBuilder {
    Builder {
        name: "check_branch".to_string(),
        code: vec![
            "-, start, -, endb, C",
            "l, start, l, endl, C",
            "x, start, x, endx, C",
        ],
    }
    .into()
}

pub fn put(x: symbols::S) -> TuringMachineBuilder {
    Builder {
        name: format!("put_{}", x),
        code: vec![
            &format!("x, start, {}, end, C", x),
            &format!("-, start, {}, end, C", x),
            &format!("l, start, {}, end, C", x),
        ],
    }
    .into()
}

pub fn putb() -> TuringMachineBuilder {
    put(symbols::S::B)
}
*/

pub fn put1() -> TuringMachineBuilder {
    put(symbols::S::L)
}

pub fn putx() -> TuringMachineBuilder {
    put(symbols::S::X)
}

pub fn right_one() -> TuringMachineBuilder {
    Builder {
        name: "right_one".to_string(),
        code: vec![
            "x, start, x, end, R",
            "-, start, -, end, R",
            "l, start, l, end, R",
        ],
    }
    .into()
}

pub fn left_one() -> TuringMachineBuilder {
    Builder {
        name: "left_one".to_string(),
        code: vec![
            "x, start, x, end, L",
            "-, start, -, end, L",
            "l, start, l, end, L",
        ],
    }
    .into()
}

// ...  ?  x A [x] ...
// ... [?] A s  x ...
// where
// - A: list of symbols in {'-', 'l'}
// - s: given symbol in {'-', 'l', 'x'}
pub fn shift_left_to_right_fill(s: symbols::S) -> TuringMachineBuilder {
    // pseudo code:
    //   let mut put: Symbol = given s
    //   assert!(*head() == x)
    //   LEFT();
    //   while *head() != x {
    //     swap(&mut put, head());
    //     LEFT();
    //   }
    //   LEFT();

    // 愚直にやる。
    Builder {
        name: format!("shift_left_to_right_fill-{}", s),
        code: vec![
            &format!("x, start,  x, put{s}, L"),
            "-,  putb, -, curb, L",
            "l,  putb, -, curl, L",
            "x,  putb, -, curx, L",
            "-,  put1, l, curb, L",
            "l,  put1, l, curl, L",
            "x,  put1, l, curx, L",
            "-,  putx, -, curb, L",
            "l,  putx, -, curl, L",
            "x,  putx, -, curx, L",
            "-,  curx, -, curb, L",
            "l,  curx, -, curl, L",
            "x,  curx, x,  end, L",
        ],
    }
    .into()
}

// ...  x  ? X_1 x ... x X_n [x] ...
// ... [?] X_1 x ... x X_n s  x ...
pub fn shift_left_to_rights(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder =
        TuringMachineBuilder::new(&format!("shift_left_to_rights-{}_{n}", x.print())).unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new({
            vec![
                vec![
                    "x, start, x, put_x, L".to_string(),
                    format!("-, put_x, {},   put-1_b, L", x.print()),
                    format!("l, put_x, {},   put-1_1, L", x.print()),
                    format!("x, put_x, {}, put-2_bar, L", x.print()),
                ],
                (1..n)
                    .flat_map(|i| {
                        vec![
                            format!("-, put-{i}_b, -, put-{i}_b, L"),
                            format!("l, put-{i}_b, -, put-{i}_1, L"),
                            format!("x, put-{i}_b, -, put-{}_bar, L", i + 1),
                            format!("-, put-{i}_1, l, put-{i}_b, L"),
                            format!("l, put-{i}_1, l, put-{i}_1, L"),
                            format!("x, put-{i}_1, l, put-{}_bar, L", i + 1),
                            format!("-, put-{i}_bar, x, put-{i}_b, L"),
                            format!("l, put-{i}_bar, x, put-{i}_1, L"),
                            format!("x, put-{i}_bar, x, put-{}_bar, L", i + 1),
                        ]
                    })
                    .collect(),
                vec![
                    format!("-, put-{n}_b, -, put-{n}_b, L"),
                    format!("l, put-{n}_b, -, put-{n}_1, L"),
                    format!("x, put-{n}_b, -, end, C"),
                    format!("-, put-{n}_1, l, put-{n}_b, L"),
                    format!("l, put-{n}_1, l, put-{n}_1, L"),
                    format!("x, put-{n}_1, l, end, C"),
                    format!("-, put-{n}_bar, x, put-{n}_b, L"),
                    format!("l, put-{n}_bar, x, put-{n}_1, L"),
                    format!("x, put-{n}_bar, x, end, C"),
                ],
            ]
            .into_iter()
            .flatten()
            .map(|str: String| parse_one_code_entry(str.as_ref()).unwrap())
            .collect()
        });
    builder
}

// ... [x] X x  ?  ...
// ...  x  s X [?] ...
pub fn shift_right_to_left_fill(s: symbols::S) -> TuringMachineBuilder {
    Builder {
        name: format!("shift_right_to_left_fill-{}", s),
        code: vec![
            &format!("x, start, x,  putx, R"),
            &format!("-, start, {}, putb, R", s),
            &format!("l, start, {}, put1, R", s),
            &format!("x, start, {},  end, R", s),
            "-,  putb, -, putb, R",
            "l,  putb, -, put1, R",
            "x,  putb, -,  end, R",
            "-,  put1, l, putb, R",
            "l,  put1, l, put1, R",
            "x,  put1, l,  end, R",
        ],
    }
    .into()
}

// [-]X1-...-Xn- を -X1-...-Xn[x] にする
pub fn shift_right_to_lefts(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder =
        TuringMachineBuilder::new(&format!("shift_right_to_lefts-{}_{n}", x.print())).unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new({
            vec![
                vec![
                    "x, start, x, put_x, R".to_string(),
                    format!("-, put_x, {},   put-1_b, R", x.print()),
                    format!("l, put_x, {},   put-1_1, R", x.print()),
                    format!("x, put_x, {}, put-2_bar, R", x.print()),
                ],
                (1..n)
                    .flat_map(|i| {
                        vec![
                            format!("-, put-{i}_b, -, put-{i}_b, R"),
                            format!("l, put-{i}_b, -, put-{i}_1, R"),
                            format!("x, put-{i}_b, -, put-{}_bar, R", i + 1),
                            format!("-, put-{i}_1, l, put-{i}_b, R"),
                            format!("l, put-{i}_1, l, put-{i}_1, R"),
                            format!("x, put-{i}_1, l, put-{}_bar, R", i + 1),
                            format!("-, put-{i}_bar, x, put-{i}_b, R"),
                            format!("l, put-{i}_bar, x, put-{i}_1, R"),
                            format!("x, put-{i}_bar, x, put-{}_bar, R", i + 1),
                        ]
                    })
                    .collect(),
                vec![
                    format!("-, put-{n}_b, -, put-{n}_b, R"),
                    format!("l, put-{n}_b, -, put-{n}_1, R"),
                    format!("x, put-{n}_b, -, end, C"),
                    format!("-, put-{n}_1, l, put-{n}_b, R"),
                    format!("l, put-{n}_1, l, put-{n}_1, R"),
                    format!("x, put-{n}_1, l, end, C"),
                    format!("-, put-{n}_bar, x, put-{n}_b, R"),
                    format!("l, put-{n}_bar, x, put-{n}_1, R"),
                    format!("x, put-{n}_bar, x, end, C"),
                ],
            ]
            .into_iter()
            .flatten()
            .map(|str: String| parse_one_code_entry(str.as_ref()).unwrap())
            .collect()
        });
    builder
}

pub fn annihilate() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "annihilate".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            move_right_till_x(),
            putb(),
            left_one(),
            check_branch(),
            putb(),
            right_one(),
            putx(),
            left_one(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "endl".parse_tc().unwrap()),
            ((3, 4), "endb".parse_tc().unwrap()),
            ((3, 5), "endx".parse_tc().unwrap()),
            ((4, 2), "end".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 7), "end".parse_tc().unwrap()),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(7),
    };
    builder_composition(graph).unwrap()
}

pub fn concat() -> TuringMachineBuilder {
    chain_builders(
        "concat",
        vec![
            move_rights(2),
            shift_left_to_right_fill(symbols::S::X),
            move_rights(2),
            putb(),
            move_lefts(2),
        ],
    )
}

// 名前の通り、先頭要素が 0 表現かどうかを判定する。
pub fn is_tuple_zero() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "is_first_of_tuple_zero".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            right_one(), // 0
            check_branch(),
            left_one(),
            right_one(),
            check_branch(),
            left_one(), //5
            left_one(),
            left_one(), //7
            left_one(),
            left_one(),
            id_end("endF"), // 10
            id_end("endT"), // 11
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "endl".parse_tc().unwrap()),
            ((1, 9), "endx".parse_tc().unwrap()),
            ((1, 3), "endb".parse_tc().unwrap()),
            ((2, 9), "end".parse_tc().unwrap()),
            ((9, 11), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "endl".parse_tc().unwrap()),
            ((4, 7), "endx".parse_tc().unwrap()),
            ((4, 7), "endb".parse_tc().unwrap()),
            ((5, 6), "end".parse_tc().unwrap()),
            ((6, 10), "end".parse_tc().unwrap()),
            ((7, 8), "end".parse_tc().unwrap()),
            ((8, 11), "end".parse_tc().unwrap()),
        ],
        acceptable: vec![
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec!["endF".parse_tc().unwrap()],
            vec!["endT".parse_tc().unwrap()],
        ],
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_safe() {
        let _ = id();
        let _ = id_end("end");
        let _ = move_right_till_x();
        let _ = move_rights(1);
        let _ = move_rights(2);
        let _ = move_left_till_x();
        let _ = check_branch();
        let _ = put1();
        let _ = putb();
        let _ = putx();
        let _ = right_one();
        let _ = left_one();
        let _ = shift_left_to_right_fill(symbols::S::X);
        let _ = shift_right_to_left_fill(symbols::S::X);
        let _ = annihilate();
        let _ = concat();
    }
    #[test]
    fn concat_test() {
        let mut builder = concat();
        let tests = vec![
            (tape_from(&["x", "x", "x"], 0), tape_from(&["x", "x"], 0)),
            (
                tape_from(&["x", "-", "x", "x"], 0),
                tape_from(&["x", "-", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "x", "-", "x"], 0),
                tape_from(&["x", "-", "-", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_left_to_right_fill_test() {
        let mut builder = shift_left_to_right_fill(symbols::S::X);
        let tests = vec![
            (
                tape_from(&["x", "x", "x"], 2),
                tape_from(&["x", "x", "x"], 0),
            ),
            (
                tape_from(&["-", "l", "l", "x", "-", "x"], 5),
                tape_from(&["-", "l", "l", "-", "x", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_left_to_rights_test() {
        let mut builder = shift_left_to_rights(symbols::S::L.into(), 3);
        let tests = vec![
            (
                tape_from(&["x", "x", "x", "x"], 3),
                tape_from(&["x", "x", "l", "x"], 0),
            ),
            (
                tape_from(&["-", "x", "l", "x", "l", "x", "x"], 6),
                tape_from(&["l", "x", "l", "x", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_right_to_lefts_test() {
        let mut builder = shift_right_to_lefts(symbols::S::L.into(), 3);
        let tests = vec![
            (
                tape_from(&["x", "x", "x", "x"], 0),
                tape_from(&["x", "l", "x", "x"], 3),
            ),
            (
                tape_from(&["x", "-", "x", "l", "x", "l", "x"], 0),
                tape_from(&["x", "l", "x", "-", "l", "x", "l"], 6),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn annihilate_test() {
        let mut builder = annihilate();
        let tests = vec![
            (
                tape_from(&["x", "-", "l", "l", "x"], 0),
                tape_from(&["x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "-", "l", "-", "-", "l", "x"], 0),
                tape_from(&["x", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn is_first_test() {
        let mut builder = is_tuple_zero();
        let tests = vec![
            (tape_from(&["x", "x"], 0), "endT".parse_tc().unwrap()),
            (tape_from(&["x", "-", "x"], 0), "endT".parse_tc().unwrap()),
            (
                tape_from(&["x", "-", "l", "l", "-", "l", "x"], 0),
                "endF".parse_tc().unwrap(),
            ),
            (
                tape_from(&["x", "-", "-", "l", "x"], 0),
                "endT".parse_tc().unwrap(),
            ),
        ];
        builder_test_predicate(&mut builder, 100, tests);
    }
}
