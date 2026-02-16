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
    let mut builder = TuringMachineBuilder::new(&format!("id_{str}")).unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec![str.parse_tc().unwrap()])
        .code_new(
            vec![
                format!("x, start, x, {str}, C"),
                format!("-, start, -, {str}, C"),
                format!("l, start, l, {str}, C"),
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str.as_ref()).unwrap())
            .collect(),
        );
    builder
}

pub fn move_right() -> TuringMachineBuilder {
    Builder {
        name: "move_right".to_string(),
        code: vec![
            "x, start, x, till, R",
            "-, start, -, till, R",
            "l, start, l, till, R",
            "-,  till,  -, till, R",
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
        chain_builders(format!("moveR_{n}"), vec![move_right(); n])
    }
}

pub fn move_left() -> TuringMachineBuilder {
    Builder {
        name: "move_left".to_string(),
        code: vec![
            "x, start, x, till, L",
            "-, start, -, till, L",
            "l, start, l, till, L",
            "-,  till,  -, till, L",
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
        chain_builders(format!("moveL_{n}"), vec![move_left(); n])
    }
}

pub fn bor1orbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("bor1").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "-, start,  -, endB, C",
                "l, start, l, end1, C",
                "x, start, x, endbar, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

pub fn putb() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putB").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, -, end, C",
                "-, start, -, end, C",
                "l, start, -, end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

pub fn put1() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("put1").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, l, end, C",
                "-, start, l, end, C",
                "l, start, l, end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

pub fn putbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putbar").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "-, start, x, end, C",
                "l, start, x, end, C",
                "x, start, x, end, C",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

pub fn right_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("rightone").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "-, start, -, end, R",
                "l, start, l, end, R",
                "x, start, x, end, R",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

pub fn left_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("leftone").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "-, start, -, end, L",
                "l, start, l, end, L",
                "x, start, x, end, L",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// y-X[-] を [y]Xx- にする
pub fn shift_left_to_right_fill(x: Sign) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("shift_left_to_right_fill").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, x, putx, L",
                &format!("-, putx, {},  putb, L", x.print()),
                &format!("l, putx, {},  put1, L", x.print()),
                &format!("x, putx, {},   end, L", x.print()),
                "-,  putb, -, putb, L",
                "l,  putb, -, put1, L",
                "x,  putb, -,  end, L",
                "-,  put1, l, putb, L",
                "l,  put1, l, put1, L",
                "x,  put1, l,  end, L",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// -yX1-...-Xn[-] を [y]X1-...-Xnx- にする
pub fn shift_left_to_rights(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder =
        TuringMachineBuilder::new(&format!("shift_left_to_rights^{}_{n}", x.print())).unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new({
            vec![
                vec![
                    "x, start, x, put_x, L".to_string(),
                    format!("-, put_x, {},   put^1_b, L", x.print()),
                    format!("l, put_x, {},   put^1_1, L", x.print()),
                    format!("x, put_x, {}, put^2_bar, L", x.print()),
                ],
                (1..n)
                    .flat_map(|i| {
                        vec![
                            format!("-, put^{i}_b, -, put^{i}_b, L"),
                            format!("l, put^{i}_b, -, put^{i}_1, L"),
                            format!("x, put^{i}_b, -, put^{}_bar, L", i + 1),
                            format!("-, put^{i}_1, l, put^{i}_b, L"),
                            format!("l, put^{i}_1, l, put^{i}_1, L"),
                            format!("x, put^{i}_1, l, put^{}_bar, L", i + 1),
                            format!("-, put^{i}_bar, x, put^{i}_b, L"),
                            format!("l, put^{i}_bar, x, put^{i}_1, L"),
                            format!("x, put^{i}_bar, x, put^{}_bar, L", i + 1),
                        ]
                    })
                    .collect(),
                vec![
                    format!("-, put^{n}_b, -, put^{n}_b, L"),
                    format!("l, put^{n}_b, -, put^{n}_1, L"),
                    format!("x, put^{n}_b, -, end, C"),
                    format!("-, put^{n}_1, l, put^{n}_b, L"),
                    format!("l, put^{n}_1, l, put^{n}_1, L"),
                    format!("x, put^{n}_1, l, end, C"),
                    format!("-, put^{n}_bar, x, put^{n}_b, L"),
                    format!("l, put^{n}_bar, x, put^{n}_1, L"),
                    format!("x, put^{n}_bar, x, end, C"),
                ],
            ]
            .into_iter()
            .flatten()
            .map(|str: String| parse_one_code_entry(str.as_ref()).unwrap())
            .collect()
        });
    builder
}

// [-]X-y を -xX[y] にする
pub fn shift_right_to_left_fill(x: Sign) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("shift_right_to_left_fill").unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new(
            vec![
                "x, start, x, putx, R",
                &format!("-, putx, {},  putb, R", x.print()),
                &format!("l, putx, {},  put1, R", x.print()),
                &format!("x, putx, {},   end, R", x.print()),
                "-,  putb, -, putb, R",
                "l,  putb, -, put1, R",
                "x,  putb, -,  end, R",
                "-,  put1, l, putb, R",
                "l,  put1, l, put1, R",
                "x,  put1, l,  end, R",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// [-]X1-...-Xn- を -X1-...-Xn[x] にする
pub fn shift_right_to_lefts(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder =
        TuringMachineBuilder::new(&format!("shift_right_to_lefts^{}_{n}", x.print())).unwrap();
    builder
        .init_state("start".parse_tc().unwrap())
        .accepted_state(vec!["end".parse_tc().unwrap()])
        .code_new({
            vec![
                vec![
                    "x, start, x, put_x, R".to_string(),
                    format!("-, put_x, {},   put^1_b, R", x.print()),
                    format!("l, put_x, {},   put^1_1, R", x.print()),
                    format!("x, put_x, {}, put^2_bar, R", x.print()),
                ],
                (1..n)
                    .flat_map(|i| {
                        vec![
                            format!("-, put^{i}_b, -, put^{i}_b, R"),
                            format!("l, put^{i}_b, -, put^{i}_1, R"),
                            format!("x, put^{i}_b, -, put^{}_bar, R", i + 1),
                            format!("-, put^{i}_1, l, put^{i}_b, R"),
                            format!("l, put^{i}_1, l, put^{i}_1, R"),
                            format!("x, put^{i}_1, l, put^{}_bar, R", i + 1),
                            format!("-, put^{i}_bar, x, put^{i}_b, R"),
                            format!("l, put^{i}_bar, x, put^{i}_1, R"),
                            format!("x, put^{i}_bar, x, put^{}_bar, R", i + 1),
                        ]
                    })
                    .collect(),
                vec![
                    format!("-, put^{n}_b, -, put^{n}_b, R"),
                    format!("l, put^{n}_b, -, put^{n}_1, R"),
                    format!("x, put^{n}_b, -, end, C"),
                    format!("-, put^{n}_1, l, put^{n}_b, R"),
                    format!("l, put^{n}_1, l, put^{n}_1, R"),
                    format!("x, put^{n}_1, l, end, C"),
                    format!("-, put^{n}_bar, x, put^{n}_b, R"),
                    format!("l, put^{n}_bar, x, put^{n}_1, R"),
                    format!("x, put^{n}_bar, x, end, C"),
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
            move_right(),
            putb(),
            left_one(),
            bor1orbar(),
            putb(),
            right_one(),
            putbar(),
            left_one(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), "end".parse_tc().unwrap()),
            ((1, 2), "end".parse_tc().unwrap()),
            ((2, 3), "end".parse_tc().unwrap()),
            ((3, 4), "end1".parse_tc().unwrap()),
            ((3, 4), "endB".parse_tc().unwrap()),
            ((3, 5), "endbar".parse_tc().unwrap()),
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
            shift_left_to_right_fill(symbols::partition_sign()),
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
            bor1orbar(),
            left_one(),
            right_one(),
            bor1orbar(),
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
            ((1, 2), "end1".parse_tc().unwrap()),
            ((1, 9), "endbar".parse_tc().unwrap()),
            ((1, 3), "endB".parse_tc().unwrap()),
            ((2, 9), "end".parse_tc().unwrap()),
            ((9, 11), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "end1".parse_tc().unwrap()),
            ((4, 7), "endbar".parse_tc().unwrap()),
            ((4, 7), "endB".parse_tc().unwrap()),
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
        let _ = move_right();
        let _ = move_rights(1);
        let _ = move_rights(2);
        let _ = move_left();
        let _ = bor1orbar();
        let _ = put1();
        let _ = putb();
        let _ = putbar();
        let _ = right_one();
        let _ = left_one();
        let _ = shift_left_to_right_fill(symbols::partition_sign());
        let _ = shift_right_to_left_fill(symbols::partition_sign());
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
        let mut builder = shift_left_to_right_fill(symbols::partition_sign());
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
        let mut builder = shift_left_to_rights(symbols::one_sign(), 3);
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
        let mut builder = shift_right_to_lefts(symbols::one_sign(), 3);
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
