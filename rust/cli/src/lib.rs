use std::backtrace::Backtrace;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use clap::Parser;
use gauntlet_common::cli::is_server_running;
use gauntlet_common::cli::open_window;
use gauntlet_common::cli::run_action;
use gauntlet_common::dirs::Dirs;
use gauntlet_management_client::start_management_client;
use gauntlet_server::PLUGIN_CONNECT_ENV;
use gauntlet_server::PLUGIN_UUID_ENV;
use tracing_subscriber::EnvFilter;
use vergen_pretty::vergen_pretty_env;

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
    tracing_subscriber::fmt::fmt()
        .with_thread_names(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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

            register_panic_hook(std::env::var(PLUGIN_UUID_ENV).ok());

            if let Ok(socket_name) = std::env::var(PLUGIN_CONNECT_ENV) {
                gauntlet_plugin_runtime::run_plugin_runtime(socket_name);

                return;
            }

            tracing::info!("Gauntlet Build Information:");
            for (name, value) in vergen_pretty_env!() {
                if let Some(value) = value {
                    tracing::info!("{}: {}", name, value);
                }
            }

            #[cfg(feature = "scenario_runner")]
            run_scenario_runner();

            #[cfg(not(feature = "scenario_runner"))]
            {
                if is_server_running() {
                    open_window()
                } else {
                    gauntlet_client::run_app(cli.minimized)
                }
            }
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
    use anyhow::Context;
    use anyhow::anyhow;
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
    use anyhow::Context;
    use anyhow::anyhow;
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

fn register_panic_hook(plugin_runtime: Option<String>) {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "full");
    };

    let dirs = Dirs::new();

    let crash_file = match plugin_runtime {
        None => dirs.server_crash_log_file(),
        Some(plugin_uuid) => dirs.plugin_crash_log_file(&plugin_uuid),
    };

    let _ = std::fs::remove_file(&crash_file);

    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload();

        let payload = if let Some(&s) = payload.downcast_ref::<&'static str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<dyn Any>"
        };

        let location = panic_info.location().map(|l| l.to_string());
        let backtrace = Backtrace::capture();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis().to_string())
            .unwrap_or("Unknown".to_string());

        let content = format!(
            "Panic on {}\nPayload: {}\nLocation: {:?}\nBacktrace:\n{}",
            now, payload, location, backtrace
        );

        let crash_file = File::options().create(true).append(true).open(&crash_file);

        if let Ok(mut crash_file) = crash_file {
            let _ = crash_file.write_all(content.as_bytes());
        }

        eprintln!("{}", content);

        exit(101); // poor man's abort on panic because actual setting makes v8 linking fail
    }));
}
