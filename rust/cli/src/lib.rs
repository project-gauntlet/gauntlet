use std::thread;
use std::time::Duration;

use clap::Parser;

use client::start_client;
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
    Standalone,
    Server,
    Management,
}

pub fn init() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Open => start_client(),
        Commands::Server => start_server(),
        Commands::Management => start_management_client(),
        Commands::Standalone => {
            thread::spawn(|| {
                start_server();
            });

            thread::sleep(Duration::from_secs(2));

            start_client();
        }
    };
}
