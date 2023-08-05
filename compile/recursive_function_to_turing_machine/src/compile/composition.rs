use turing_machine::manipulation::{
    builder::TuringMachineBuilder,
    graph_compose::{naive_builder_composition, GraphOfBuilder},
};

use super::*;

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
            vec![copy::n_times_copy(num - 1)],
            inner_builder
                .into_iter()
                .map(|builder| {
                    vec![
                        move_rights(num - 1),
                        builder,
                        move_lefts(num - 1),
                        rotate::rotate(num),
                    ]
                })
                .flatten()
                .collect(),
            vec![outer_builder],
        ]
        .into_iter()
        .flatten()
        .collect(),
        assign_edge_to_state: series_edge_end_only(4 * num + 1),
        acceptable: accept_end_only(4 * num + 1),
    };
    naive_builder_composition(graph).unwrap()
}
