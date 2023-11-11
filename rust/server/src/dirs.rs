use std::path::{Path, PathBuf};

use directories::ProjectDirs;

#[derive(Clone)]
pub struct Dirs {
    inner: ProjectDirs
}

impl Dirs {
    pub fn new() -> Self {
        Self {
            inner: ProjectDirs::from("org", "placeholdername", "PlaceHolderName").unwrap()
        }
    }

    pub fn data_db_file(&self) -> PathBuf {
        self.data_dir().join("data.db")
    }

    pub fn data_dir(&self) -> PathBuf {
        let data_dir = if cfg!(feature = "dev") {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/data")).to_owned()
        } else {
            self.inner.data_dir().to_path_buf()
        };

        std::fs::create_dir_all(&data_dir).unwrap();

        data_dir
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

        std::fs::create_dir_all(&config_dir).unwrap();

        config_dir
    }
}