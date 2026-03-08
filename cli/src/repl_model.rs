use std::io::{self, Write};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::runtime::ModelHost;

fn read_multiline_until_eof(stdin: &io::Stdin) -> Result<String> {
    let mut out = String::new();
    let mut line = String::new();
    print!("code> ");
    io::stdout().flush().context("flush stdout failed")?;
    loop {
        line.clear();
        let read = stdin
            .read_line(&mut line)
            .context("failed to read line from stdin")?;
        if read == 0 {
            break;
        }
        let part = line.trim_end_matches(['\n', '\r']);
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(part);
    }
    Ok(out)
}

fn read_ainput(stdin: &io::Stdin) -> Result<String> {
    println!("ainput を1行で入力してください（空にしたい場合は Enter）。");
    print!("ainput> ");
    io::stdout().flush().context("flush stdout failed")?;

    let mut line = String::new();
    let read = stdin
        .read_line(&mut line)
        .context("failed to read line from stdin")?;
    if read == 0 {
        return Ok(String::new());
    }
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}

pub fn run(name: &str, code: Option<&str>, ainput: Option<&str>) -> Result<()> {
    let stdin = io::stdin();
    let code = match code {
        Some(code) => code.to_string(),
        None => {
            println!("code を入力してください。複数行入力でき、確定は Ctrl-D です。");
            read_multiline_until_eof(&stdin)?
        }
    };
    let ainput = match ainput {
        Some(ainput) => ainput.to_string(),
        None => read_ainput(&stdin)?,
    };

    let mut model = ModelHost::load(name)?;
    model.create(&code, &ainput)?;
    let snapshot = model.checkpoint()?;

    println!("world=model");
    println!("create=ok");
    println!("snapshot={snapshot}");

    let mut line = String::new();
    let mut block = String::new();
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
