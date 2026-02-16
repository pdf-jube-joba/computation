pub mod machine;
pub mod manipulation;

use crate::machine::{Process, Program, RecursiveFunctions};
use serde_json::json;
use utils::{json_text, TextCodec};

impl From<Program> for serde_json::Value {
    fn from(program: Program) -> Self {
        let function_text = function_text(&program.function);
        let input_text = program
            .input
            .iter()
            .map(|num| num.print())
            .collect::<Vec<_>>()
            .join(", ");
        let mut status = format!("function: {function_text} | input: ({input_text})");
        if let Some(result) = program.process.result() {
            status.push_str(&format!(" | result: {}", result.print()));
        }

        let status_text = json_text!(status);
        let process_block = render_process(program.process);
        let process_container = json!({
            "kind": "container",
            "title": "process",
            "orientation": "vertical",
            "display": "block",
            "children": [process_block]
        });

        json!([status_text, process_container])
    }
}

fn function_text(function: &RecursiveFunctions) -> String {
    format!("{function}")
}

fn render_process(process: Process) -> serde_json::Value {
    match process {
        Process::Result(num) => json_text!(format!("Result: {}", num.print())),
        Process::Comp { function, args } => {
            let title = json_text!(format!(
                "Call {} ({} args)",
                function_text(&function),
                args.len()
            ));
            let arg_blocks: Vec<serde_json::Value> =
                args.into_iter().map(render_process).collect();
            let args_container = json!({
                "kind": "container",
                "title": "args",
                "orientation": "vertical",
                "display": "block",
                "children": arg_blocks
            });
            json!({
                "kind": "container",
                "orientation": "vertical",
                "display": "block",
                "children": [title, args_container]
            })
        }
        Process::MuOpComp {
            now_index,
            args,
            function,
            process,
        } => {
            let args_text = args
                .iter()
                .map(|num| num.print())
                .collect::<Vec<_>>()
                .join(", ");
            let header = json_text!(format!(
                "MuOp index={} args=({})",
                now_index.print(),
                args_text
            ));
            let function_line = json_text!(format!("Function: {}", function_text(&function)));
            let inner = render_process(*process);
            json!({
                "kind": "container",
                "orientation": "vertical",
                "display": "block",
                "children": [header, function_line, inner]
            })
        }
    }
}
