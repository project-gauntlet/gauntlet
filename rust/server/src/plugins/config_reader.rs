use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use gauntlet_common::dirs::Dirs;
use serde::Deserialize;

pub struct ConfigReader {
    dirs: Dirs,
    close_on_unfocus: AtomicBool,
}

impl ConfigReader {
    pub fn new(dirs: Dirs) -> Self {
        Self {
            dirs,
            close_on_unfocus: AtomicBool::new(true),
        }
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        let config = self.read_config();

        self.close_on_unfocus.store(
            config.main_window.unwrap_or_default().close_on_unfocus,
            Ordering::SeqCst,
        );

        Ok(())
    }

    fn read_config(&self) -> ApplicationConfig {
        let config_file = self.dirs.config_file();
        let config_content = std::fs::read_to_string(config_file);

        match config_content {
            Ok(config_content) => {
                toml::from_str(&config_content).unwrap_or_else(|err| {
                    tracing::error!("Unable to parse config, error: {:#}", err);

                    ApplicationConfig::default()
                })
            }
            Err(_) => {
                tracing::info!("No config found, using default configuration");

                ApplicationConfig::default()
            }
        }
    }

    pub fn close_on_unfocus(&self) -> bool {
        self.close_on_unfocus.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ApplicationConfig {
    main_window: Option<ApplicationConfigWindow>,
    // #[serde(default)]
    // configuration_mode: ConfigurationModeConfig,
    // #[serde(default)]
    // plugins: Vec<PluginEntryConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationConfigWindow {
    close_on_unfocus: bool,
}

impl Default for ApplicationConfigWindow {
    fn default() -> Self {
        Self { close_on_unfocus: true }
    }
}

// #[derive(Debug, Deserialize)]
// struct PluginEntryConfig {
//     id: String,
// }

// #[derive(Deserialize, Debug, Default)]
// enum ConfigurationModeConfig {
//     #[serde(rename = "config")]
//     Config,
//     #[default]
//     #[serde(rename = "config_and_state")]
//     ConfigAndState
// }
