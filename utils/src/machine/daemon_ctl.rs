use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

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
    let status = Command::new("kill")
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

fn wait_for_socket(child: &mut Child, socket: &Path, timeout: Duration) -> Result<()> {
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

pub fn start(socket: &str, pidfile: &str) -> Result<()> {
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

    let mut child = Command::new(&daemon)
        .arg("--socket")
        .arg(socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to start daemon: {}", daemon.display()))?;

    write_pidfile(pidfile_path, child.id())?;
    if let Err(err) = wait_for_socket(&mut child, Path::new(socket), Duration::from_secs(2)) {
        let _ = Command::new("kill").arg(child.id().to_string()).status();
        let _ = fs::remove_file(pidfile_path);
        return Err(err);
    }
    println!("[log] started: pid={}", child.id());
    Ok(())
}

pub fn kill(pidfile: &str) -> Result<()> {
    let pidfile_path = Path::new(pidfile);
    if !pidfile_path.exists() {
        anyhow::bail!("pidfile not found: {}", pidfile_path.display());
    }
    let pid = read_pidfile(pidfile_path)?;
    let status = Command::new("kill")
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

pub fn status(pidfile: &str) -> Result<()> {
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
