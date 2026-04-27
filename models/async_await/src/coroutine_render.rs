use utils::{RenderDisplay, RenderOrientation, RenderState};

use crate::coroutine::{CoroutineMachine, Frame};

pub fn render_machine(snapshot: CoroutineMachine) -> RenderState {
    let status_rows = vec![
        utils::render_row!([
            utils::render_text!("agents".to_string()),
            utils::render_text!(snapshot.workers.len().to_string())
        ]),
        utils::render_row!([
            utils::render_text!("queue_len".to_string()),
            utils::render_text!(snapshot.queue.len().to_string())
        ]),
        utils::render_row!([
            utils::render_text!("task_count".to_string()),
            utils::render_text!(snapshot.tasks.len().to_string())
        ]),
    ];

    let var_rows = snapshot
        .env
        .vars
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(name.clone()),
                utils::render_text!(value.to_decimal_string())
            ])
        })
        .collect::<Vec<_>>();

    let task_id_rows = snapshot
        .env
        .task_ids
        .iter()
        .map(|(name, value)| {
            utils::render_row!([
                utils::render_text!(format!("${name}")),
                utils::render_text!(
                    value
                        .map(|task_id| task_id.to_string())
                        .unwrap_or_else(|| "none".to_string())
                )
            ])
        })
        .collect::<Vec<_>>();

    let worker_rows = snapshot
        .workers
        .iter()
        .enumerate()
        .map(|(agent, task)| {
            utils::render_row!([
                utils::render_text!(agent.to_string()),
                utils::render_text!(
                    task.map(|task_id| task_id.to_string())
                        .unwrap_or_else(|| "idle".to_string())
                )
            ])
        })
        .collect::<Vec<_>>();

    let queue_rows = snapshot
        .queue
        .iter()
        .enumerate()
        .map(|(slot, task_id)| {
            utils::render_row!([
                utils::render_text!(slot.to_string()),
                utils::render_text!(task_id.to_string())
            ])
        })
        .collect::<Vec<_>>();

    let task_children = snapshot
        .tasks
        .iter()
        .enumerate()
        .map(|(task_id, stack)| render_task(task_id, stack))
        .collect::<Vec<_>>();

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field".to_string()), utils::render_text!("value".to_string())],
            rows: status_rows,
            title: "status"
        ),
        utils::render_container!(
            children: [
                utils::render_table!(
                    columns: vec![utils::render_text!("var".to_string()), utils::render_text!("value".to_string())],
                    rows: var_rows,
                    title: "vars"
                ),
                utils::render_table!(
                    columns: vec![utils::render_text!("id".to_string()), utils::render_text!("task".to_string())],
                    rows: task_id_rows,
                    title: "task_ids"
                )
            ],
            orientation: RenderOrientation::Horizontal,
            display: RenderDisplay::Block,
            title: "env"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("agent".to_string()), utils::render_text!("task".to_string())],
            rows: worker_rows,
            title: "workers"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("slot".to_string()), utils::render_text!("task".to_string())],
            rows: queue_rows,
            title: "queue"
        ),
        utils::render_container!(
            children: task_children,
            orientation: RenderOrientation::Vertical,
            display: RenderDisplay::Block,
            title: "tasks"
        )
    ]
}

fn render_task(task_id: usize, stack: &[Frame]) -> utils::RenderBlock {
    let rows = stack
        .iter()
        .enumerate()
        .map(|(depth, frame)| {
            utils::render_row!([
                utils::render_text!(depth.to_string()),
                utils::render_text!(frame.function.clone()),
                utils::render_text!(frame.pc.to_string())
            ])
        })
        .collect::<Vec<_>>();

    utils::render_table!(
        columns: vec![
            utils::render_text!("depth".to_string()),
            utils::render_text!("function".to_string()),
            utils::render_text!("pc".to_string())
        ],
        rows: rows,
        title: format!("task {task_id}")
    )
}
