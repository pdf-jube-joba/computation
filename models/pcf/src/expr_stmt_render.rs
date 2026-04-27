use utils::RenderState;

use crate::expr_stmt_machine::{Env, ExprStmtMachine, Frame, State};

pub fn render_machine(snapshot: ExprStmtMachine) -> RenderState {
    let (mode, focus, env, kont) = match &snapshot.state {
        State::EvalExpr { control, env, kont } => {
            ("eval-expr", control.to_string(), env.clone(), kont.clone())
        }
        State::Return { value, kont } => (
            "return",
            format!("{value:?}"),
            env_from_last_frame(kont),
            kont.clone(),
        ),
        State::EvalStmt { control, env, kont } => {
            ("eval-stmt", control.to_string(), env.clone(), kont.clone())
        }
        State::Done { env, kont } => {
            ("done", "scope complete".to_string(), env.clone(), kont.clone())
        }
    };
    let env_rows = env
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(name.as_str().to_string()),
                utils::render_text!(format!("{value:?}"))
            ])
        })
        .collect::<Vec<_>>();
    let kont_rows = kont
        .iter()
        .rev()
        .enumerate()
        .map(|(depth, frame)| {
            utils::render_row!([
                utils::render_text!(depth.to_string()),
                utils::render_text!(format!("{frame:?}"))
            ])
        })
        .collect::<Vec<_>>();

    utils::render_state![
        utils::render_text!(snapshot.code.0.to_string(), title: "code"),
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: vec![
                utils::render_row!([utils::render_text!("mode".to_string()), utils::render_text!(mode.to_string())]),
                utils::render_row!([utils::render_text!("focus".to_string()), utils::render_text!(focus.to_string())]),
            ],
            title: "state"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("name".to_string()), utils::render_text!("value".to_string())],
            rows: env_rows,
            title: "env"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("depth".to_string()), utils::render_text!("frame".to_string())],
            rows: kont_rows,
            title: "kont"
        )
    ]
}

fn env_from_last_frame(kont: &[Frame]) -> Env {
    for frame in kont.iter().rev() {
        match frame {
            Frame::BinOpLeft { env, .. }
            | Frame::AppFun { env, .. }
            | Frame::IfExpr { env, .. }
            | Frame::Let { env, .. }
            | Frame::IfStmt { env, .. }
            | Frame::While { env, .. } => return env.clone(),
            Frame::BinOpRight { .. }
            | Frame::UnOp { .. }
            | Frame::AppArg { .. }
            | Frame::Seq { .. }
            | Frame::ScopeOut { .. }
            | Frame::Init => {}
        }
    }
    Vec::new()
}
