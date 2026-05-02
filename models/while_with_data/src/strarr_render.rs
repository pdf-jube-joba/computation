use utils::RenderState;

use crate::strarr::{StrArrMachine, char_list_to_text, stmt_to_text};

pub fn render_machine(snapshot: StrArrMachine) -> RenderState {
    let env_rows = snapshot
        .env
        .vars
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(name.as_str().to_string()),
                utils::render_text!(char_list_to_text(value))
            ])
        })
        .collect::<Vec<_>>();
    utils::render_state![
        utils::render_text!(stmt_to_text(&snapshot.stmt), title: "stmt"),
        utils::render_table!(
            columns: vec![utils::render_text!("var".to_string()), utils::render_text!("value".to_string())],
            rows: env_rows,
            title: "env"
        )
    ]
}
