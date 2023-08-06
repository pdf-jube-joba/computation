use turing_machine::{
    machine::*,
    manipulation::{
        builder::TuringMachineBuilder,
        graph_compose::{naive_builder_composition, GraphOfBuilder},
    },
};

use super::*;

pub fn id() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("id").unwrap();
    builder.from_source(include_str!("id.txt")).unwrap();
    builder
}

pub fn id_end(str: &str) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("id_{str}")).unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state(str)])
        .code_new(
            vec![
                format!("-, start, -, {str}, C"),
                format!(" , start,  , {str}, C"),
                format!("1, start, 1, {str}, C"),
            ]
            .into_iter()
            .map(|str| CodeEntry::try_from(str.as_ref()).unwrap())
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
            init_state: state("start"),
            assign_vertex_to_builder: vec![move_right(); n],
            assign_edge_to_state: series_edge_end_only(n - 1),
            acceptable: accept_end_only(n - 1),
        };
        naive_builder_composition(graph).unwrap()
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
            init_state: state("start"),
            assign_vertex_to_builder: vec![move_left(); n],
            assign_edge_to_state: series_edge_end_only(n - 1),
            acceptable: accept_end_only(n - 1),
        };
        naive_builder_composition(graph).unwrap()
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
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start,  , end, C",
                " , start,  , end, C",
                "1, start,  , end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn put1() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("put1").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                "-, start, 1, end, C",
                " , start, 1, end, C",
                "1, start, 1, end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn putbar() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("putbar").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start, -, end, C",
                "1, start, -, end, C",
                "-, start, -, end, C",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn right_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("rightone").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start,  , end, R",
                "1, start, 1, end, R",
                "-, start, -, end, R",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

pub fn left_one() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("leftone").unwrap();
    builder
        .init_state(state("start"))
        .accepted_state(vec![state("end")])
        .code_new(
            vec![
                " , start,  , end, L",
                "1, start, 1, end, L",
                "-, start, -, end, L",
            ]
            .into_iter()
            .map(|str| str.try_into().unwrap())
            .collect(),
        );
    builder
}

// y-X[-] を [y]Xx- にする
pub fn shift_left_to_right_fill(x: Sign) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("shift_left_to_right_fill")).unwrap();
    builder.init_state(state("start")).accepted_state(vec![state("end")])
        .code_new(vec![
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
        .map(|str| str.try_into().unwrap())
        .collect(),
        );
    builder
}

// -xX1-...-Xn[-] を [x]X1-...-Xnx- にする
pub fn shift_left_to_rights(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("shift_left_to_rights^{x}_{n}")).unwrap();
    builder.init_state(state("start")).accepted_state(vec![state("end")])
        .code_new({
            vec![
                vec![
                    "-, start, -, put_x, L".to_string(),
                    format!(" , put_x, {x},   put^1_b, L"),
                    format!("1, put_x, {x},   put^1_1, L"),
                    format!("-, put_x, {x}, put^2_bar, L"),
                ],
                (1..n).map(|i| vec![
                    format!(" , put^{i}_b,  , put^{i}_b, L"),
                    format!("1, put^{i}_b,  , put^{i}_1, L"),
                    format!("-, put^{i}_b,  , put^{}_bar, L", i+1),
                    format!(" , put^{i}_1, 1, put^{i}_b, L"),
                    format!("1, put^{i}_1, 1, put^{i}_1, L"),
                    format!("-, put^{i}_1, 1, put^{}_bar, L", i+1),
                    format!(" , put^{i}_bar, -, put^{i}_b, L"),
                    format!("1, put^{i}_bar, -, put^{i}_1, L"),
                    format!("-, put^{i}_bar, -, put^{}_bar, L", i+1),
                ]).flatten().collect(),
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
                ]
            ].into_iter()
        .flatten()
        .map(|str: String| CodeEntry::try_from(str.as_ref()).unwrap())
        .collect()
        });
    builder
}

// [-]X-y を -xX[y] にする
pub fn shift_right_to_left_fill(x: Sign) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("shift_left_to_right_fill")).unwrap();
    builder.init_state(state("start")).accepted_state(vec![state("end")])
        .code_new(vec![
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
        .map(|str| str.try_into().unwrap())
        .collect(),
        );
    builder
}

