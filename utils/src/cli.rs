use std::ffi::OsString;
use std::io::{self, Read};

use clap::{Parser, error::ErrorKind};

use crate::serde_json::Value;
use crate::{Machine, TextCodec};

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
    #[arg(long)]
    snapshot: bool,
}

fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

fn read_stdin_all() -> Result<String, String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

fn split_stdin(input: &str, delim: &str) -> Result<(String, String, String), String> {
    let mut parts = input.splitn(3, delim);
    let code = parts.next().unwrap_or_default();
    let ainput = parts.next().ok_or("Missing ainput section")?;
    let rinput = parts.next().ok_or("Missing rinput section")?;
    Ok((code.to_string(), ainput.to_string(), rinput.to_string()))
}

fn step_machine<T: Machine>(machine: &mut T, rinput: &str, snapshot: bool) -> Result<bool, String>
where
    T::SnapShot: Into<Value>,
{
    let parsed = T::parse_rinput(rinput)?;
    let output = machine.step(parsed)?;
    if snapshot {
        let json: Value = machine.current().into();
        let serialized = crate::serde_json::to_string(&json).map_err(|e| e.to_string())?;
        println!("{serialized}");
    }
    if let Some(o) = output {
        let s = <T::Output as TextCodec>::print(&o);
        println!("{s}");
        return Ok(true);
    }
    Ok(false)
}

fn run_with_args<T: Machine>(parsed: CliArgs) -> Result<(), String>
where
    T::SnapShot: Into<Value>,
{
    if parsed.split.is_some()
        && (parsed.code_text.is_some()
            || parsed.ainput_text.is_some()
            || parsed.code_arg != "-"
            || parsed.ainput_arg != "-")
    {
        return Err("--split is only valid when <code> and <ainput> are '-'".to_string());
    }

    let (code_src, ainput_src, rinput_src) = if parsed.code_arg == "-" || parsed.ainput_arg == "-"
    {
        if parsed.code_arg != "-" || parsed.ainput_arg != "-" {
            return Err("Using '-' requires both <code> and <ainput> to be '-'".to_string());
        }
        let delim = parsed
            .split
            .ok_or("--split DELIM is required when using '-' for both inputs")?;
        let stdin = read_stdin_all()?;
        let (code, ainput, rinput) = split_stdin(&stdin, &delim)?;
        (Some(code), Some(ainput), Some(rinput))
    } else {
        (None, None, None)
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
    let mut machine = T::make(code, ainput)?;

    if let Some(rinput_text) = rinput_src {
        for raw in rinput_text.split('\n') {
            let line = raw.trim_end_matches('\r');
            if step_machine::<T>(&mut machine, line, parsed.snapshot)? {
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
