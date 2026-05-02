use utils::RenderState;

use crate::symbolic_asm::SymbolicAsmMachine;

pub fn render_machine(snapshot: SymbolicAsmMachine) -> RenderState {
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
    let data_rows = snapshot
        .data_memory
        .iter()
        .enumerate()
        .map(|(addr, value)| {
            let mut label = snapshot
                .debug_info
                .data_labels_by_addr
                .get(&addr)
                .cloned()
                .unwrap_or_default()
                .join(", ");
            if label.is_empty() {
                label = "-".to_string();
            }
            utils::render_row!([
                utils::render_text!(addr.to_string()),
                utils::render_text!(label),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();
    let next = snapshot
        .insts
        .get(snapshot.pc)
        .map(|inst| format!("{inst:?}"))
        .unwrap_or_else(|| "halt".to_string());

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: vec![
                utils::render_row!([
                    utils::render_text!("pc".to_string()),
                    utils::render_text!(snapshot.pc.to_string())
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
            columns: vec![
                utils::render_text!("addr".to_string()),
                utils::render_text!("label".to_string()),
                utils::render_text!("value".to_string())
            ],
            rows: data_rows,
            title: "data"
        )
    ]
}
