use utils::RenderState;

use crate::expr_lang::{ExprLangMachine, expr_lang_parser};

pub fn render_machine(snapshot: ExprLangMachine) -> RenderState {
    let env_rows = snapshot
        .env
        .vars
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(name.as_str().to_string()),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();
    utils::render_state![
        utils::render_text!(expr_lang_parser::stmt_to_text(&snapshot.stmt), title: "stmt"),
        utils::render_table!(
            columns: vec![utils::render_text!("var".to_string()), utils::render_text!("value".to_string())],
            rows: env_rows,
            title: "env"
        )
    ]
}
