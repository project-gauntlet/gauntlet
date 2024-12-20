use anyhow::anyhow;
use gauntlet_common::dirs::Dirs;

#[derive(Clone)]
pub struct IconCache {
    dirs: Dirs,
}

impl IconCache {
    pub fn new(dirs: Dirs) -> Self {
        Self {
            dirs
        }
    }

    pub fn clear_all_icon_cache_dir(&self) -> anyhow::Result<()> {
        let cache_dir = self.dirs.icon_cache_dir();
        std::fs::create_dir_all(&cache_dir)?;

        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)?;
        }

        Ok(())
    }

    pub fn clear_plugin_icon_cache_dir(&self, plugin_uuid: &str) -> anyhow::Result<()> {
        let cache_dir = self.dirs.icon_cache_dir();
        let plugin_cache_dir = cache_dir.join(plugin_uuid);

        if plugin_cache_dir.exists() {
            std::fs::remove_dir_all(&plugin_cache_dir)?;
        }

        Ok(())
    }

    pub fn save_entrypoint_icon_to_cache(&self, plugin_uuid: &str, entrypoint_uuid: &str, data: impl AsRef<[u8]>) -> anyhow::Result<String> {
        let cache_dir = self.dirs.icon_cache_dir();
        let plugin_cache_dir = cache_dir.join(plugin_uuid);
        std::fs::create_dir_all(&plugin_cache_dir)?;

        let path_to_icon = plugin_cache_dir.join(format!("{}.png", &entrypoint_uuid));

        std::fs::write(&path_to_icon, data).expect(&format!("unable to create icon file {:?}", &path_to_icon));

        let path_to_icon = path_to_icon.to_str()
            .ok_or(anyhow!("unable to convert {:?} to utf-8 while saving icon to cache", &path_to_icon))?;

        Ok(path_to_icon.to_string())
    }
}


