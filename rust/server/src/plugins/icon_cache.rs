use gauntlet_common::dirs::Dirs;

#[derive(Clone)]
pub struct IconCache {
    dirs: Dirs,
}

impl IconCache {
    pub fn new(dirs: Dirs) -> Self {
        Self { dirs }
    }

    // legacy
    pub fn clear_all_icon_cache_dir(&self) -> anyhow::Result<()> {
        let cache_dir = self.dirs.icon_cache_dir();
        std::fs::create_dir_all(&cache_dir)?;

        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)?;
        }

        Ok(())
    }
}
