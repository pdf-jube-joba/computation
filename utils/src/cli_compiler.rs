use std::ffi::OsString;
use std::io::{self, Read};

use clap::{Parser, Subcommand, error::ErrorKind};

use crate::{Compiler, Machine, TextCodec};

#[derive(Parser, Debug)]
#[command(about = "Run compiler helpers from CLI", arg_required_else_help = true)]
struct CliArgs {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand, Debug)]
enum TopCommand {
    Transpile(TranspileArgs),
}

#[derive(Parser, Debug)]
struct TranspileArgs {
    #[command(subcommand)]
    command: TranspileCommand,
}

#[derive(Subcommand, Debug)]
enum TranspileCommand {
    Code(InputArg),
    Ainput(InputArg),
    Rinput(InputArg),
    Routput(InputArg),
    Foutput(InputArg),
}

#[derive(Parser, Debug)]
struct InputArg {
    #[arg(value_name = "input", required_unless_present = "text")]
    input: Option<String>,
    #[arg(long, value_name = "TEXT", conflicts_with = "input")]
    text: Option<String>,
}

fn read_input(input: InputArg) -> Result<String, String> {
    if let Some(text) = input.text {
        return Ok(text);
    }
    let path = input
        .input
        .ok_or("missing input: provide <path|-> or --text")?;
    if path == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| e.to_string())?;
        return Ok(buf);
    }
    std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))
}

fn transpile_code<T: Compiler>(input: InputArg) -> Result<String, String> {
    let src = read_input(input)?;
    let source_code = <T::Source as Machine>::parse_code(&src)?;
    let target_code = T::compile(source_code)?;
    Ok(target_code.print())
}

fn transpile_ainput<T: Compiler>(input: InputArg) -> Result<String, String> {
    let src = read_input(input)?;
    let source_ainput = <T as Compiler>::Source::parse_ainput(&src)?;
    let target_ainput = T::encode_ainput(source_ainput)?;
    Ok(target_ainput.print())
}

fn transpile_rinput<T: Compiler>(input: InputArg) -> Result<String, String> {
    let src = read_input(input)?;
    let source_rinput = <T as Compiler>::Source::parse_rinput(&src)?;
    let target_rinput = T::encode_rinput(source_rinput)?;
    Ok(target_rinput.print())
}

fn transpile_routput<T: Compiler>(input: InputArg) -> Result<String, String> {
    let src = read_input(input)?;
    let output_target = <<<T as Compiler>::Target as Machine>::ROutput as TextCodec>::parse(&src)?;
    let output_source = T::decode_routput(output_target)?;
    Ok(output_source.print())
}

fn transpile_foutput<T: Compiler>(input: InputArg) -> Result<String, String> {
    let src = read_input(input)?;
    let output_target = <<<T as Compiler>::Target as Machine>::FOutput as TextCodec>::parse(&src)?;
    let output_source = T::decode_foutput(output_target)?;
    Ok(output_source.print())
}

fn run_with_args<T: Compiler>(args: CliArgs) -> Result<(), String> {
    let output = match args.command {
        TopCommand::Transpile(transpile) => match transpile.command {
            TranspileCommand::Code(input) => transpile_code::<T>(input)?,
            TranspileCommand::Ainput(input) => transpile_ainput::<T>(input)?,
            TranspileCommand::Rinput(input) => transpile_rinput::<T>(input)?,
            TranspileCommand::Routput(input) => transpile_routput::<T>(input)?,
            TranspileCommand::Foutput(input) => transpile_foutput::<T>(input)?,
        },
    };
    println!("{output}");
    Ok(())
}

pub fn run_cli<T: Compiler, I, A>(args: I) -> Result<(), clap::Error>
where
    I: IntoIterator<Item = A>,
    A: Into<OsString> + Clone,
{
    let parsed = CliArgs::try_parse_from(args)?;
    run_with_args::<T>(parsed).map_err(|e| clap::Error::raw(ErrorKind::ValueValidation, e))
}

pub fn run_main<T: Compiler>() -> i32 {
    match run_cli::<T, _, _>(std::env::args_os()) {
        Ok(()) => 0,
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                print!("{e}");
                0
            }
            _ => {
                eprint!("{e}");
                1
            }
        },
    }
}

#[macro_export]
macro_rules! cli_compiler {
    ($compiler:path) => {
        fn main() {
            let code = $crate::cli_compiler::run_main::<$compiler>();
            std::process::exit(code);
        }
    };
}
