pub mod machine;
pub mod manipulation;

use crate::machine::{Command, Program};
use serde_json::json;
use utils::{json_text, TextCodec};

impl From<Program> for serde_json::Value {
    fn from(program: Program) -> Self {
        let pc_index = program.pc.as_usize().ok();
        let code_children: Vec<serde_json::Value> = program
            .commands
            .0
            .into_iter()
            .enumerate()
            .map(|(idx, command)| {
                let text = command_text(&command);
                if pc_index == Some(idx) {
                    json_text!(text, class: "highlight")
                } else {
                    json_text!(text)
                }
            })
            .collect();
        let code_container = json!({
            "kind": "container",
            "title": "code",
            "orientation": "vertical",
            "display": "inline",
            "children": code_children
        });

        let env_rows: Vec<serde_json::Value> = program
            .env
            .env
            .into_iter()
            .map(|(var, value)| {
                json!({
                    "cells": [
                        json_text!(var.as_str()),
                        json_text!(value.print())
                    ]
                })
            })
            .collect();
        let env_table = json!({
            "kind": "table",
            "title": "env",
            "columns": [
                json_text!("var"),
                json_text!("value")
            ],
            "rows": env_rows
        });

        json!([code_container, env_table])
    }
}

fn command_text(command: &Command) -> String {
    match command {
        Command::Clr(var) => format!("clr {}", var.as_str()),
        Command::Inc(var) => format!("inc {}", var.as_str()),
        Command::Dec(var) => format!("dec {}", var.as_str()),
        Command::Cpy(dest, src) => format!("cpy {} {}", dest.as_str(), src.as_str()),
        Command::Ifnz(var, target) => format!("ifnz {} {}", var.as_str(), target.print()),
    }
}
