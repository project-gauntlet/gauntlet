use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{anyhow, Context};
use serde::Deserialize;

use common::model::PluginId;
use crate::dbus::DbusManagementServer;
use crate::plugins::data_db_repository::{Code, DataDbRepository, SavePlugin, SavePluginEntrypoint, PluginPermissions};

pub struct PluginLoader {
    db_repository: DataDbRepository,
}

impl PluginLoader {
    pub fn new(db_repository: DataDbRepository) -> Self {
        Self {
            db_repository
        }
    }

    pub async fn download_and_add_plugin(
        &self,
        signal_context: zbus::SignalContext<'_>,
        plugin_id: PluginId
    ) -> anyhow::Result<()> {
        let data_db_repository = self.db_repository.clone();
        let signal_context = signal_context.to_owned();
        let handle = tokio::runtime::Handle::current();

        thread::spawn(move || {
            handle.block_on(async move {
                let temp_dir = tempfile::tempdir()?;

                let plugin_dir = PluginLoader::download(temp_dir.path(), plugin_id.clone())?;

                let plugin_data = PluginLoader::read_plugin_dir(plugin_dir, plugin_id.clone())
                    .await?;

                DbusManagementServer::remote_plugin_download_finished_signal(&signal_context, &plugin_id.to_string())
                    .await?;

                data_db_repository.save_plugin(SavePlugin {
                    id: plugin_data.id,
                    name: plugin_data.name,
                    code: plugin_data.code,
                    entrypoints: plugin_data.entrypoints,
                    permissions: plugin_data.permissions,
                    from_config: false,
                }).await?;

                anyhow::Ok(())
            }).expect("error when downloading and adding plugin");
        });

        Ok(())
    }

    pub async fn add_local_plugin(&self, plugin_id: PluginId, overwrite: bool) -> anyhow::Result<()> {
        let plugin_dir = plugin_id.try_to_path()?;

        let plugin_data = PluginLoader::read_plugin_dir(plugin_dir, plugin_id.clone())
            .await?;

        if overwrite {
            self.db_repository.remove_plugin(&plugin_data.id).await?
        }

        self.db_repository.save_plugin(SavePlugin {
            id: plugin_data.id,
            name: plugin_data.name,
            code: plugin_data.code,
            entrypoints: plugin_data.entrypoints,
            permissions: plugin_data.permissions,
            from_config: false,
        }).await?;

        Ok(())
    }

    fn download(target_dir: &Path, plugin_id: PluginId) -> anyhow::Result<PathBuf> {
        let url = plugin_id.try_to_git_url()?;

        let mut prepare_fetch = gix::clone::PrepareFetch::new(url, &target_dir, gix::create::Kind::WithWorktree, Default::default(), Default::default())?
            .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(1.try_into().unwrap()))
            .configure_remote(|mut remote| {
                remote.replace_refspecs(
                    Some("+refs/heads/gauntlet/releases:refs/remotes/origin/gauntlet/releases"),
                    gix::remote::Direction::Fetch,
                )?;

                Ok(remote)
            });

        let (mut prepare_checkout, _) = prepare_fetch.fetch_then_checkout(
            gix::progress::Discard,
            &gix::interrupt::IS_INTERRUPTED,
        )?;

        let (_repo, _) = prepare_checkout.main_worktree(
            gix::progress::Discard,
            &gix::interrupt::IS_INTERRUPTED,
        )?;

        let plugins_path = target_dir.join("plugins");

        let mut latest_version = None;

        for entry in std::fs::read_dir(plugins_path.clone())? {
            let entry = entry?;
            let version = entry.file_name()
                .into_string()
                .map_err(|os_str| anyhow!("\"{:?}\" is not a valid utf-8", os_str))?
                .replace("v", "")
                .parse::<u32>()?;

            latest_version = latest_version.max(Some(version));
        }

        let latest_version = latest_version.ok_or_else(|| anyhow!("Repository contains no versions"))?;

        let version_path = plugins_path.join(format!("v{}", latest_version));

        Ok(version_path)
    }

    async fn read_plugin_dir(plugin_dir: PathBuf, plugin_id: PluginId) -> anyhow::Result<PluginDirData> {
        let js_dir = plugin_dir.join("js");

        let js_dir_context = js_dir.display().to_string();
        let js_files = std::fs::read_dir(js_dir).context(js_dir_context)?;

        let js: HashMap<_, _> = js_files.into_iter()
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .into_iter()
            .map(|dist_path| dist_path.path())
            .filter(|dist_path| dist_path.extension() == Some(OsStr::new("js")))
            .map(|dist_path| {
                let js_content = std::fs::read_to_string(&dist_path)?;
                let id = dist_path.file_stem()
                    .expect("file returned from read_dir doesn't have filename?")
                    .to_str()
                    .ok_or(anyhow!("filename is not a valid utf-8"))?
                    .to_owned();

                Ok((id, js_content))
            })
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .collect();

        let config_path = plugin_dir.join("gauntlet.toml");
        let config_path_context = config_path.display().to_string();
        let config_content = std::fs::read_to_string(config_path).context(config_path_context)?;
        let config: PluginConfig = toml::from_str(&config_content)?;

        let plugin_name = config.metadata.name;

        let entrypoints: Vec<_> = config.entrypoints
            .into_iter()
            .map(|entrypoint| SavePluginEntrypoint {
                id: entrypoint.id,
                name: entrypoint.name,
            })
            .collect();

        let permissions = PluginPermissions {
            environment: config.permissions.environment,
            high_resolution_time: config.permissions.high_resolution_time,
            network: config.permissions.network,
            ffi: config.permissions.ffi,
            fs_read_access: config.permissions.fs_read_access,
            fs_write_access: config.permissions.fs_write_access,
            run_subprocess: config.permissions.run_subprocess,
            system: config.permissions.system,
        };

        Ok(PluginDirData {
            id: plugin_id.to_string(),
            name: plugin_name,
            code: Code {
                js
            },
            entrypoints,
            permissions
        })
    }
}

struct PluginDirData {
    pub id: String,
    pub name: String,
    pub code: Code,
    pub entrypoints: Vec<SavePluginEntrypoint>,
    pub permissions: PluginPermissions,
}

#[derive(Debug, Deserialize)]
struct PluginConfig {
    metadata: PluginConfigMetadata,
    entrypoints: Vec<PluginConfigEntrypoint>,
    #[serde(default)]
    permissions: PluginConfigPermissions,
}

#[derive(Debug, Deserialize)]
struct PluginConfigEntrypoint {
    id: String,
    name: String,
    #[allow(unused)] // used when building plugin
    path: String,
}

#[derive(Debug, Deserialize)]
struct PluginConfigMetadata {
    name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct PluginConfigPermissions {
    #[serde(default)]
    environment: Vec<String>,
    #[serde(default)]
    high_resolution_time: bool,
    #[serde(default)]
    network: Vec<String>,
    #[serde(default)]
    ffi: Vec<PathBuf>,
    #[serde(default)]
    fs_read_access: Vec<PathBuf>,
    #[serde(default)]
    fs_write_access: Vec<PathBuf>,
    #[serde(default)]
    run_subprocess: Vec<String>,
    #[serde(default)]
    system: Vec<String>,
}
