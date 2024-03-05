use serde::Deserialize;
use tracing::{error, info};

use crate::dirs::Dirs;
use crate::plugins::data_db_repository::{DataDbRepository, DbWritePendingPlugin};

pub struct ConfigReader {
    dirs: Dirs,
    repository: DataDbRepository,
}

impl ConfigReader {
    pub fn new(dirs: Dirs, repository: DataDbRepository) -> Self {
        Self {
            dirs,
            repository
        }
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        let config = self.read_config();

        for plugin in config.plugins {
            let exists = self.repository.does_plugin_exist(&plugin.id).await?;
            if !exists {
                let pending = self.repository.is_plugin_pending(&plugin.id).await?;
                if !pending {
                    let pending_plugin = DbWritePendingPlugin {
                        id: plugin.id
                    };
                    self.repository.save_pending_plugin(pending_plugin).await?
                }
            }
        }

        Ok(())
    }

    fn read_config(&self) -> ApplicationConfig {
        let config_file = self.dirs.config_file();
        let config_content = std::fs::read_to_string(config_file);

        match config_content {
            Ok(config_content) => {
                match toml::from_str(&config_content) {
                    Ok(config) => config,
                    Err(err) => {
                        error!("Unable to parse config, error: {error}", error = err.message());

                        ApplicationConfig::default()
                    }
                }
            }
            Err(_) => {
                info!("No config found, using default configuration");

                ApplicationConfig::default()
            }
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ApplicationConfig {
    // #[serde(default)] // TODO
    // configuration_mode: ConfigurationModeConfig,
    #[serde(default)]
    plugins: Vec<PluginEntryConfig>,
}

#[derive(Debug, Deserialize)]
struct PluginEntryConfig {
    id: String,
}

// #[derive(Deserialize, Debug, Default)]
// enum ConfigurationModeConfig {
//     #[serde(rename = "config")]
//     Config,
//     #[default]
//     #[serde(rename = "config_and_state")]
//     ConfigAndState
// }
