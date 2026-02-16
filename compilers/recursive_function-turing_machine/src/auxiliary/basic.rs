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

//  ... [?_1] ?_2 ... ?_n ...
//  ... ?_1 ?_2 ... [?_n] ...
//  where ?_i in {'-', 'l'} for i < n, ?_n = 'x'
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

//  ... ?_1 ?_2 ... [?_n] ...
//  ... [?_1] ?_2 ... ?_n ...
//  where ?_i in {'-', 'l'} for 1 < i, ?_1 = 'x'
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

//  ... [?] ... ->
pub fn check_current() -> TuringMachineBuilder {
    Builder {
        name: "check_current".to_string(),
        code: vec![
            "-, start, -, end-, C",
            "l, start, l, endl, C",
            "x, start, x, endx, C",
        ],
    }
    .into()
}

// b for blank
pub fn putb() -> TuringMachineBuilder {
    Builder {
        name: "putB".to_string(),
        code: vec![
            "x, start, -, end, C",
            "-, start, -, end, C",
            "l, start, -, end, C",
        ],
    }
    .into()
}

pub fn putl() -> TuringMachineBuilder {
    Builder {
        name: "putl".to_string(),
        code: vec![
            "x, start, l, end, C",
            "-, start, l, end, C",
            "l, start, l, end, C",
        ],
    }
    .into()
}

pub fn putx() -> TuringMachineBuilder {
    Builder {
        name: "putbar".to_string(),
        code: vec![
            "-, start, x, end, C",
            "l, start, x, end, C",
            "x, start, x, end, C",
        ],
    }
    .into()
}

pub fn right_one() -> TuringMachineBuilder {
    Builder {
        name: "rightone".to_string(),
        code: vec![
            "-, start, -, end, R",
            "l, start, l, end, R",
            "x, start, x, end, R",
        ],
    }
    .into()
}

pub fn left_one() -> TuringMachineBuilder {
    Builder {
        name: "leftone".to_string(),
        code: vec![
            "-, start, -, end, L",
            "l, start, l, end, L",
            "x, start, x, end, L",
        ],
    }
    .into()
}

// ... ? x A [x] ...
// ... [?] A x a ...
// A: list of {'-', 'l'}, may empty
pub fn shift_l2r_fill(a: Sign) -> TuringMachineBuilder {
    let start = &format!("x, start, {}, putx, L", a.print());

    Builder {
        name: "shift_l2r_fill".to_string(),
        code: vec![
            start,
            "-,  putx, x, putb, L",
            "l,  putx, x, putl, L",
            "-,  putb, -, putb, L",
            "l,  putb, -, putl, L",
            "-,  putl, l, putb, L",
            "l,  putl, l, putl, L",
            "x,  putx, x,  end, L",
            "x,  putb, -,  end, L",
            "x,  putl, l,  end, L",
        ],
    }
    .into()
}

// ... [x] - A_1 - ... - A_n x - B_1 - ... - B_m x ...
// ... [x] - A_1 - ... - A_n - B_1 - ... - B_m x ...
pub fn concat() -> TuringMachineBuilder {
    chain_builders(
        "concat",
        vec![
            move_rights(2),
            shift_l2r_fill(symbols::partition_sign()),
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
            check_current(),
            left_one(),
            right_one(),
            check_current(),
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
            ((1, 3), "end-".parse_tc().unwrap()),
            ((2, 9), "end".parse_tc().unwrap()),
            ((9, 11), "end".parse_tc().unwrap()),
            ((3, 4), "end".parse_tc().unwrap()),
            ((4, 5), "endl".parse_tc().unwrap()),
            ((4, 7), "endx".parse_tc().unwrap()),
            ((4, 7), "end-".parse_tc().unwrap()),
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
        let _ = check_current();
        let _ = putl();
        let _ = putb();
        let _ = putx();
        let _ = right_one();
        let _ = left_one();
        let _ = shift_l2r_fill(symbols::partition_sign());
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
        let mut builder = shift_l2r_fill(symbols::partition_sign());
        let tests = vec![
            (
                tape_from(&["x", "x", "x"], 2),
                tape_from(&["x", "x", "x"], 0),
            ),
            (
                tape_from(&["-", "l", "l", "x", "-", "x"], 5),
                tape_from(&["-", "l", "l", "-", "x", "x"], 2),
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
