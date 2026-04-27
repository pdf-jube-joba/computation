use utils::RenderState;

use crate::expr_machine::ExprMachine;

pub fn render_machine(snapshot: ExprMachine) -> RenderState {
    utils::render_state![
        utils::render_text!(snapshot.code.0.to_string(), title: "code"),
        utils::render_text!(snapshot.expr.to_string(), title: "current")
    ]
}
