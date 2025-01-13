use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::Deserialize;

use gauntlet_common::dirs::Dirs;
use crate::plugins::data_db_repository::{DataDbRepository, DbWritePendingPlugin};

pub struct ConfigReader {
    dirs: Dirs,
    repository: DataDbRepository,
    close_on_unfocus: AtomicBool,
}

impl ConfigReader {
    pub fn new(dirs: Dirs, repository: DataDbRepository) -> Self {
        Self {
            dirs,
            repository,
            close_on_unfocus: AtomicBool::new(true),
        }
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        let config = self.read_config();

        // for plugin in config.plugins {
        //     let exists = self.repository.does_plugin_exist(&plugin.id).await?;
        //     if !exists {
        //         let pending = self.repository.is_plugin_pending(&plugin.id).await?;
        //         if !pending {
        //             let pending_plugin = DbWritePendingPlugin {
        //                 id: plugin.id
        //             };
        //             self.repository.save_pending_plugin(pending_plugin).await?
        //         }
        //     }
        // }

        self.close_on_unfocus.store(config.main_window.close_on_unfocus, Ordering::SeqCst);

        Ok(())
    }

    fn read_config(&self) -> ApplicationConfig {
        let config_file = self.dirs.config_file();
        let config_content = std::fs::read_to_string(config_file);

        match config_content {
            Ok(config_content) => {
                toml::from_str(&config_content)
                    .unwrap_or_else(|err| {
                        tracing::error!("Unable to parse config, error: {:?}", err);

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
pub struct ApplicationConfig {
    main_window: ApplicationConfigWindow
    // #[serde(default)]
    // configuration_mode: ConfigurationModeConfig,
    // #[serde(default)]
    // plugins: Vec<PluginEntryConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationConfigWindow {
    close_on_unfocus: bool
}

impl Default for ApplicationConfigWindow {
    fn default() -> Self {
        Self {
            close_on_unfocus: true,
        }
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
