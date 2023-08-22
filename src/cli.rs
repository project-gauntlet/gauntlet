use clap::Parser;
use crate::agent::run_agent;
use crate::server::run_server;


#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    OpenWindow,
}

pub fn init() {
    let cli = Cli::parse();
    match &cli.command {
        None => run_server(false),
        Some(Commands::OpenWindow) => run_agent()
    };
}
