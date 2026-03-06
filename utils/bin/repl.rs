use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use utils::machine_ipc::{Request, Response};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

const DEFAULT_SOCKET: &str = "/tmp/computation-machine.sock";

mod web_compiler {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "web-compiler",
    });
}

#[derive(Parser, Debug)]
#[command(about = "Load and call wasm component from wasm_bundle/<name>.component.wasm")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Model(ModelArgs),
    Compiler(CompilerArgs),
}

#[derive(Args, Debug)]
struct ModelArgs {
    #[arg(long, default_value = DEFAULT_SOCKET)]
    socket: String,
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    code: String,
    #[arg(long, default_value = "")]
    ainput: String,
}

#[derive(Args, Debug)]
struct CompilerArgs {
    #[command(subcommand)]
    command: CompilerCommand,
}

#[derive(Subcommand, Debug)]
enum CompilerCommand {
    CompileCode(CompileCodeArgs),
    CompileAinput(CompileAinputArgs),
    CompileRinput(CompileRinputArgs),
    DecodeRoutput(DecodeRoutputArgs),
    DecodeFoutput(DecodeFoutputArgs),
}

#[derive(Args, Debug)]
struct CompileCodeArgs {
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    code: String,
}

#[derive(Args, Debug)]
struct CompileAinputArgs {
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    ainput: String,
}

#[derive(Args, Debug)]
struct CompileRinputArgs {
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    rinput: String,
}

#[derive(Args, Debug)]
struct DecodeRoutputArgs {
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    routput: String,
}

#[derive(Args, Debug)]
struct DecodeFoutputArgs {
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    foutput: String,
}

fn resolve_component_path(name: &str) -> PathBuf {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("utils has workspace parent");
    workspace
        .join("wasm_bundle")
        .join(format!("{name}.component.wasm"))
}

fn new_engine() -> Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    Engine::new(&config).context("failed to initialize wasmtime engine")
}

fn expect_ok<T>(label: &str, value: std::result::Result<T, String>) -> Result<T> {
    value.map_err(|e| anyhow::anyhow!("{label}: {e}"))
}

fn send_request(socket: &str, req: &Request) -> Result<Response> {
    let mut stream = UnixStream::connect(socket).with_context(|| {
        format!("failed to connect daemon socket: {socket} (start with `machine daemon start`)")
    })?;
    let text = serde_json::to_string(req).context("failed to serialize request")?;
    stream
        .write_all(text.as_bytes())
        .context("failed to write request")?;
    stream.write_all(b"\n").context("failed to write newline")?;
    stream.flush().context("failed to flush request")?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .context("failed to read response")?;
    if read == 0 {
        anyhow::bail!("empty response from daemon");
    }
    serde_json::from_str(line.trim_end()).context("invalid response json")
}

fn response_ok(resp: Response) -> Result<(Option<String>, Option<String>, Option<String>)> {
    match resp {
        Response::Error { error } => anyhow::bail!("{error}"),
        Response::Ok {
            create,
            step,
            snapshot,
            ..
        } => Ok((create, step, snapshot)),
    }
}

fn load_component(name: &str) -> Result<(Engine, Component)> {
    let component_path = resolve_component_path(name);
    if !component_path.exists() {
        anyhow::bail!("component not found: {}", component_path.display());
    }

    let engine = new_engine()?;
    let component = Component::from_file(&engine, &component_path)
        .with_context(|| format!("failed to load component: {}", component_path.display()))?;
    Ok((engine, component))
}

fn run_as_model(args: &ModelArgs) -> Result<()> {
    send_request(
        &args.socket,
        &Request::Model {
            name: args.name.clone(),
        },
    )
    .context("web-model select failed")
    .and_then(|r| response_ok(r).map(|_| ()))?;

    let (create, _step, snapshot) = response_ok(send_request(
        &args.socket,
        &Request::Create {
            code: args.code.clone(),
            ainput: args.ainput.clone(),
        },
    )?)?;
    println!("world=web-model");
    if let Some(create) = create {
        println!("create={create}");
    }
    if let Some(snapshot) = snapshot {
        println!("snapshot={snapshot}");
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

        let (_create, step, snapshot) = response_ok(send_request(
            &args.socket,
            &Request::Step { rinput },
        )?)?;
        if let Some(step) = step {
            println!("step={step}");
        }
        if let Some(snapshot) = snapshot {
            println!("snapshot={snapshot}");
        }
    }

    Ok(())
}

fn run_as_compiler(engine: &Engine, component: &Component, args: &CompilerArgs) -> Result<()> {
    let linker = Linker::new(engine);
    let mut store = Store::new(engine, ());
    let instance = web_compiler::WebCompiler::instantiate(&mut store, component, &linker)
        .context("instantiate as web-compiler failed")?;
    println!("world=web-compiler");

    match &args.command {
        CompilerCommand::CompileCode(cmd) => {
            let out = expect_ok(
                "web-compiler compile-code",
                instance
                    .call_compile_code(&mut store, &cmd.code)
                    .context("web-compiler compile-code failed")?,
            )?;
            println!("compile-code={out}");
            Ok(())
        }
        CompilerCommand::CompileAinput(cmd) => {
            let out = expect_ok(
                "web-compiler compile-ainput",
                instance
                    .call_compile_ainput(&mut store, &cmd.ainput)
                    .context("web-compiler compile-ainput failed")?,
            )?;
            println!("compile-ainput={out}");
            Ok(())
        }
        CompilerCommand::CompileRinput(cmd) => {
            let out = expect_ok(
                "web-compiler compile-rinput",
                instance
                    .call_compile_rinput(&mut store, &cmd.rinput)
                    .context("web-compiler compile-rinput failed")?,
            )?;
            println!("compile-rinput={out}");
            Ok(())
        }
        CompilerCommand::DecodeRoutput(cmd) => {
            let out = expect_ok(
                "web-compiler decode-routput",
                instance
                    .call_decode_routput(&mut store, &cmd.routput)
                    .context("web-compiler decode-routput failed")?,
            )?;
            println!("decode-routput={out}");
            Ok(())
        }
        CompilerCommand::DecodeFoutput(cmd) => {
            let out = expect_ok(
                "web-compiler decode-foutput",
                instance
                    .call_decode_foutput(&mut store, &cmd.foutput)
                    .context("web-compiler decode-foutput failed")?,
            )?;
            println!("decode-foutput={out}");
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Model(args) => run_as_model(&args),
        Command::Compiler(args) => {
            let name = match &args.command {
                CompilerCommand::CompileCode(cmd) => &cmd.name,
                CompilerCommand::CompileAinput(cmd) => &cmd.name,
                CompilerCommand::CompileRinput(cmd) => &cmd.name,
                CompilerCommand::DecodeRoutput(cmd) => &cmd.name,
                CompilerCommand::DecodeFoutput(cmd) => &cmd.name,
            };
            let (engine, component) = load_component(name)?;
            run_as_compiler(&engine, &component, &args)
        }
    }
}
