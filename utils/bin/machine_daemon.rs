use anyhow::Result;
use clap::Parser;
use utils::machine::config::DEFAULT_SOCKET;

#[derive(Parser, Debug)]
#[command(about = "Daemon for one-shot machine CLI over Unix socket")]
struct Args {
    #[arg(long, default_value = DEFAULT_SOCKET)]
    socket: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    utils::machine::service::run(&args.socket)
}
