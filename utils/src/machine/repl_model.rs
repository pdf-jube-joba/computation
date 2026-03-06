use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::machine::client::request_ok;
use crate::machine::ipc::{OkBody, Request};

pub fn run(socket: &str, name: &str, code: &str, ainput: &str) -> Result<()> {
    request_ok(
        socket,
        &Request::Model {
            name: name.to_string(),
        },
    )
    .context("web-model select failed")?;

    let create_body = request_ok(
        socket,
        &Request::Create {
            code: code.to_string(),
            ainput: ainput.to_string(),
        },
    )?;

    println!("world=web-model");
    if let OkBody::Created {
        create, snapshot, ..
    } = create_body
    {
        println!("create={create}");
        println!("snapshot={snapshot}");
    } else {
        anyhow::bail!("unexpected response for create");
    }

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

        let step_body = request_ok(socket, &Request::Step { rinput })?;
        if let OkBody::Stepped { step, snapshot, .. } = step_body {
            println!("step={step}");
            println!("snapshot={snapshot}");
        } else {
            anyhow::bail!("unexpected response for step");
        }
    }

    Ok(())
}
