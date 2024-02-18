use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

use anyhow::{anyhow, Context};
use serde::Deserialize;

use common::model::{DownloadStatus, PluginId};

use crate::model::{entrypoint_to_str, PluginEntrypointType};
use crate::plugins::data_db_repository::{Code, DataDbRepository, PluginPermissions, SavePlugin, SavePluginEntrypoint};
use crate::plugins::download_status::DownloadStatusHolder;

pub struct PluginLoader {
    db_repository: DataDbRepository,
    download_status_holder: DownloadStatusHolder
}

impl PluginLoader {
    pub fn new(db_repository: DataDbRepository) -> Self {
        Self {
            db_repository,
            download_status_holder: DownloadStatusHolder::new()
        }
    }

    pub fn download_status(&self) -> HashMap<String, DownloadStatus> {
        self.download_status_holder.download_status()
    }

    pub async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        let download_status_guard = self.download_status_holder.download_started(plugin_id.clone());

        let data_db_repository = self.db_repository.clone();
        let handle = tokio::runtime::Handle::current();

        thread::spawn(move || {
            let result = handle.block_on(async move {
                let temp_dir = tempfile::tempdir()?;

                PluginLoader::download(temp_dir.path(), plugin_id.clone())?;

                let plugin_data = PluginLoader::read_plugin_dir(temp_dir.path(), plugin_id.clone())
                    .await?;

                data_db_repository.save_plugin(SavePlugin {
                    id: plugin_data.id,
                    name: plugin_data.name,
                    enabled: false,
                    code: plugin_data.code,
                    entrypoints: plugin_data.entrypoints,
                    permissions: plugin_data.permissions,
                    from_config: false,
                }).await?;

                anyhow::Ok(())
            });

            match result {
                Ok(()) => download_status_guard.download_finished(),
                Err(err) => download_status_guard.download_failed(err.to_string())
            }
        });

        Ok(())
    }

    pub async fn save_local_plugin(&self, path: &str, overwrite: bool) -> anyhow::Result<PluginId> {
        let plugin_id = PluginId::from_string(format!("file://{path}"));
        let plugin_dir = plugin_id.try_to_path()?;

        let plugin_data = PluginLoader::read_plugin_dir(plugin_dir.as_path(), plugin_id.clone())
            .await?;

        if overwrite {
            self.db_repository.remove_plugin(&plugin_data.id).await?
        }

        self.db_repository.save_plugin(SavePlugin {
            id: plugin_data.id,
            name: plugin_data.name,
            enabled: true,
            code: plugin_data.code,
            entrypoints: plugin_data.entrypoints,
            permissions: plugin_data.permissions,
            from_config: false,
        }).await?;

        Ok(plugin_id)
    }

    fn download(target_dir: &Path, plugin_id: PluginId) -> anyhow::Result<()> {
        let url = plugin_id.try_to_git_url()?;

        let mut prepare_fetch = gix::clone::PrepareFetch::new(url, &target_dir, gix::create::Kind::WithWorktree, Default::default(), Default::default())?
            .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(1.try_into().unwrap()))
            .configure_remote(|mut remote| {
                remote.replace_refspecs(
                    Some("+refs/heads/gauntlet/release:refs/remotes/origin/gauntlet/release"),
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

        Ok(())
    }

    async fn read_plugin_dir(plugin_dir: &Path, plugin_id: PluginId) -> anyhow::Result<PluginDirData> {
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

        tracing::debug!("Plugin config read: {:?}", config);

        let plugin_name = config.gauntlet.name;

        let entrypoints: Vec<_> = config.entrypoint
            .into_iter()
            .map(|entrypoint| SavePluginEntrypoint {
                id: entrypoint.id,
                name: entrypoint.name,
                entrypoint_type: entrypoint_to_str(match entrypoint.entrypoint_type {
                    PluginConfigEntrypointTypes::Command => PluginEntrypointType::Command,
                    PluginConfigEntrypointTypes::View => PluginEntrypointType::View,
                    PluginConfigEntrypointTypes::InlineView => PluginEntrypointType::InlineView
                }).to_owned()
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
    gauntlet: PluginConfigMetadata,
    entrypoint: Vec<PluginConfigEntrypoint>,
    #[serde(default)]
    supported_system: Vec<PluginConfigSupportedSystem>,
    #[serde(default)]
    permissions: PluginConfigPermissions,
}

#[derive(Debug, Deserialize)]
struct PluginConfigEntrypoint {
    id: String,
    name: String,
    #[allow(unused)] // used when building plugin
    path: String,
    #[serde(rename = "type")]
    entrypoint_type: PluginConfigEntrypointTypes,
}

#[derive(Debug, Deserialize)]
pub enum PluginConfigEntrypointTypes {
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "view")]
    View,
    #[serde(rename = "inline-view")]
    InlineView,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "os")]
pub enum PluginConfigSupportedSystem {
    #[serde(rename = "linux")]
    Linux,
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
