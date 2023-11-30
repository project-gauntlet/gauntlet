use std::path::{Path, PathBuf};
use anyhow::Context;

use directories::ProjectDirs;

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

    pub fn data_db_file(&self) -> anyhow::Result<PathBuf> {
        let path = self.data_dir()?.join("data.db");
        Ok(path)
    }

    pub fn data_dir(&self) -> anyhow::Result<PathBuf> {
        let data_dir = if cfg!(feature = "dev") {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/data")).to_owned()
        } else {
            self.inner.data_dir().to_path_buf()
        };

        std::fs::create_dir_all(&data_dir)
            .context("Unable to create data directory")?;

        Ok(data_dir)
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    pub fn config_dir(&self) -> PathBuf {
        let config_dir = if cfg!(feature = "dev") {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/config")).to_owned()
        } else {
            self.inner.config_dir().to_path_buf()
        };

        config_dir
    }
}