use common::dirs::Dirs;
use common::model::{BackendRequestData, BackendResponseData, UiRequestData, UiResponseData};
use common::rpc::backend_api::BackendApi;
use utils::channel::{RequestReceiver, RequestSender};
use crate::ui::GauntletComplexTheme;

pub(in crate) mod ui;
pub(in crate) mod model;
pub mod global_shortcut;

pub fn start_client(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) {
    ui::run(minimized, frontend_receiver, backend_sender);
}

pub fn open_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let result = BackendApi::new().await;

            match result {
                Ok(mut backend_api) => {
                    tracing::info!("Server is already running, opening window...");

                    backend_api.show_window()
                        .await
                        .expect("Unknown error")
                }
                Err(_) => {
                    tracing::error!("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                }
            }
        })
}

pub fn open_settings_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let result = BackendApi::new().await;

            match result {
                Ok(mut backend_api) => {
                    backend_api.show_settings_window()
                        .await
                        .expect("Unknown error")
                }
                Err(_) => {
                    tracing::error!("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                }
            }
        })
}

pub fn generate_complex_theme_sample() -> anyhow::Result<()> {
    let dirs = Dirs::new();

    let sample_complex_theme_file = dirs.sample_complex_theme_file();
    let complex_theme_file = dirs.complex_theme_file();

    let theme_complex = GauntletComplexTheme::default_theme(GauntletComplexTheme::default_simple_theme());

    let string = serde_json::to_string_pretty(&theme_complex)?;

    let sample_theme_parent = sample_complex_theme_file
        .parent()
        .expect("no parent?");

    std::fs::create_dir_all(sample_theme_parent)?;

    std::fs::write(&sample_complex_theme_file, string)?;

    println!("Created sample using default complex theme at {:?}", sample_complex_theme_file);
    println!("Make changes and rename file to {:?}", complex_theme_file.file_name().unwrap());

    Ok(())
}

pub fn generate_simple_theme_sample() -> anyhow::Result<()> {
    let dirs = Dirs::new();

    let sample_simple_theme_file = dirs.sample_simple_theme_color_file();
    let simple_theme_file = dirs.theme_simple_file();

    let theme = GauntletComplexTheme::default_simple_theme();

    let string = serde_json::to_string_pretty(&theme)?;

    let sample_theme_parent = sample_simple_theme_file
        .parent()
        .expect("no parent?");

    std::fs::create_dir_all(sample_theme_parent)?;

    std::fs::write(&sample_simple_theme_file, string)?;

    println!("Created sample using default simple theme at {:?}", sample_simple_theme_file);
    println!("Make changes and rename file to {:?}", simple_theme_file.file_name().unwrap());

    Ok(())
}