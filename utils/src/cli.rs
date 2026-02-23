use std::ffi::OsString;
use std::io;

use clap::{Parser, error::ErrorKind};

use crate::serde_json::Value;
use crate::{Machine, StepResult, TextCodec};

#[derive(Parser, Debug)]
#[command(
    about = "Run a Machine from files, stdin sections, or direct text",
    arg_required_else_help = true
)]
struct CliArgs {
    #[arg(value_name = "code")]
    code_arg: String,
    #[arg(value_name = "ainput")]
    ainput_arg: String,

    #[arg(long, value_name = "TEXT")]
    code_text: Option<String>,
    #[arg(long, value_name = "TEXT")]
    ainput_text: Option<String>,
    #[arg(long, value_name = "DELIM")]
    split: Option<String>,
    #[arg(long, value_name = "TEXT")]
    rinput: Option<String>,
    #[arg(long, value_name = "N")]
    limit: Option<usize>,
    #[arg(long)]
    snapshot: bool,
}

fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

fn read_code_and_ainput_from_stdin(
    delim: &str,
    expect_interactive_rinput: bool,
) -> Result<(String, String), String> {
    let stdin = io::stdin();
    let mut line = String::new();
    let mut code = String::new();
    let mut ainput = String::new();
    let mut section = 0usize;
    eprintln!("[phase] code: enter code, then delimiter '{delim}'");

    loop {
        line.clear();
        let n = stdin.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Missing delimiter while reading stdin sections".to_string());
        }
        let trimmed = line.trim_end_matches(&['\n', '\r'][..]);
        if trimmed == delim {
            section += 1;
            if section == 1 {
                eprintln!("[phase] ainput: enter ainput, then delimiter '{delim}'");
            }
            if section == 2 {
                if expect_interactive_rinput {
                    eprintln!("[phase] rinput: enter runtime input lines");
                }
                break;
            }
            continue;
        }
        if section == 0 {
            code.push_str(&line);
        } else {
            ainput.push_str(&line);
        }
    }

    Ok((code, ainput))
}

fn step_machine<T: Machine>(
    machine: &mut Option<T>,
    rinput: &str,
    snapshot: bool,
) -> Result<bool, String>
where
    T::SnapShot: Into<Value>,
{
    let parsed = T::parse_rinput(rinput)?;
    let current = machine
        .take()
        .ok_or_else(|| "Machine not initialized".to_string())?;
    match current.step(parsed)? {
        StepResult::Continue { next, output: routput } => {
            if snapshot {
                let json: Value = next.current().into();
                let serialized = crate::serde_json::to_string(&json).map_err(|e| e.to_string())?;
                println!("{serialized}");
            }
            println!("{}", <T::ROutput as TextCodec>::print(&routput));
            *machine = Some(next);
            Ok(false)
        }
        StepResult::Halt {
            snapshot: snapshot_value,
            output: foutput,
        } => {
            if snapshot {
                let json: Value = snapshot_value.into();
                let serialized = crate::serde_json::to_string(&json).map_err(|e| e.to_string())?;
                println!("{serialized}");
            }
            println!("{}", <T::FOutput as TextCodec>::print(&foutput));
            Ok(true)
        }
    }
}

fn run_with_args<T: Machine>(parsed: CliArgs) -> Result<(), String>
where
    T::SnapShot: Into<Value>,
{
    if parsed.rinput.is_some() != parsed.limit.is_some() {
        return Err("--rinput and --limit must be specified together".to_string());
    }

    if parsed.split.is_some()
        && (parsed.code_text.is_some()
            || parsed.ainput_text.is_some()
            || parsed.code_arg != "-"
            || parsed.ainput_arg != "-")
    {
        return Err("--split is only valid when <code> and <ainput> are '-'".to_string());
    }

    let (code_src, ainput_src) = if parsed.code_arg == "-" || parsed.ainput_arg == "-" {
        if parsed.code_arg != "-" || parsed.ainput_arg != "-" {
            return Err("Using '-' requires both <code> and <ainput> to be '-'".to_string());
        }
        let delim = parsed
            .split
            .ok_or("--split DELIM is required when using '-' for both inputs")?;
        let (code, ainput) = read_code_and_ainput_from_stdin(&delim, parsed.rinput.is_none())?;
        (Some(code), Some(ainput))
    } else {
        (None, None)
    };

    let code_text = if let Some(text) = parsed.code_text {
        text
    } else if let Some(text) = code_src {
        text
    } else {
        read_file(&parsed.code_arg)?
    };

    let ainput_text = if let Some(text) = parsed.ainput_text {
        text
    } else if let Some(text) = ainput_src {
        text
    } else {
        read_file(&parsed.ainput_arg)?
    };

    let code = T::parse_code(&code_text)?;
    let ainput = T::parse_ainput(&ainput_text)?;
    let mut machine = Some(T::make(code, ainput)?);

    if let (Some(rinput), Some(limit)) = (parsed.rinput.as_deref(), parsed.limit) {
        for _ in 0..limit {
            if step_machine::<T>(&mut machine, rinput, parsed.snapshot)? {
                break;
            }
        }
        return Ok(());
    }

    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        let n = stdin.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let line = line.trim_end_matches(&['\n', '\r'][..]);
        if step_machine::<T>(&mut machine, line, parsed.snapshot)? {
            break;
        }
    }

    Ok(())
}

pub fn run_cli<T: Machine, I, A>(args: I) -> Result<(), clap::Error>
where
    T::SnapShot: Into<Value>,
    I: IntoIterator<Item = A>,
    A: Into<OsString> + Clone,
{
    let parsed = CliArgs::try_parse_from(args)?;
    run_with_args::<T>(parsed).map_err(|e| clap::Error::raw(ErrorKind::ValueValidation, e))
}

pub fn run_main<T: Machine>() -> i32
where
    T::SnapShot: Into<Value>,
{
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
macro_rules! cli_model {
    ($machine:path) => {
        fn main() {
            let code = $crate::cli::run_main::<$machine>();
            std::process::exit(code);
        }
    };
}
