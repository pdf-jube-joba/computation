use anyhow::Result;
use clap::Parser;

use utils::machine::cli::{request_for, DaemonCommand, MachineCli, MachineCommand};
use utils::machine::client::request_ok;
use utils::machine::output::print_ok_body;

fn run_request(command: &MachineCommand) -> Result<()> {
    let (socket, req) =
        request_for(command).ok_or_else(|| anyhow::anyhow!("daemon command has no IPC request"))?;
    let body = request_ok(&socket, &req)?;
    print_ok_body(body);
    Ok(())
}

fn main() -> Result<()> {
    let cli = MachineCli::parse();
    match &cli.command {
        MachineCommand::Daemon(args) => match &args.command {
            DaemonCommand::Start { socket, pidfile } => {
                utils::machine::daemon_ctl::start(socket, pidfile)
            }
            DaemonCommand::Kill { pidfile } => utils::machine::daemon_ctl::kill(pidfile),
            DaemonCommand::Status { pidfile } => utils::machine::daemon_ctl::status(pidfile),
        },
        other => run_request(other),
    }
}
