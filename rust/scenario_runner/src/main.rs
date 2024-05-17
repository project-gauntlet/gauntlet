pub mod frontend_mock;
pub mod backend_mock;
mod model;

use clap::Parser;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Server,
    Frontend,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Server => backend_mock::start_mock_backend().await,
        Commands::Frontend => frontend_mock::start_mock_frontend().await?,
    };

    Ok(())
}

