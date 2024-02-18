use clap::Parser;

use client::open_window;
use management_client::start_management_client;
use server::start_server;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Open,
    Server,
    Management,
}

pub fn init() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Open => open_window(),
        Commands::Server => start_server(),
        Commands::Management => start_management_client(),
    };
}
