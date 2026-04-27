use utils::RenderState;

use crate::tiny_isa::TinyIsaMachine;

pub fn render_machine(snapshot: TinyIsaMachine) -> RenderState {
    let pc = snapshot.regs[0].as_usize().ok();
    let reg_rows = snapshot
        .regs
        .iter()
        .enumerate()
        .map(|(index, value)| {
            utils::render_row!([
                utils::render_text!(format!("r{index}")),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();
    let mem_rows = snapshot
        .memory
        .iter()
        .enumerate()
        .map(|(addr, value)| {
            if Some(addr) == pc {
                utils::render_row!(
                    cells: vec![
                        utils::render_text!(addr.to_string()),
                        utils::render_text!(value.to_decimal_string())
                    ],
                    class: "highlight"
                )
            } else {
                utils::render_row!([
                    utils::render_text!(addr.to_string()),
                    utils::render_text!(value.to_decimal_string())
                ])
            }
        })
        .collect::<Vec<_>>();
    let next = pc
        .and_then(|addr| snapshot.memory.get(addr).cloned())
        .and_then(|word| TinyIsaMachine::decode(&word).ok().map(|inst| format!("{inst:?}")))
        .unwrap_or_else(|| "halt".to_string());

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: vec![
                utils::render_row!([
                    utils::render_text!("pc".to_string()),
                    utils::render_text!(pc.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string()))
                ]),
                utils::render_row!([
                    utils::render_text!("flag".to_string()),
                    utils::render_text!(snapshot.flag.to_string())
                ]),
                utils::render_row!([
                    utils::render_text!("next".to_string()),
                    utils::render_text!(next)
                ]),
                utils::render_row!([
                    utils::render_text!("halted".to_string()),
                    utils::render_text!(snapshot.halted.to_string())
                ])
            ],
            title: "status"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("reg".to_string()), utils::render_text!("value".to_string())],
            rows: reg_rows,
            title: "registers"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("addr".to_string()), utils::render_text!("value".to_string())],
            rows: mem_rows,
            title: "memory"
        )
    ]
}
