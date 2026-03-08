use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use cli::runtime::CompilerHost;

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
    #[arg(value_name = "NAME")]
    name: String,
    #[arg(long)]
    code: Option<String>,
    #[arg(long)]
    ainput: Option<String>,
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

fn run_as_compiler(name: &str, args: &CompilerArgs) -> Result<()> {
    let mut compiler = CompilerHost::load(name)?;
    println!("world=compiler");

    match &args.command {
        CompilerCommand::CompileCode(cmd) => {
            let out = compiler.compile_code(&cmd.code)?;
            println!("compile-code={out}");
            Ok(())
        }
        CompilerCommand::CompileAinput(cmd) => {
            let out = compiler.compile_ainput(&cmd.ainput)?;
            println!("compile-ainput={out}");
            Ok(())
        }
        CompilerCommand::CompileRinput(cmd) => {
            let out = compiler.compile_rinput(&cmd.rinput)?;
            println!("compile-rinput={out}");
            Ok(())
        }
        CompilerCommand::DecodeRoutput(cmd) => {
            let out = compiler.decode_routput(&cmd.routput)?;
            println!("decode-routput={out}");
            Ok(())
        }
        CompilerCommand::DecodeFoutput(cmd) => {
            let out = compiler.decode_foutput(&cmd.foutput)?;
            println!("decode-foutput={out}");
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Model(args) => {
            cli::repl_model::run(&args.name, args.code.as_deref(), args.ainput.as_deref())
        }
        Command::Compiler(args) => {
            let name = match &args.command {
                CompilerCommand::CompileCode(cmd) => &cmd.name,
                CompilerCommand::CompileAinput(cmd) => &cmd.name,
                CompilerCommand::CompileRinput(cmd) => &cmd.name,
                CompilerCommand::DecodeRoutput(cmd) => &cmd.name,
                CompilerCommand::DecodeFoutput(cmd) => &cmd.name,
            };
            run_as_compiler(name, &args)
        }
    }
}
