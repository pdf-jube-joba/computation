use utils::RenderState;

use crate::cek_machine::{CekMachine, CekState, Env, Frame};

pub fn render_machine(snapshot: CekMachine) -> RenderState {
    let (mode, focus, env, kont) = match &snapshot.state {
        CekState::Eval { control, env, kont } => {
            ("eval", control.to_string(), env.clone(), kont.clone())
        }
        CekState::Return { value, kont } => (
            "return",
            format!("{value:?}"),
            env_from_last_frame(kont),
            kont.clone(),
        ),
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
            Frame::BinOpLeft { env, .. } | Frame::AppFun { env, .. } | Frame::If { env, .. } => {
                return env.clone();
            }
            Frame::BinOpRight { .. }
            | Frame::UnOp { .. }
            | Frame::AppArg { .. }
            | Frame::Init => {}
        }
    }
    Vec::new()
}
