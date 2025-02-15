use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context;
use clap::Parser;
use gauntlet_client::open_window;
use gauntlet_management_client::start_management_client;
use gauntlet_server::run_action;
use gauntlet_server::start;

/// Gauntlet CLI
///
/// If no subcommand is provided server will be started or if one is already running window will be opened
#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Start server without opening Gauntlet window, only used if no subcommand is provided
    #[arg(long)]
    minimized: bool,

    /// Display version and exit
    #[arg(long)]
    version: bool,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Open Gauntlet window
    Open,
    /// Open Gauntlet settings
    Settings,
    /// Run action (only ones visible in main window search results) of specific entrypoint of specific plugin
    Run {
        /// Plugin ID, can be found in settings
        plugin_id: String,

        /// Entrypoint ID, can be found in plugin manifest at `entrypoint.*.id`
        entrypoint_id: String,

        /// Action ID, can be found in plugin manifest at `entrypoint.actions.*.id`.
        /// Alternatively, following special values are supported:
        /// `:primary` (action run with Enter shortcut) or
        /// `:secondary` (action run with Shift+Enter shortcut)
        action_id: String,
    },
}

pub fn init() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if cli.version {
        println!(
            "Gauntlet v{}",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../VERSION"))
        );
        return;
    }

    match cli.command {
        None => {
            if cfg!(feature = "release") {
                #[cfg(target_os = "macos")]
                let result = setup_auto_launch_macos();

                #[cfg(target_os = "windows")]
                let result = setup_auto_launch_windows();

                #[cfg(any(target_os = "macos", target_os = "windows"))]
                if let Err(err) = &result {
                    tracing::warn!("error occurred when setting up auto-launch {:?}", err)
                }
            }

            start(cli.minimized)
        }
        Some(command) => {
            match command {
                Commands::Open => open_window(),
                Commands::Settings => start_management_client(),
                Commands::Run {
                    plugin_id,
                    entrypoint_id,
                    action_id,
                } => {
                    run_action(plugin_id, entrypoint_id, action_id);
                }
            };
        }
    }
}

#[cfg(target_os = "macos")]
fn setup_auto_launch_macos() -> anyhow::Result<()> {
    let app_path = std::env::current_exe().context("Unable to get current_exe from env")?;

    // expect Gauntlet.app in path according to macos app bundle structure
    let app_path_fn = || {
        let path = std::path::PathBuf::from(&app_path);
        let path = path.parent()?.parent()?.parent()?;
        let extension = path.extension()?.to_str()?;
        match extension == "app" {
            true => Some(path.as_os_str().to_str()?.to_string()),
            false => None,
        }
    };

    let app_path = app_path_fn().ok_or(anyhow!("Unexpected executable path: {:?}", &app_path))?;

    setup_auto_launch(app_path)
}

#[cfg(target_os = "windows")]
fn setup_auto_launch_windows() -> anyhow::Result<()> {
    let app_path = std::env::current_exe()
        .context("Unable to get current_exe from env")?
        .as_os_str()
        .to_str()
        .ok_or(anyhow!("failed to convert app_path to utf-8"))?
        .to_string();

    setup_auto_launch(app_path)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn setup_auto_launch(app_path: String) -> anyhow::Result<()> {
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Gauntlet")
        .set_app_path(&app_path)
        .set_args(&["--minimized"])
        .build()
        .and_then(|auto| auto.enable())?;

    Ok(())
}
