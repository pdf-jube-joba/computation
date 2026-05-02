use utils::RenderState;

use crate::secd_machine::SecdMachine;

pub fn render_machine(snapshot: SecdMachine) -> RenderState {
    let stack_rows = snapshot
        .stack
        .iter()
        .rev()
        .enumerate()
        .map(|(depth, value)| {
            utils::render_row!([
                utils::render_text!(depth.to_string()),
                utils::render_text!(format!("{value:?}"))
            ])
        })
        .collect::<Vec<_>>();
    let env_rows = snapshot
        .env
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(name.as_str().to_string()),
                utils::render_text!(format!("{value:?}"))
            ])
        })
        .collect::<Vec<_>>();
    let control_rows = snapshot
        .control
        .iter()
        .enumerate()
        .map(|(pc, instr)| {
            utils::render_row!([
                utils::render_text!(pc.to_string()),
                utils::render_text!(format!("{instr:?}"))
            ])
        })
        .collect::<Vec<_>>();
    let dump_rows = snapshot
        .dump
        .iter()
        .rev()
        .enumerate()
        .map(|(depth, frame)| {
            utils::render_row!([
                utils::render_text!(depth.to_string()),
                utils::render_text!(frame.stack.len().to_string()),
                utils::render_text!(frame.env.len().to_string()),
                utils::render_text!(frame.control.len().to_string())
            ])
        })
        .collect::<Vec<_>>();

    utils::render_state![
        utils::render_text!(format!("{:?}", snapshot.code), title: "code"),
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: vec![
                utils::render_row!([
                    utils::render_text!("next".to_string()),
                    utils::render_text!(
                        snapshot
                            .control
                            .first()
                            .map(|instr| format!("{instr:?}"))
                            .unwrap_or_else(|| "ret".to_string())
                    )
                ]),
                utils::render_row!([
                    utils::render_text!("dump_depth".to_string()),
                    utils::render_text!(snapshot.dump.len().to_string())
                ])
            ],
            title: "status"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("depth".to_string()), utils::render_text!("value".to_string())],
            rows: stack_rows,
            title: "stack"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("name".to_string()), utils::render_text!("value".to_string())],
            rows: env_rows,
            title: "env"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("pc".to_string()), utils::render_text!("instr".to_string())],
            rows: control_rows,
            title: "control"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("depth".to_string()),
                utils::render_text!("stack".to_string()),
                utils::render_text!("env".to_string()),
                utils::render_text!("control".to_string())
            ],
            rows: dump_rows,
            title: "dump"
        )
    ]
}
