use std::io::{self, Write};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::runtime::ModelHost;

pub fn run(name: &str, code: &str, ainput: &str) -> Result<()> {
    let mut model = ModelHost::load(name)?;
    model.create(code, ainput)?;
    let snapshot = model.checkpoint()?;

    println!("world=model");
    println!("create=ok");
    println!("snapshot={snapshot}");

    let mut line = String::new();
    let mut block = String::new();
    let stdin = io::stdin();
    loop {
        print!("rinput> ");
        io::stdout().flush().context("flush stdout failed")?;

        line.clear();
        let read = stdin
            .read_line(&mut line)
            .context("failed to read line from stdin")?;
        if read == 0 {
            break;
        }

        let mut rinput = line.trim_end_matches(['\n', '\r']).to_string();
        if matches!(rinput.as_str(), ":q" | ":quit" | ":exit") {
            break;
        }
        if rinput == ":begin" {
            block.clear();
            loop {
                print!("...> ");
                io::stdout().flush().context("flush stdout failed")?;

                line.clear();
                let read = stdin
                    .read_line(&mut line)
                    .context("failed to read line from stdin")?;
                if read == 0 {
                    break;
                }

                let part = line.trim_end_matches(['\n', '\r']);
                if part == ":end" {
                    break;
                }
                if !block.is_empty() {
                    block.push('\n');
                }
                block.push_str(part);
            }
            rinput = block.clone();
        }

        let step = model.step(&rinput)?;
        println!("step={step}");
        let parsed: Value = serde_json::from_str(&step).context("invalid JSON from step")?;
        if parsed.get("kind").and_then(Value::as_str) == Some("halt") {
            println!("halted");
            break;
        } else {
            println!("snapshot={}", model.checkpoint()?);
        }
    }

    Ok(())
}
