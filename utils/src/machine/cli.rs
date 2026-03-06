use clap::{Args, Parser, Subcommand};

use crate::machine::config::{DEFAULT_PIDFILE, DEFAULT_SOCKET};
use crate::machine::ipc::Request;

#[derive(Parser, Debug)]
#[command(about = "One-shot client for machine daemon")]
pub struct MachineCli {
    #[command(subcommand)]
    pub command: MachineCommand,
}

#[derive(Subcommand, Debug)]
pub enum MachineCommand {
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
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommand,
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommand {
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

pub fn request_for(command: &MachineCommand) -> Option<(String, Request)> {
    match command {
        MachineCommand::Model { socket, name } => {
            Some((socket.clone(), Request::Model { name: name.clone() }))
        }
        MachineCommand::Create {
            socket,
            code,
            ainput,
        } => Some((
            socket.clone(),
            Request::Create {
                code: code.clone(),
                ainput: ainput.clone(),
            },
        )),
        MachineCommand::Step { socket, rinput } => Some((
            socket.clone(),
            Request::Step {
                rinput: rinput.clone(),
            },
        )),
        MachineCommand::Current { socket } => Some((socket.clone(), Request::Current)),
        MachineCommand::Drop { socket } => Some((socket.clone(), Request::Drop)),
        MachineCommand::Ping { socket } => Some((socket.clone(), Request::Ping)),
        MachineCommand::Daemon(_) => None,
    }
}
