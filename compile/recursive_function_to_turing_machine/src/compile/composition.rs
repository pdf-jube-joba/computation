use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{naive_builder_composition, GraphOfBuilder},
};

use super::*;

// -p_1-...-p_n- を -p_1p_2...p_n- にする
fn format(n: usize) -> TuringMachineBuilder {
    if n == 0 {
        panic!("0 is invalid arg");
    }
    if n == 1 {
        return id();
    }
    let graph = GraphOfBuilder {
        name: format!("format_{n}"),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            vec![move_rights(n-2)],
            vec![vec![concat(), move_left()]; n-2].into_iter().flatten().collect(),
            vec![concat()],
        ].into_iter().flatten().collect(),
        assign_edge_to_state: series_edge_end_only(2 * n-3),
        acceptable: accept_end_only(2 * n -3)
    };
    naive_builder_composition(graph).unwrap()
}

pub fn composition(
    inner_builder: Vec<TuringMachineBuilder>,
    outer_builder: TuringMachineBuilder,
) -> TuringMachineBuilder {
    let num = inner_builder.len();
    if num == 0 {}
    let graph = GraphOfBuilder {
        name: "compose".to_string(),
        init_state: state("start"),
        assign_vertex_to_builder: vec![
            vec![copy::n_times_iter(num)],
            inner_builder
                .into_iter()
                .enumerate()
                .map(|(i, builder)| {
                    if i == 0 {
                        vec![
                            move_rights(num - 1),
                            builder,
                            move_lefts(num - 1),
                        ]
                    } else {
                        vec![
                            rotate::rotate(num),
                            move_rights(num - 1),
                            builder,
                            move_lefts(num - 1),
                        ]
                    }
                })
                .flatten()
                .collect(),
            vec![format(num), outer_builder],
        ]
        .into_iter()
        .flatten()
        .collect(),
        assign_edge_to_state: series_edge_end_only(4 * num + 1),
        acceptable: accept_end_only(4 * num + 1),
    };
    naive_builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_test() {
        let mut builder = format(3);
        let tests = vec![
            (
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["-", "-", "-"]),
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
                    right: vec_sign(vec!["", "1", "-", "", "1", "-", "", "1", "-"]),
                },
                TapeAsVec {
                    left: vec![],
                    head: sign("-"),
                    right: vec_sign(vec!["", "1", "", "1","", "1", "-"]),
                },
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
}
