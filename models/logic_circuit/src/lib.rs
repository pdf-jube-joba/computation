pub mod machine;
pub mod manipulation;
pub mod parse;

use serde_json::json;
use utils::{Machine, TextCodec, json_text};
pub use crate::machine::{Graph, LogicCircuit, NamedPin, Signal, LogicCircuitTrait};
use utils::bool::Bool;
pub mod example {
    use crate::manipulation::{init_maps, List};
    use crate::parse::parse;

    pub fn utils_map() -> List {
        let mut list = init_maps();
        let code = include_str!("./logic_circuits/examples.txt");
        parse(code, &mut list).unwrap();
        list
    }
}

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct Snapshot {
    kind: utils::identifier::Identifier,
    graph: Graph,
}

impl Snapshot {
    fn new(kind: utils::identifier::Identifier, graph: Graph) -> Self {
        Snapshot { kind, graph }
    }
}

impl TextCodec for LogicCircuit {
    fn parse(text: &str) -> Result<Self, String> {
        crate::parse::parse_main(text).map_err(|e| e.to_string())
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "logic_circuit")
    }
}

fn parse_named_pin(text: &str) -> Result<NamedPin, String> {
    let text = text.trim();
    let (name, pin) = text
        .rsplit_once('.')
        .ok_or_else(|| "Expected NAME.PIN".to_string())?;
    let name = utils::identifier::Identifier::new(name).map_err(|e| e.to_string())?;
    let pin = utils::identifier::Identifier::new(pin).map_err(|e| e.to_string())?;
    Ok((name, pin))
}

impl TextCodec for Signal {
    fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(Signal::new(vec![]));
        }
        let mut items = Vec::new();
        for item in text.split(',') {
            let item = item.trim();
            let (pin, value) = item
                .rsplit_once('=')
                .ok_or_else(|| "Expected NAME.PIN=BOOL".to_string())?;
            let pin = parse_named_pin(pin)?;
            let value = <Bool as TextCodec>::parse(value)?;
            items.push((pin, value));
        }
        Ok(Signal::new(items))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (idx, (pin, value)) in self.0.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}.{}=", pin.0.as_str(), pin.1.as_str())?;
            value.write_fmt(f)?;
        }
        Ok(())
    }
}

fn find_output_value(graph: &Graph, internal: &NamedPin) -> Option<Bool> {
    graph
        .verts
        .iter()
        .find(|(name, _)| name == &internal.0)
        .and_then(|(_, lc)| {
            lc.get_otputs()
                .into_iter()
                .find(|(pin, _)| pin.1 == internal.1)
                .map(|(_, value)| value)
        })
}

impl From<Snapshot> for serde_json::Value {
    fn from(snapshot: Snapshot) -> Self {
        let mut blocks = Vec::new();
        blocks.push(json_text!(snapshot.kind.to_string(), title: "kind"));

        let input_rows: Vec<serde_json::Value> = snapshot
            .graph
            .inpins_map
            .iter()
            .map(|(external, _)| {
                json!({
                    "cells": [
                        json_text!(external.0.to_string()),
                        json_text!(external.1.to_string())
                    ]
                })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "inputs",
            "columns": [json_text!("name"), json_text!("pin")],
            "rows": input_rows
        }));

        let output_rows: Vec<serde_json::Value> = snapshot
            .graph
            .otpins_map
            .iter()
            .map(|(external, internal)| {
                let value = find_output_value(&snapshot.graph, internal)
                    .map(|b| b.print())
                    .unwrap_or("?".to_string());
                json!({
                    "cells": [
                        json_text!(external.0.to_string()),
                        json_text!(external.1.to_string()),
                        json_text!(value)
                    ]
                })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "outputs",
            "columns": [json_text!("name"), json_text!("pin"), json_text!("value")],
            "rows": output_rows
        }));

        let vert_rows: Vec<serde_json::Value> = snapshot
            .graph
            .verts
            .iter()
            .flat_map(|(name, lc)| {
                lc.get_otputs().into_iter().map(move |(pin, value)| {
                    json!({
                        "cells": [
                            json_text!(name.to_string()),
                            json_text!(pin.1.to_string()),
                            json_text!(value.print())
                        ]
                    })
                })
            })
            .collect();
        blocks.push(json!({
            "kind": "table",
            "title": "verts",
            "columns": [json_text!("vert"), json_text!("pin"), json_text!("value")],
            "rows": vert_rows
        }));

        serde_json::Value::Array(blocks)
    }
}

impl Machine for LogicCircuit {
    type Code = LogicCircuit;
    type AInput = ();
    type RInput = Signal;
    type Output = ();
    type SnapShot = Snapshot;

    fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
        Ok(code)
    }

    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        LogicCircuitTrait::step(self, rinput);
        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        Snapshot::new(self.kind(), self.as_graph_group())
    }
}
