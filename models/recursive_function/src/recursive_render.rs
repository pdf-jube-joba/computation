use utils::RenderState;

use crate::machine::Program;

pub fn render_machine(snapshot: Program) -> RenderState {
    let input_rows = snapshot
        .input
        .iter()
        .enumerate()
        .map(|(index, value)| {
            utils::render_row!([
                utils::render_text!(index.to_string()),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();
    utils::render_state![
        utils::render_text!(snapshot.function.to_string(), title: "function"),
        utils::render_table!(
            columns: vec![utils::render_text!("arg".to_string()), utils::render_text!("value".to_string())],
            rows: input_rows,
            title: "input"
        ),
        utils::render_text!(snapshot.process.to_string(), title: "process")
    ]
}
