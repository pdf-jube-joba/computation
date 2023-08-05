use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{naive_builder_composition, GraphOfBuilder},
};

use super::*;

// 名前の通り -bp- の形になっているか == -bb...になっているかを判定する。
fn is_first_of_tuple_zero() -> TuringMachineBuilder {
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
            id_end("endT"),
            id_end("endF"),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end1")),
            ((1, 2), state("endbar")),
            ((2, 9), state("end")),
            ((1, 3), state("endb")),
            ((3, 4), state("end")),
            ((4, 5), state("endbar")),
            ((4, 5), state("end1")),
            ((4, 7), state("endb")),
            ((5, 6), state("end")),
            ((6, 9), state("end")),
            ((7, 8), state("end")),
            ((8, 10), state("end")),
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
            vec![state("endF")],
            vec![state("endT")],
        ],
    };
    naive_builder_composition(graph).unwrap()
}

// -1-はタプルとしては現れないのでそれをシグネチャとし、判定する
// -1-が左にあると T そうじゃないと F を返す
fn is_left_sig() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "is_left_sig".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            left_one(), // 0
            bor1orbar(),
            right_one(),
            left_one(),
            bor1orbar(),
            right_one(), //5
            right_one(),
            right_one(), //7
            right_one(),
            id_end("endF"),
            id_end("endT"),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("endB")),
            ((1, 2), state("endbar")),
            ((2, 9), state("end")),
            ((1, 3), state("end1")),
            ((3, 4), state("end")),
            ((4, 5), state("endb")),
            ((4, 5), state("end1")),
            ((4, 7), state("endbar")),
            ((5, 6), state("end")),
            ((6, 9), state("end")),
            ((7, 8), state("end")),
            ((8, 10), state("end")),
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
            vec![state("endF")],
            vec![state("endT")],
        ],
    };
    naive_builder_composition(graph).unwrap()
}

// -b(x)p- を -b(x-1)p- にする
// -- や -bp- はエラー
fn expand_aux_shrink() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shift_R".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_right(),
            putb(),
            left_one(),
            bor1orbar(),
            putbar(), // 4
            left_one(),
            bor1orbar(),
            putb(),
            putb(),
            putb(),
            putbar(), // 10
            left_one(),
            bor1orbar(),
            put1(),
            put1(),
            put1(),
            right_one(), // 16
            putb(),
            left_one(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("endB")),
            ((3, 10), state("end1")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("endb")),
            ((6, 8), state("end1")),
            ((6, 9), state("endbar")),
            ((7, 5), state("end")),
            ((8, 11), state("end")),
            ((9, 16), state("end")),
            ((10, 11), state("end")),
            ((11, 12), state("end")),
            ((12, 13), state("endb")),
            ((12, 14), state("end1")),
            ((12, 15), state("endbar")),
            ((13, 5), state("end")),
            ((14, 9), state("end")),
            ((15, 16), state("end")),
            ((16, 17), state("end")),
            ((17, 18), state("end")),
        ],
        acceptable: accept_end_only(17),
    };
    naive_builder_composition(graph).unwrap()
}

fn expand_aux_shift_right() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "shift_R".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            right_one(),
            bor1orbar(),
            putbar(),
            right_one(),
            bor1orbar(),
            putb(),
            putb(),
            putb(),
            putbar(),
            right_one(),
            bor1orbar(),
            put1(),
            put1(),
            put1(),
            putbar(),
            right_one(),
            putbar(),
            move_lefts(2),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("endB")),
            ((1, 8), state("end1")),
            ((1, 14), state("endbar")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("endb")),
            ((4, 6), state("end1")),
            ((4, 7), state("endbar")),
            ((5, 3), state("end")),
            ((6, 9), state("end")),
            ((7, 15), state("end")),
            ((8, 9), state("end")),
            ((9, 10), state("end")),
            ((10, 11), state("endb")),
            ((10, 12), state("end1")),
            ((10, 13), state("endbar")),
            ((11, 3), state("end")),
            ((12, 9), state("end")),
            ((13, 15), state("end")),
            ((14, 15), state("end")),
            ((15, 16), state("end")),
            ((16, 17), state("end")),
        ],
        acceptable: accept_end_only(17),
    };
    naive_builder_composition(graph).unwrap()
}

fn expand_aux_concat() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "concat".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            move_rights(2),
            putb(),
            left_one(),
            bor1orbar(),
            putbar(), // 4
            left_one(),
            bor1orbar(),
            put1(),
            put1(),
            put1(),
            putbar(), // 10
            left_one(),
            bor1orbar(),
            putb(),
            putb(),
            putb(),
            id(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end1")),
            ((3, 11), state("endb")),
            ((3, 16), state("endbar")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("endb")),
            ((6, 8), state("end1")),
            ((6, 9), state("endbar")),
            ((7, 5), state("end")),
            ((8, 11), state("end")),
            ((9, 16), state("end")),
            ((10, 11), state("end")),
            ((11, 12), state("end")),
            ((12, 13), state("endb")),
            ((12, 14), state("end1")),
            ((12, 15), state("endbar")),
            ((13, 5), state("end")),
            ((14, 11), state("end")),
            ((15, 16), state("end")),
        ],
        acceptable: accept_end_only(16),
    };
    naive_builder_composition(graph).unwrap()
}

// -b(x)p- を -1-b(x-1)p-...-b(1)p-p- にする
// ただし、展開後は -b(1)p[-]p- の位置にセットする。
fn expand() -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: "expand".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            expand_aux_shift_right(),
            expand_aux_shift_right(),
            right_one(),
            put1(),
            right_one(),
            copy::copy(),
            move_right(),
            expand_aux_shrink(),
            is_first_of_tuple_zero(),
            expand_aux_shrink(),
            move_left(),
            is_left_sig(),
            id(),
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 3), state("end")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 7), state("end")),
            ((7, 8), state("end")),
            ((8, 5), state("endF")),
            ((8, 9), state("endT")),
            ((9, 10), state("end")),
            ((10, 11), state("end")),
            ((11, 10), state("endF")),
            ((11, 12), state("endT")),
        ],
        acceptable: accept_end_only(12),
    };
    naive_builder_composition(graph).unwrap()
}

pub fn primitive_recursion(
    zero_case: TuringMachineBuilder,
    succ_case: TuringMachineBuilder,
) -> TuringMachineBuilder {
    let graph = GraphOfBuilder {
        name: format!(
            "primitive_recursion_{}_{}",
            zero_case.get_name(),
            succ_builder().get_name()
        ),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            expand(), // 0
            zero_case,
            is_left_sig(),
            move_left(),
            rotate::rotate(2),
            expand_aux_concat(),
            succ_case,
            id(), // 7
        ],
        assign_edge_to_state: vec![
            ((0, 1), state("end")),
            ((1, 2), state("end")),
            ((2, 7), state("endT")),
            ((2, 3), state("endF")),
            ((3, 4), state("end")),
            ((4, 5), state("end")),
            ((5, 6), state("end")),
            ((6, 2), state("end")),
        ],
        acceptable: accept_end_only(7),
    };
    naive_builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_safe() {
        let _ = is_first_of_tuple_zero();
        let _ = is_left_sig();
        let _ = expand_aux_shrink();
        let _ = expand_aux_shift_right();
        let _ = expand_aux_shrink();
        let _ = expand();
    }
}
