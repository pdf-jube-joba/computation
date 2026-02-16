use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{builder_composition, GraphOfBuilder},
};

use crate::auxiliary::{basic, copy, rotate};
use crate::*;

// -x_1-...-x_n- を -x_1x_2...x_n- にする
fn format(n: usize) -> TuringMachineBuilder {
    if n == 0 || n == 1 {
        return basic::id();
    }
    let graph = GraphOfBuilder {
        name: format!("format_{n}"),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            vec![basic::move_rights(n - 2)],
            vec![vec![basic::concat(), basic::move_left()]; n - 2]
                .into_iter()
                .flatten()
                .collect(),
            vec![basic::concat()],
        ]
        .into_iter()
        .flatten()
        .collect(),
        assign_edge_to_state: series_edge_end_only(2 * n - 3),
        acceptable: accept_end_only(2 * n - 3),
    };
    builder_composition(graph).unwrap()
}

pub fn composition(
    inner_builder: Vec<TuringMachineBuilder>,
    outer_builder: TuringMachineBuilder,
) -> TuringMachineBuilder {
    let num = inner_builder.len();
    let graph = GraphOfBuilder {
        name: "compose".to_string(),
        init_state: "start".parse_tc().unwrap(),
        assign_vertex_to_builder: vec![
            vec![copy::n_times_iter(num)],
            inner_builder
                .into_iter()
                .enumerate()
                .flat_map(|(i, builder)| {
                    vec![
                        basic::move_rights(num - 1),
                        builder,
                        basic::move_lefts(num - 1),
                        if i != num - 1 {
                            rotate::rotate(num)
                        } else {
                            format(num)
                        },
                    ]
                })
                .collect(),
            vec![outer_builder],
        ]
        .into_iter()
        .flatten()
        .collect(),
        assign_edge_to_state: series_edge_end_only(4 * num + 1),
        acceptable: accept_end_only(4 * num + 1),
    };
    builder_composition(graph).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_test() {
        let mut builder = format(3);
        let tests = vec![
            (
                tape_from(&["x", "x", "x", "x"], 0),
                tape_from(&["x", "x"], 0),
            ),
            (
                tape_from(&["x", "-", "l", "x", "-", "l", "x", "-", "l", "x"], 0),
                tape_from(&["x", "-", "l", "-", "l", "-", "l", "x"], 0),
            ),
        ];
        builder_test(&mut builder, 500, tests);
    }
}
