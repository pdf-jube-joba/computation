use turing_machine::{
    machine::*,
    manipulation::{
        builder::TuringMachineBuilder,
        code::parse_one_code_entry,
        graph_compose::{builder_composition, GraphOfBuilder},
    },
};

use crate::*;

pub fn id() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("id").unwrap();
    builder.from_source(include_str!("id.txt")).unwrap();
    builder
}

pub fn id_end(str: &str) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("id_{str}")).unwrap();
    builder
        .init_state("start".parse().unwrap())
        .accepted_state(vec![str.parse().unwrap()])
        .code_new(
            vec![
                format!("-, start, -, {str}, C"),
                format!(" , start,  , {str}, C"),
                format!("1, start, 1, {str}, C"),
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str.as_ref()).unwrap())
            .collect(),
        );
    builder
}

pub fn move_right() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_right").unwrap();
    builder.from_source(include_str!("move_right.txt")).unwrap();
    builder
}

pub fn move_rights(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: format!("moveR_{n}"),
            init_state: "start".parse().unwrap(),
            assign_vertex_to_builder: vec![move_right(); n],
            assign_edge_to_state: series_edge_end_only(n - 1),
            acceptable: accept_end_only(n - 1),
        };
        builder_composition(graph).unwrap()
    }
}

pub fn move_left() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("move_left").unwrap();
    builder.from_source(include_str!("move_left.txt")).unwrap();
    builder
}

pub fn move_lefts(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        id()
    } else {
        let graph = GraphOfBuilder {
            name: format!("moveL_{n}"),
            init_state: "start".parse().unwrap(),
            assign_vertex_to_builder: vec![move_left(); n],
            assign_edge_to_state: series_edge_end_only(n - 1),
            acceptable: accept_end_only(n - 1),
        };
        builder_composition(graph).unwrap()
    }
}

pub fn bor1orbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("bor1").unwrap();
    builder.from_source(include_str!("bor1orbar.txt")).unwrap();
    builder
}

pub fn putb() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putB").unwrap();
    builder
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                "-, start,  , end, C",
                " , start,  , end, C",
                "1, start,  , end, C",
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
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                "-, start, 1, end, C",
                " , start, 1, end, C",
                "1, start, 1, end, C",
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
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                " , start, -, end, C",
                "1, start, -, end, C",
                "-, start, -, end, C",
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
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                " , start,  , end, R",
                "1, start, 1, end, R",
                "-, start, -, end, R",
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
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                " , start,  , end, L",
                "1, start, 1, end, L",
                "-, start, -, end, L",
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
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                "-, start, -, putx, L",
                &format!(" , putx, {x},  putb, L"),
                &format!("1, putx, {x},  put1, L"),
                &format!("-, putx, {x},   end, L"),
                " ,  putb,  , putb, L",
                "1,  putb,  , put1, L",
                "-,  putb,  ,  end, L",
                " ,  put1, 1, putb, L",
                "1,  put1, 1, put1, L",
                "-,  put1, 1,  end, L",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// -yX1-...-Xn[-] を [y]X1-...-Xnx- にする
