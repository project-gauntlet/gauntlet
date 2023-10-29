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

    pub fn data_dir(&self) -> PathBuf {
        if cfg!(feature = "dev") {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/data")
        } else {
            self.inner.data_dir().to_path_buf()
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        if cfg!(feature = "dev") {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/config")
        } else {
            self.inner.config_dir().to_path_buf()
        }
    }
}