use utils::{RenderDisplay, RenderOrientation, RenderState};

use crate::mini_prog_machine::{Control, Exec, InnerFrame, MiniProgMachine};

pub fn render_machine(snapshot: MiniProgMachine) -> RenderState {
    let status_rows = vec![
        utils::render_row!([
            utils::render_text!("function"),
            utils::render_text!(snapshot.current_function.clone())
        ]),
        utils::render_row!([
            utils::render_text!("exec"),
            utils::render_text!(exec_text(&snapshot.exec))
        ]),
        utils::render_row!([
            utils::render_text!("next_local_id"),
            utils::render_text!(snapshot.next_local_id.to_string())
        ]),
        utils::render_row!([
            utils::render_text!("next_heap_id"),
            utils::render_text!(snapshot.next_heap_id.to_string())
        ]),
    ];

    let env_rows = snapshot
        .env
        .iter()
        .rev()
        .enumerate()
        .flat_map(|(depth, scope)| {
            scope.iter().map(move |(name, loc)| {
                utils::render_row!([
                    utils::render_text!(depth.to_string()),
                    utils::render_text!(name.clone()),
                    utils::render_text!(loc.to_string())
                ])
            })
        })
        .collect::<Vec<_>>();

    let static_rows = snapshot
        .store
        .statics
        .iter()
        .map(|(name, cell)| {
            utils::render_row!([
                utils::render_text!(name.clone()),
                utils::render_text!(cell.ty.to_string()),
                utils::render_text!(cell.value.to_string())
            ])
        })
        .collect::<Vec<_>>();

    let heap_rows = snapshot
        .store
        .heap
        .iter()
        .map(|(id, cell)| {
            utils::render_row!([
                utils::render_text!(id.to_string()),
                utils::render_text!(cell.ty.to_string()),
                utils::render_text!(cell.value.to_string())
            ])
        })
        .collect::<Vec<_>>();

    let locals_children = snapshot
        .store
        .locals
        .iter()
        .rev()
        .enumerate()
        .map(|(fn_depth, scopes)| {
            let scope_children = scopes
                .iter()
                .rev()
                .enumerate()
                .map(|(scope_depth, scope)| {
                    let rows = scope
                        .iter()
                        .map(|(id, cell)| {
                            utils::render_row!([
                                utils::render_text!(id.to_string()),
                                utils::render_text!(cell.ty.to_string()),
                                utils::render_text!(cell.value.to_string())
                            ])
                        })
                        .collect::<Vec<_>>();
                    utils::render_table!(
                        columns: vec![
                            utils::render_text!("local"),
                            utils::render_text!("type"),
                            utils::render_text!("value")
                        ],
                        rows: rows,
                        title: format!("scope {}", scope_depth)
                    )
                })
                .collect::<Vec<_>>();
            utils::render_container!(
                children: scope_children,
                orientation: RenderOrientation::Vertical,
                display: RenderDisplay::Block,
                title: format!("fn frame {}", fn_depth)
            )
        })
        .collect::<Vec<_>>();

    let inner_rows = snapshot
        .inner_k
        .iter()
        .rev()
        .enumerate()
        .map(|(depth, frame)| {
            utils::render_row!([
                utils::render_text!(depth.to_string()),
                utils::render_text!(inner_text(frame))
            ])
        })
        .collect::<Vec<_>>();

    let fn_rows = snapshot
        .fn_k
        .iter()
        .rev()
        .enumerate()
        .map(|(depth, frame)| {
            utils::render_row!([
                utils::render_text!(depth.to_string()),
                utils::render_text!(frame.function_name.clone()),
                utils::render_text!(frame.ret_locs.len().to_string()),
                utils::render_text!(frame.inner_k.len().to_string())
            ])
        })
        .collect::<Vec<_>>();

    utils::render_state![
        utils::render_table!(
            columns: vec![utils::render_text!("field"), utils::render_text!("value")],
            rows: status_rows,
            title: "status"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("scope"),
                utils::render_text!("name"),
                utils::render_text!("local")
            ],
            rows: env_rows,
            title: "env"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("name"),
                utils::render_text!("type"),
                utils::render_text!("value")
            ],
            rows: static_rows,
            title: "statics"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("heap"),
                utils::render_text!("type"),
                utils::render_text!("value")
            ],
            rows: heap_rows,
            title: "heap"
        ),
        utils::render_container!(
            children: locals_children,
            orientation: RenderOrientation::Vertical,
            display: RenderDisplay::Block,
            title: "locals"
        ),
        utils::render_table!(
            columns: vec![utils::render_text!("depth"), utils::render_text!("frame")],
            rows: inner_rows,
            title: "inner_k"
        ),
        utils::render_table!(
            columns: vec![
                utils::render_text!("depth"),
                utils::render_text!("function"),
                utils::render_text!("rets"),
                utils::render_text!("resume")
            ],
            rows: fn_rows,
            title: "fn_k"
        )
    ]
}

fn exec_text(exec: &Exec) -> String {
    match exec {
        Exec::Eval(stmt) => format!("eval {stmt}"),
        Exec::Ctrl(Control::Next) => "ctrl next".to_string(),
        Exec::Ctrl(Control::Break(label)) => format!("ctrl break {label}"),
        Exec::Ctrl(Control::Continue(label)) => format!("ctrl continue {label}"),
        Exec::Ret(values) => format!(
            "ret {}",
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn inner_text(frame: &InnerFrame) -> String {
    match frame {
        InnerFrame::Stmt(stmt) => format!("stmt {stmt}"),
        InnerFrame::Scope => "scope".to_string(),
        InnerFrame::Loop { label, stmt } => format!("loop {label}: {stmt}"),
    }
}