pub fn shift_left_to_rights(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("shift_left_to_rights^{x}_{n}")).unwrap();
    builder
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new({
            vec![
                vec![
                    "-, start, -, put_x, L".to_string(),
                    format!(" , put_x, {x},   put^1_b, L"),
                    format!("1, put_x, {x},   put^1_1, L"),
                    format!("-, put_x, {x}, put^2_bar, L"),
                ],
                (1..n)
                    .flat_map(|i| {
                        vec![
                            format!(" , put^{i}_b,  , put^{i}_b, L"),
                            format!("1, put^{i}_b,  , put^{i}_1, L"),
                            format!("-, put^{i}_b,  , put^{}_bar, L", i + 1),
                            format!(" , put^{i}_1, 1, put^{i}_b, L"),
                            format!("1, put^{i}_1, 1, put^{i}_1, L"),
                            format!("-, put^{i}_1, 1, put^{}_bar, L", i + 1),
                            format!(" , put^{i}_bar, -, put^{i}_b, L"),
                            format!("1, put^{i}_bar, -, put^{i}_1, L"),
                            format!("-, put^{i}_bar, -, put^{}_bar, L", i + 1),
                        ]
                    })
                    .collect(),
                vec![
                    format!(" , put^{n}_b,  , put^{n}_b, L"),
                    format!("1, put^{n}_b,  , put^{n}_1, L"),
                    format!("-, put^{n}_b,  , end, C"),
                    format!(" , put^{n}_1, 1, put^{n}_b, L"),
                    format!("1, put^{n}_1, 1, put^{n}_1, L"),
                    format!("-, put^{n}_1, 1, end, C"),
                    format!(" , put^{n}_bar, -, put^{n}_b, L"),
                    format!("1, put^{n}_bar, -, put^{n}_1, L"),
                    format!("-, put^{n}_bar, -, end, C"),
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
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new(
            vec![
                "-, start, -, putx, R",
                &format!(" , putx, {x},  putb, R"),
                &format!("1, putx, {x},  put1, R"),
                &format!("-, putx, {x},   end, R"),
                " ,  putb,  , putb, R",
                "1,  putb,  , put1, R",
                "-,  putb,  ,  end, R",
                " ,  put1, 1, putb, R",
                "1,  put1, 1, put1, R",
                "-,  put1, 1,  end, R",
            ]
            .into_iter()
            .map(|str| parse_one_code_entry(str).unwrap())
            .collect(),
        );
    builder
}

// [-]X1-...-Xn- を -X1-...-Xn[x] にする
pub fn shift_right_to_lefts(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("shift_right_to_lefts^{x}_{n}")).unwrap();
    builder
        .init_state("start".parse().unwrap())
        .accepted_state(vec!["end".parse().unwrap()])
        .code_new({
            vec![
                vec![
                    "-, start, -, put_x, R".to_string(),
                    format!(" , put_x, {x},   put^1_b, R"),
                    format!("1, put_x, {x},   put^1_1, R"),
                    format!("-, put_x, {x}, put^2_bar, R"),
                ],
                (1..n)
                    .flat_map(|i| {
                        vec![
                            format!(" , put^{i}_b,  , put^{i}_b, R"),
                            format!("1, put^{i}_b,  , put^{i}_1, R"),
                            format!("-, put^{i}_b,  , put^{}_bar, R", i + 1),
                            format!(" , put^{i}_1, 1, put^{i}_b, R"),
                            format!("1, put^{i}_1, 1, put^{i}_1, R"),
                            format!("-, put^{i}_1, 1, put^{}_bar, R", i + 1),
                            format!(" , put^{i}_bar, -, put^{i}_b, R"),
                            format!("1, put^{i}_bar, -, put^{i}_1, R"),
                            format!("-, put^{i}_bar, -, put^{}_bar, R", i + 1),
                        ]
                    })
                    .collect(),
                vec![
                    format!(" , put^{n}_b,  , put^{n}_b, R"),
                    format!("1, put^{n}_b,  , put^{n}_1, R"),
                    format!("-, put^{n}_b,  , end, C"),
                    format!(" , put^{n}_1, 1, put^{n}_b, R"),
                    format!("1, put^{n}_1, 1, put^{n}_1, R"),
                    format!("-, put^{n}_1, 1, end, C"),
                    format!(" , put^{n}_bar, -, put^{n}_b, R"),
                    format!("1, put^{n}_bar, -, put^{n}_1, R"),
                    format!("-, put^{n}_bar, -, end, C"),
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
        init_state: "start".parse().unwrap(),
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
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end".parse().unwrap()),
            ((2, 3), "end".parse().unwrap()),
            ((3, 4), "end1".parse().unwrap()),
            ((3, 4), "endB".parse().unwrap()),
            ((3, 5), "endbar".parse().unwrap()),
            ((4, 2), "end".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 7), "end".parse().unwrap()),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(7),
    };
    builder_composition(graph).unwrap()
}

pub fn concat() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "concat".to_string(),
        init_state: "start".parse().unwrap(),
        assign_vertex_to_builder: vec![
            move_rights(2),
            shift_left_to_right_fill("-".parse().unwrap()),
            move_rights(2),
            putb(),
            move_lefts(2), // 16
        ],
        assign_edge_to_state: series_edge_end_only(4),
        acceptable: accept_end_only(4),
    };
    builder_composition(graph).unwrap()
}

// 名前の通り -b0p- や -- の形になっているか、 つまり -- や -bb... や -b-... になっているかを判定する。
// ということは -b1...でなければよい？
pub fn is_tuple_zero() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "is_first_of_tuple_zero".to_string(),
        init_state: "start".parse().unwrap(),
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
            ((0, 1), "end".parse().unwrap()),
            ((1, 2), "end1".parse().unwrap()),
            ((1, 9), "endbar".parse().unwrap()),
            ((1, 3), "endB".parse().unwrap()),
            ((2, 9), "end".parse().unwrap()),
            ((9, 11), "end".parse().unwrap()),
            ((3, 4), "end".parse().unwrap()),
            ((4, 5), "end1".parse().unwrap()),
            ((4, 7), "endbar".parse().unwrap()),
            ((4, 7), "endB".parse().unwrap()),
            ((5, 6), "end".parse().unwrap()),
            ((6, 10), "end".parse().unwrap()),
            ((7, 8), "end".parse().unwrap()),
            ((8, 11), "end".parse().unwrap()),
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
            vec!["endF".parse().unwrap()],
            vec!["endT".parse().unwrap()],
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
        let _ = shift_left_to_right_fill("-".parse().unwrap());
        let _ = shift_right_to_left_fill("-".parse().unwrap());
        let _ = annihilate();
        let _ = concat();
    }
    #[test]
    fn concat_test() {
        let mut builder = concat();
        let tests = vec![
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-", "-"]),
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "-", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "-"]), 0),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "-", "-"]),
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "-", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "", "-"]), 0),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "-", "", "-"]),
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "-", "", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "", "", "-"]), 0),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "1", "-", "", "1", "-"]),
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "1", "", "1", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "1", "-", "", "1", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "", "1", "", "1", "-"]), 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_left_to_right_fill_test() {
        let mut builder = shift_left_to_right_fill("-".parse().unwrap());
        let tests = vec![
            // `[-][ ]{-}` -> `[ ]{-}[-]`
            // (
            //     // Tape {
            //     //     left: vec_sign(vec!["-", ""]),
            //     //     head: "-".parse().unwrap(),
            //     //     right: vec![],
            //     // },
            //     // Tape {
            //     //     left: vec![],
            //     //     head: Sign::blank(),
            //     //     right: vec_sign(vec!["-", "-"]),
            //     // },
            //     Tape::from_vec(vec_sign(vec!["-", "", "-"]), 2),
            //     Tape::from_vec(vec_sign(vec!["", "-", "-"]), 0),
            // ),
            (
                // Tape {
                //     left: vec_sign(vec!["-", "-"]),
                //     head: "-".parse().unwrap(),
                //     right: vec![],
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "-", "-"]), 2),
                Tape::from_vec(vec_sign(vec!["-", "-", "-"]), 0),
            ),
            (
                // Tape {
                //     left: vec_sign(vec!["", "1", "1", "-", ""]),
                //     head: "-".parse().unwrap(),
                //     right: vec![],
                // },
                // Tape {
                //     left: vec![],
                //     head: Sign::blank(),
                //     right: vec_sign(vec!["1", "1", "", "-", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["", "1", "1", "-", "", "-"]), 5),
                Tape::from_vec(vec_sign(vec!["", "1", "1", "", "-", "-"]), 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_left_to_rights_test() {
        let mut builder = shift_left_to_rights("1".parse().unwrap(), 3);
        let tests = vec![
            (
                // Tape {
                //     left: vec_sign(vec!["-", "-", "-"]),
                //     head: "-".parse().unwrap(),
                //     right: vec![],
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-", "1", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "-", "-", "-"]), 3),
                Tape::from_vec(vec_sign(vec!["-", "-", "1", "-"]), 0),
            ),
            (
                // Tape {
                //     left: vec_sign(vec!["", "-", "1", "-", "1", "-"]),
                //     head: "-".parse().unwrap(),
                //     right: vec![],
                // },
                // Tape {
                //     left: vec![],
                //     head: "1".parse().unwrap(),
                //     right: vec_sign(vec!["-", "1", "-", "", "1", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["", "-", "1", "-", "1", "-", "-"]), 6),
                Tape::from_vec(vec_sign(vec!["1", "-", "1", "-", "", "1", "-"]), 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_right_to_lefts_test() {
        let mut builder = shift_right_to_lefts("1".parse().unwrap(), 3);
        let tests = vec![
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-", "-", "-"]),
                // },
                // Tape {
                //     left: vec_sign(vec!["-", "1", "-"]),
                //     head: "-".parse().unwrap(),
                //     right: vec![],
                // },
                Tape::from_vec(vec_sign(vec!["-", "-", "-", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "1", "-", "-"]), 3),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "-", "1", "-", "1", "-"]),
                // },
                // Tape {
                //     left: vec_sign(vec!["-", "1", "-", "", "1", "-"]),
                //     head: "1".parse().unwrap(),
                //     right: vec![],
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "-", "1", "-", "1", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "1", "-", "", "1", "-", "1"]), 6),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn annihilate_test() {
        let mut builder = annihilate();
        let tests = vec![
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "1", "1", "-"]),
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "1", "1", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "-"]), 0),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "1", "", "1", "", "", "1", "-"]),
                // },
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "1", "", "1", "", "", "1", "-"]), 0),
                Tape::from_vec(vec_sign(vec!["-", "-"]), 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn is_first_test() {
        let mut builder = is_tuple_zero();
        let tests = vec![
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "-"]), 0),
                "endT".parse().unwrap(),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "-"]), 0),
                "endT".parse().unwrap(),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "1", "1", "", "1", "-"]), 0),
                "endF".parse().unwrap(),
            ),
            (
                // Tape {
                //     left: vec![],
                //     head: "-".parse().unwrap(),
                //     right: vec_sign(vec!["", "", "1", "-"]),
                // },
                Tape::from_vec(vec_sign(vec!["-", "", "", "1", "-"]), 0),
                "endT".parse().unwrap(),
            ),
        ];
        builder_test_predicate(&mut builder, 100, tests);
    }
}
