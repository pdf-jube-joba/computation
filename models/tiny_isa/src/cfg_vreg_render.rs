use utils::RenderState;

use crate::cfg_vreg::CfgVRegMachine;

pub fn render_machine(snapshot: CfgVRegMachine) -> RenderState {
    let current_block = snapshot.compiled.blocks.get(snapshot.current_block);
    let vreg_rows = snapshot
        .vregs
        .iter()
        .enumerate()
        .map(|(index, value)| {
            utils::render_row!([
                utils::render_text!(format!("v{index}")),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();
    let mem_rows = snapshot
        .memory
        .iter()
        .enumerate()
        .map(|(addr, value)| {
            utils::render_row!([
                utils::render_text!(addr.to_string()),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();
    let block_rows = current_block
        .map(|block| {
            block
                .stmts
                .iter()
                .enumerate()
                .map(|(index, stmt)| {
                    utils::render_row!([
                        utils::render_text!(index.to_string()),
                        utils::render_text!(format!("{stmt:?}"))
                    ])
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: vec![
                utils::render_row!([
                    utils::render_text!("block".to_string()),
                    utils::render_text!(snapshot.current_block.to_string())
                ]),
                utils::render_row!([
                    utils::render_text!("label".to_string()),
                    utils::render_text!(
                        current_block
                            .map(|block| block.label.clone())
                            .unwrap_or_else(|| "<invalid>".to_string())
                    )
                ]),
                utils::render_row!([
                    utils::render_text!("halted".to_string()),
                    utils::render_text!(snapshot.halted.to_string())
                ])
            ],
            title: "status"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("index".to_string()), utils::render_text!("stmt".to_string())],
            rows: block_rows,
            title: "block"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("vreg".to_string()), utils::render_text!("value".to_string())],
            rows: vreg_rows,
            title: "vregs"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("addr".to_string()), utils::render_text!("value".to_string())],
            rows: mem_rows,
            title: "memory"
        )
    ]
}
