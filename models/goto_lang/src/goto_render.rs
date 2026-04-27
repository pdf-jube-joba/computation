use utils::RenderState;

use crate::machine::{Command, Program};

pub fn render_machine(snapshot: Program) -> RenderState {
    let pc = snapshot.pc.as_usize().unwrap_or(usize::MAX);
    let env_rows = snapshot
        .env
        .env
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(name.as_str().to_string()),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: vec![
                utils::render_row!([
                    utils::render_text!("pc".to_string()),
                    utils::render_text!(snapshot.pc.to_decimal_string())
                ]),
                utils::render_row!([
                    utils::render_text!("next".to_string()),
                    utils::render_text!(
                        snapshot
                            .commands
                            .0
                            .get(pc)
                            .map(render_command)
                            .unwrap_or_else(|| "halt".to_string())
                    )
                ]),
            ],
            title: "status"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("var".to_string()), utils::render_text!("value".to_string())],
            rows: env_rows,
            title: "env"
        )
    ]
}

fn render_command(command: &Command) -> String {
    match command {
        Command::Clr(var) => format!("clr {}", var.as_str()),
        Command::Inc(var) => format!("inc {}", var.as_str()),
        Command::Dec(var) => format!("dec {}", var.as_str()),
        Command::Cpy(dst, src) => format!("cpy {} <- {}", dst.as_str(), src.as_str()),
        Command::Ifnz(var, target) => format!("ifnz {} : {}", var.as_str(), target.to_decimal_string()),
    }
}
