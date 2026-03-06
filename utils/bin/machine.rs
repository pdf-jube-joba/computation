use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use utils::machine_ipc::{Request, Response};

const DEFAULT_SOCKET: &str = "/tmp/computation-machine.sock";
const DEFAULT_PIDFILE: &str = "/tmp/computation-machine.pid";

#[derive(Parser, Debug)]
#[command(about = "One-shot client for machine daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Daemon(DaemonArgs),
    Model {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
        name: String,
    },
    Create {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
        #[arg(long)]
        code: String,
        #[arg(long, default_value = "")]
        ainput: String,
    },
    Step {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
        #[arg(long)]
        rinput: String,
    },
    Current {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },
    Drop {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },
    Ping {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },
}

#[derive(Args, Debug)]
struct DaemonArgs {
    #[command(subcommand)]
    command: DaemonCommand,
}

#[derive(Subcommand, Debug)]
enum DaemonCommand {
    Start {
        #[arg(long, default_value = DEFAULT_SOCKET)]
        socket: String,
        #[arg(long, default_value = DEFAULT_PIDFILE)]
        pidfile: String,
    },
    Kill {
        #[arg(long, default_value = DEFAULT_PIDFILE)]
        pidfile: String,
    },
    Status {
        #[arg(long, default_value = DEFAULT_PIDFILE)]
        pidfile: String,
    },
}

fn send_request(socket: &str, req: &Request) -> Result<Response> {
    let mut stream =
        UnixStream::connect(socket).with_context(|| format!("failed to connect: {socket}"))?;
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

fn print_response(resp: Response) -> Result<()> {
    match resp {
        Response::Error { error } => anyhow::bail!("{error}"),
        Response::Ok {
            message,
            model,
            create,
            step,
            snapshot,
        } => {
            if let Some(msg) = message {
                if let Some(name) = model {
                    println!("[log] {msg}: {name}");
                } else {
                    println!("[log] {msg}");
                }
            }
            if let Some(create) = create {
                println!("[create] {create}");
            }
            if let Some(step) = step {
                println!("[step] {step}");
            }
            if let Some(snapshot) = snapshot {
                println!("[snapshot] {snapshot}");
            }
            Ok(())
        }
    }
}

fn read_pidfile(path: &Path) -> Result<u32> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let pid = text
        .trim()
        .parse::<u32>()
        .with_context(|| format!("invalid pid in {}", path.display()))?;
    Ok(pid)
}

fn write_pidfile(path: &Path, pid: u32) -> Result<()> {
    fs::write(path, format!("{pid}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn is_pid_alive(pid: u32) -> Result<bool> {
    let status = ProcCommand::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .context("failed to run kill -0")?;
    Ok(status.success())
}

fn daemon_executable_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("failed to get current executable path")?;
    let parent = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve executable directory"))?;
    Ok(parent.join("machine-daemon"))
}

fn wait_for_socket(
    child: &mut std::process::Child,
    socket: &Path,
    timeout: Duration,
) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if socket.exists() {
            return Ok(());
        }
        if let Some(status) = child
            .try_wait()
            .context("failed to poll daemon process status")?
        {
            anyhow::bail!("daemon exited early with status: {status}");
        }
        thread::sleep(Duration::from_millis(50));
    }
    anyhow::bail!("daemon started but socket was not created: {}", socket.display())
}

fn daemon_start(socket: &str, pidfile: &str) -> Result<()> {
    let pidfile_path = Path::new(pidfile);
    if pidfile_path.exists() {
        let pid = read_pidfile(pidfile_path)?;
        if is_pid_alive(pid)? {
            anyhow::bail!("daemon already running: pid={pid}");
        }
        fs::remove_file(pidfile_path)
            .with_context(|| format!("failed to remove stale pidfile: {}", pidfile_path.display()))?;
    }

    let daemon = daemon_executable_path()?;
    if !daemon.exists() {
        anyhow::bail!("machine-daemon binary not found: {}", daemon.display());
    }

    let mut child = ProcCommand::new(&daemon)
        .arg("--socket")
        .arg(socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to start daemon: {}", daemon.display()))?;

    write_pidfile(pidfile_path, child.id())?;
    if let Err(err) = wait_for_socket(&mut child, Path::new(socket), Duration::from_secs(2)) {
        let _ = ProcCommand::new("kill")
            .arg(child.id().to_string())
            .status();
        let _ = fs::remove_file(pidfile_path);
        return Err(err);
    }
    println!("[log] started: pid={}", child.id());
    Ok(())
}

fn daemon_kill(pidfile: &str) -> Result<()> {
    let pidfile_path = Path::new(pidfile);
    if !pidfile_path.exists() {
        anyhow::bail!("pidfile not found: {}", pidfile_path.display());
    }
    let pid = read_pidfile(pidfile_path)?;
    let status = ProcCommand::new("kill")
        .arg(pid.to_string())
        .status()
        .context("failed to run kill")?;
    if !status.success() {
        anyhow::bail!("failed to kill daemon: pid={pid}");
    }
    fs::remove_file(pidfile_path)
        .with_context(|| format!("failed to remove pidfile: {}", pidfile_path.display()))?;
    println!("[log] killed: pid={pid}");
    Ok(())
}

fn daemon_status(pidfile: &str) -> Result<()> {
    let pidfile_path = Path::new(pidfile);
    if !pidfile_path.exists() {
        println!("[log] stopped");
        return Ok(());
    }
    let pid = read_pidfile(pidfile_path)?;
    if is_pid_alive(pid)? {
        println!("[log] running: pid={pid}");
    } else {
        println!("[log] stale pidfile: pid={pid}");
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Daemon(args) => match args.command {
            DaemonCommand::Start { socket, pidfile } => daemon_start(&socket, &pidfile),
            DaemonCommand::Kill { pidfile } => daemon_kill(&pidfile),
            DaemonCommand::Status { pidfile } => daemon_status(&pidfile),
        },
        Command::Model { socket, name } => {
            let resp = send_request(&socket, &Request::Model { name })?;
            print_response(resp)
        }
        Command::Create {
            socket,
            code,
            ainput,
        } => {
            let resp = send_request(&socket, &Request::Create { code, ainput })?;
            print_response(resp)
        }
        Command::Step { socket, rinput } => {
            let resp = send_request(&socket, &Request::Step { rinput })?;
            print_response(resp)
        }
        Command::Current { socket } => {
            let resp = send_request(&socket, &Request::Current)?;
            print_response(resp)
        }
        Command::Drop { socket } => {
            let resp = send_request(&socket, &Request::Drop)?;
            print_response(resp)
        }
        Command::Ping { socket } => {
            let resp = send_request(&socket, &Request::Ping)?;
            print_response(resp)
        }
    }
}