// [-]X1-...-Xn- を -X1-...-Xn[x] にする
pub fn shift_right_to_lefts(x: Sign, n: usize) -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new(&format!("shift_right_to_lefts^{x}_{n}")).unwrap();
    builder.init_state(state("start")).accepted_state(vec![state("end")])
        .code_new({
            vec![
                vec![
                    "-, start, -, put_x, R".to_string(),
                    format!(" , put_x, {x},   put^1_b, R"),
                    format!("1, put_x, {x},   put^1_1, R"),
                    format!("-, put_x, {x}, put^2_bar, R"),
                ],
                (1..n).map(|i| vec![
                    format!(" , put^{i}_b,  , put^{i}_b, R"),
                    format!("1, put^{i}_b,  , put^{i}_1, R"),
                    format!("-, put^{i}_b,  , put^{}_bar, R", i+1),
                    format!(" , put^{i}_1, 1, put^{i}_b, R"),
                    format!("1, put^{i}_1, 1, put^{i}_1, R"),
                    format!("-, put^{i}_1, 1, put^{}_bar, R", i+1),
                    format!(" , put^{i}_bar, -, put^{i}_b, R"),
                    format!("1, put^{i}_bar, -, put^{i}_1, R"),
                    format!("-, put^{i}_bar, -, put^{}_bar, R", i+1),
                ]).flatten().collect(),
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
                ]
            ].into_iter()
        .flatten()
        .map(|str: String| CodeEntry::try_from(str.as_ref()).unwrap())
        .collect()
        });
    builder
}

pub fn annihilate() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "annihilate".to_string(),
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end1")),
            ((3, 4), state("endB")),
            ((3, 5), state("endbar")),
            ((4, 2), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("end")),
        ]
        .into_iter()
        .collect(),
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn concat() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "concat".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_rights(2),
            putb(),
            left_one(),
            bor1orbar(), // 3
            putbar(), // 4
            left_one(),
            bor1orbar(),
            put1(),
            put1(),
            put1(),
            putbar(), // 10
            left_one(), // 11
            bor1orbar(),
            putb(),
            putb(),
            putb(), 
            move_left(), // 16
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end1")),
            ((3, 11), state("endB")),
            ((3, 16), state("endbar")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("end1")),
            ((6, 8), state("endB")),
            ((6, 9), state("endbar")),
            ((7, 5), state("end")),
            ((8, 11), state("end")),
            ((9, 16), state("end")),
            ((10, 11), state("end")),
            ((11, 12), state("end")),
            ((12, 13), state("end1")),
            ((12, 14), state("endB")),
            ((12, 15), state("endbar")),
            ((13, 5), state("end")),
            ((14, 11), state("end")),
            ((15, 16), state("end")),
        ],
        acceptable: accept_end_only(16),
    };
    naive_builder_composition(graph).unwrap()
}

// 名前の通り -b0p- や -- の形になっているか、 つまり -- や -bb... や -b-... になっているかを判定する。
// ということは -b1...でなければよい？
pub fn is_tuple_zero() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "is_first_of_tuple_zero".to_string(),
        init_state: state("start"),
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
            ((0, 1), state("end")),
            ((1, 2), state("end1")),
            ((1, 9), state("endbar")),
            ((1, 3), state("endB")),

            ((2, 9), state("end")),
            ((9,11), state("end")),

            ((3, 4), state("end")),
            ((4, 5), state("end1")),
            ((4, 7), state("endbar")),
            ((4, 7), state("endB")),
            ((5, 6), state("end")),
            ((6, 10), state("end")),
            ((7, 8), state("end")),
            ((8, 11), state("end")),
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
            vec![state("endF")],
            vec![state("endT")],
        ],
    };
    naive_builder_composition(graph).unwrap()
}

pub mod copy;
pub mod rotate;

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
        let _ = shift_left_to_right_fill(sign("-"));
        let _ = shift_right_to_left_fill(sign("-"));
        let _ = annihilate();
        let _ = concat();
    }
    #[test]
    fn concat_test() {
        let mut builder = concat();
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
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "-", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_left_to_right_fill_test() {
        let mut builder = shift_left_to_right_fill(sign("-"));
        let tests = vec![
            (
                TapeAsVec {
                    left: vec_sign(vec!["-", ""]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec!["-", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["-", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "1", "1", "-", ""]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec![],
                    head: sign(""),
                    right: vec_sign(vec!["1", "1", "", "-", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_left_to_rights_test() {
        let mut builder = shift_left_to_rights(sign("1"), 3);
        let tests = vec![
            (
                TapeAsVec {
                    left: vec_sign(vec!["-", "-", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "1", "-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec_sign(vec!["", "-", "1", "-", "1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("1"),
                    right: vec_sign(vec!["-", "1", "-", "", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn shift_right_to_lefts_test() {
        let mut builder = shift_right_to_lefts(sign("1"), 3);
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "-", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["-", "1", "-"]),
                    head: sign("-"),
                    right: vec![],
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-", "1", "-", "1", "-"]),
                },
                TapeAsVec {
                    left: vec_sign(vec!["-", "1", "-", "", "1", "-"]),
                    head: sign("1"),
                    right: vec![],
                },
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn annihilate_test() {
        let mut builder = annihilate();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "", "1", "", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
    #[test]
    fn is_first_test() {
        let mut builder = is_tuple_zero();
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-"]),
                },
                state("endT")
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "-"]),
                },
                state("endT")
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "1", "", "1", "-"]),
                },
                state("endF")
            ),
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "", "1", "-"]),
                },
                state("endT")
            ),
        ];
        builder_test_predicate(&mut builder, 100, tests);
    }
}
