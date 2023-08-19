use clap::Parser;
use crate::agent::run_agent;
use crate::server::run_server;

#[derive(clap::Parser)]
enum Cli {
    Server,
    OpenWindow,
}

pub fn init() {
    match Cli::parse() {
        Cli::Server => run_server(),
        Cli::OpenWindow => run_agent()
    };
}
