use std::io::Read;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use cli::runtime::{CompilerHost, ModelHost};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(about = "One-shot machine CLI over wasm components in wasm_bundle/")]
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
    #[arg(value_name = "NAME")]
    name: String,
    #[command(subcommand)]
    command: ModelCommand,
}

#[derive(Subcommand, Debug)]
enum ModelCommand {
    Create {
        #[arg(long)]
        code: String,
        #[arg(long, default_value = "")]
        ainput: String,
    },
    Step {
        #[arg(long)]
        rinput: String,
    },
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

fn read_stdin_snapshot() -> Result<Value> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("missing snapshot JSON on stdin");
    }
    serde_json::from_str(trimmed).context("stdin is not valid JSON")
}

fn run_model_create(name: &str, code: &str, ainput: &str) -> Result<()> {
    let mut model = ModelHost::load(name)?;
    model.create(code, ainput)?;
    println!("{}", model.checkpoint()?);
    Ok(())
}

fn run_model_step(name: &str, rinput: &str) -> Result<()> {
    let snapshot = read_stdin_snapshot()?;
    let snapshot_str = serde_json::to_string(&snapshot)?;

    let mut model = ModelHost::load(name)?;
    model.restore(&snapshot_str)?;
    let step_raw = model.step(rinput)?;
    let step: Value = serde_json::from_str(&step_raw).context("invalid step result JSON")?;
    let kind = step
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing `kind` in step result"))?;

    match kind {
        "continue" => {
            let routput = step
                .get("routput")
                .cloned()
                .unwrap_or(Value::String(String::new()));
            eprintln!("{}", serde_json::to_string(&routput)?);
            println!("{}", model.checkpoint()?);
        }
        "halt" => {
            let foutput = step
                .get("foutput")
                .cloned()
                .unwrap_or(Value::String(String::new()));
            eprintln!("{}", serde_json::to_string(&foutput)?);
        }
        other => anyhow::bail!("unknown step kind: {other}"),
    }
    Ok(())
}

fn run_compiler(args: &CompilerArgs) -> Result<()> {
    let name = match &args.command {
        CompilerCommand::CompileCode(cmd) => &cmd.name,
        CompilerCommand::CompileAinput(cmd) => &cmd.name,
        CompilerCommand::CompileRinput(cmd) => &cmd.name,
        CompilerCommand::DecodeRoutput(cmd) => &cmd.name,
        CompilerCommand::DecodeFoutput(cmd) => &cmd.name,
    };
    let mut compiler = CompilerHost::load(name)?;

    match &args.command {
        CompilerCommand::CompileCode(cmd) => println!("{}", compiler.compile_code(&cmd.code)?),
        CompilerCommand::CompileAinput(cmd) => {
            println!("{}", compiler.compile_ainput(&cmd.ainput)?)
        }
        CompilerCommand::CompileRinput(cmd) => {
            println!("{}", compiler.compile_rinput(&cmd.rinput)?)
        }
        CompilerCommand::DecodeRoutput(cmd) => {
            println!("{}", compiler.decode_routput(&cmd.routput)?)
        }
        CompilerCommand::DecodeFoutput(cmd) => {
            println!("{}", compiler.decode_foutput(&cmd.foutput)?)
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Model(args) => match args.command {
            ModelCommand::Create { code, ainput } => run_model_create(&args.name, &code, &ainput),
            ModelCommand::Step { rinput } => run_model_step(&args.name, &rinput),
        },
        Command::Compiler(args) => run_compiler(&args),
    }
}
