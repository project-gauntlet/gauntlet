use std::path::{Path, PathBuf};
use anyhow::Context;

use directories::{BaseDirs, ProjectDirs};

#[derive(Clone)]
pub struct Dirs {
    inner: ProjectDirs
}

impl Dirs {
    pub fn new() -> Self {
        Self {
            inner: ProjectDirs::from("dev", "project-gauntlet", "Gauntlet").unwrap()
        }
    }

    pub fn home_dir(&self) -> PathBuf {
        let path = BaseDirs::new()
            .expect("System didn't report any home directory")
            .home_dir()
            .to_path_buf();

        path
    }

    pub fn data_db_file(&self) -> anyhow::Result<PathBuf> {
        let path = self.data_dir()?.join("data.db");
        Ok(path)
    }

    pub fn plugin_data(&self, plugin_uuid: &str) -> anyhow::Result<PathBuf> {
        let plugin_data_dir = self.data_dir()?.join("plugins").join(&plugin_uuid);

        std::fs::create_dir_all(&plugin_data_dir)
            .context("Unable to create plugin data directory")?;

        Ok(plugin_data_dir)
    }

    pub fn data_dir(&self) -> anyhow::Result<PathBuf> {
        let data_dir = if cfg!(feature = "release") || cfg!(feature = "scenario_runner") {
            self.inner.data_dir().to_path_buf()
        } else {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_data/data")).to_owned()
        };

        std::fs::create_dir_all(&data_dir)
            .context("Unable to create data directory")?;

        Ok(data_dir)
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    pub fn complex_theme_file(&self) -> PathBuf {
        self.config_dir().join("complex-theme.json")
    }

    pub fn sample_complex_theme_file(&self) -> PathBuf {
        self.config_dir().join("complex-theme.sample.json")
    }

    pub fn theme_simplified_file(&self) -> PathBuf {
        self.config_dir().join("theme.json")
    }

    pub fn sample_simplified_theme_color_file(&self) -> PathBuf {
        self.config_dir().join("theme.sample.json")
    }

    pub fn config_dir(&self) -> PathBuf {
        let config_dir = if cfg!(feature = "release") || cfg!(feature = "scenario_runner") {
            self.inner.config_dir().to_path_buf()
        } else {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_data/config")).to_owned()
        };

        config_dir
    }

    pub fn icon_cache_dir(&self) -> PathBuf {
        self.cache_dir().join("icons")
    }

    pub fn plugin_cache(&self, plugin_uuid: &str) -> anyhow::Result<PathBuf> {
        let plugin_cache_dir = self.cache_dir().join("plugins").join(&plugin_uuid);

        std::fs::create_dir_all(&plugin_cache_dir)
            .context("Unable to create plugin cache directory")?;

        Ok(plugin_cache_dir)
    }

    pub fn cache_dir(&self) -> PathBuf {
        let cache_dir = if cfg!(feature = "release") || cfg!(feature = "scenario_runner") {
            self.inner.cache_dir().to_path_buf()
        } else {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_data/cache")).to_owned()
        };

        cache_dir
    }

    pub fn plugin_log_files(&self, plugin_uuid: &str) -> (PathBuf, PathBuf) {
        let plugin_dir = self.state_dir().join("logs").join(&plugin_uuid);

        let out_log_file = plugin_dir.join("stdout.txt");
        let err_log_file = plugin_dir.join("stderr.txt");

        (out_log_file, err_log_file)
    }

    pub fn plugin_local_storage(&self, plugin_uuid: &str) -> PathBuf {
        self.state_dir().join("local_storage").join(&plugin_uuid)
    }

    pub fn state_dir(&self) -> PathBuf {
        let state_dir = if cfg!(feature = "release") || cfg!(feature = "scenario_runner") {
            let dir = match self.inner.state_dir() {
                Some(dir) => dir,
                None => self.inner.data_local_dir(),
            };

            dir.to_path_buf()
        } else {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_data/state")).to_owned()
        };

        state_dir
    }
}