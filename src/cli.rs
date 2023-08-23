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
        None => {
            let dev = std::env::var("PLACEHOLDERNAME_DEV").is_ok();
            run_server(dev)
        },
        Some(Commands::OpenWindow) => run_agent()
    };
}
