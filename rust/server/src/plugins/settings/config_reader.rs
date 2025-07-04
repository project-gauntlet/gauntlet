use gauntlet_common::dirs::Dirs;

use crate::plugins::settings::config::ApplicationConfig;

#[derive(Clone)]
pub struct ConfigReader {
    dirs: Dirs,
}

impl ConfigReader {
    pub fn new(dirs: Dirs) -> Self {
        Self { dirs }
    }

    pub fn read_config(&self) -> ApplicationConfig {
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
}
