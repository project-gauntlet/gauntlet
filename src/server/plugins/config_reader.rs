use anyhow::Context;

use crate::server::dirs::Dirs;
use crate::server::plugins::Config;

pub struct ConfigReader {
    dirs: Dirs,
}

impl ConfigReader {
    pub fn new(dirs: Dirs) -> Self {
        Self {
            dirs
        }
    }

    pub fn read_config(&self) -> Config {
        let config_dir = self.dirs.config_dir();

        std::fs::create_dir_all(&config_dir).unwrap();

        let config_file = config_dir.join("config.toml");
        let config_file_context = config_file.display().to_string();
        let config_content = std::fs::read_to_string(config_file).context(config_file_context).unwrap();
        let config: Config = toml::from_str(&config_content).unwrap();

        config
    }
}