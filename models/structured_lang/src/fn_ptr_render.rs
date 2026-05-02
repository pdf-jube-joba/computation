use utils::{RenderDisplay, RenderOrientation, RenderState};

use crate::fn_ptr_machine::{CallFrame, FnPtrMachine, FrameStore, Value};

pub fn render_machine(snapshot: FnPtrMachine) -> RenderState {
    let status_rows = vec![
        utils::render_row!([
            utils::render_text!("control".to_string()),
            utils::render_text!(if snapshot.current_stmt.is_some() {
                "eval".to_string()
            } else {
                "next".to_string()
            })
        ]),
        utils::render_row!([
            utils::render_text!("function".to_string()),
            utils::render_text!(snapshot.current_function.clone())
        ]),
        utils::render_row!([
            utils::render_text!("current".to_string()),
            utils::render_text!(
                snapshot
                    .current_stmt
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<none>".to_string())
            )
        ]),
        utils::render_row!([
            utils::render_text!("next_frame_id".to_string()),
            utils::render_text!(snapshot.next_frame_id.to_string())
        ]),
    ];

    let env_rows = snapshot
        .env
        .iter()
        .map(|(name, loc)| {
            let value = snapshot
                .store
                .iter()
                .rev()
                .find_map(|frame| frame.values.get(loc))
                .cloned()
                .unwrap_or(Value::NullPtr);
            utils::render_row!([
                utils::render_text!(name.clone()),
                utils::render_text!(loc.to_string()),
                utils::render_text!(value.to_string())
            ])
        })
        .collect::<Vec<_>>();

    let store_children = snapshot
        .store
        .iter()
        .map(render_store_frame)
        .collect::<Vec<_>>();

    let inner_k_children = snapshot
        .inner_k
        .iter()
        .rev()
        .map(|stmt| utils::render_text!(stmt.to_string()))
        .collect::<Vec<_>>();

    let fn_k_rows = snapshot
        .fn_k
        .iter()
        .rev()
        .enumerate()
        .map(|(depth, frame)| render_call_frame(depth, frame))
        .collect::<Vec<_>>();

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: status_rows,
            title: "status"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("var".to_string()),
                utils::render_text!("location".to_string()),
                utils::render_text!("value".to_string())
            ],
            rows: env_rows,
            title: "env"
        ),
        utils::render_container!(
            children: store_children,
            orientation: RenderOrientation::Vertical,
            display: RenderDisplay::Block,
            title: "store"
        ),
        utils::render_container!(
            children: inner_k_children,
            orientation: RenderOrientation::Vertical,
            display: RenderDisplay::Block,
            title: "inner_k"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("depth".to_string()),
                utils::render_text!("function".to_string()),
                utils::render_text!("resume_len".to_string())
            ],
            rows: fn_k_rows,
            title: "fn_k"
        )
    ]
}

fn render_store_frame(frame: &FrameStore) -> utils::RenderBlock {
    let rows = frame
        .values
        .iter()
        .map(|(loc, value)| {
            utils::render_row!([
                utils::render_text!(loc.to_string()),
                utils::render_text!(value.to_string())
            ])
        })
        .collect::<Vec<_>>();

    utils::render_table!(
        columns: vec![utils::render_text!("location".to_string()), utils::render_text!("value".to_string())],
        rows: rows,
        title: format!("frame {}", frame.frame_id)
    )
}

fn render_call_frame(depth: usize, frame: &CallFrame) -> utils::RenderTableRow {
    utils::render_row!([
        utils::render_text!(depth.to_string()),
        utils::render_text!(frame.function_name.clone()),
        utils::render_text!(frame.inner_k.len().to_string())
    ])
}
