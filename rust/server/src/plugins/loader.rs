use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

use anyhow::{anyhow, Context};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

use common::model::{DownloadStatus, PluginId};

use crate::model::{entrypoint_to_str, PluginEntrypointType};
use crate::plugins::data_db_repository::{DbCode, DataDbRepository, DbPluginPermissions, DbPluginPreference, DbPluginPreferenceUserData, DbWritePlugin, DbWritePluginEntrypoint, DbPreferenceEnumValue, DbWritePluginAssetData};
use crate::plugins::download_status::DownloadStatusHolder;
use crate::plugins::js::asset_data;

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

                data_db_repository.save_plugin(DbWritePlugin {
                    id: plugin_data.id,
                    name: plugin_data.name,
                    description: plugin_data.description,
                    enabled: false,
                    code: plugin_data.code,
                    entrypoints: plugin_data.entrypoints,
                    asset_data: plugin_data.asset_data,
                    permissions: plugin_data.permissions,
                    from_config: false,
                    preferences: plugin_data.preferences,
                    preferences_user_data: HashMap::new(),
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
            .await
            .context("Unable to read plugin directory")?;

        if overwrite {
            // TODO instead of overwrite just update the code and assets
            self.db_repository.remove_plugin(&plugin_data.id).await?
        }

        self.db_repository.save_plugin(DbWritePlugin {
            id: plugin_data.id,
            name: plugin_data.name,
            description: plugin_data.description,
            enabled: true,
            code: plugin_data.code,
            entrypoints: plugin_data.entrypoints,
            asset_data: plugin_data.asset_data,
            permissions: plugin_data.permissions,
            from_config: false,
            preferences: plugin_data.preferences,
            preferences_user_data: HashMap::new()
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

    async fn read_plugin_dir(plugin_dir: &Path, plugin_id: PluginId) -> anyhow::Result<PluginDownloadData> {
        let js_dir = plugin_dir.join("js");
        let assets = plugin_dir.join("assets");

        let js_dir_context = js_dir.display().to_string();
        let js_files = std::fs::read_dir(js_dir).context(js_dir_context)?;

        let js: HashMap<_, _> = js_files.into_iter()
            .collect::<std::io::Result<Vec<DirEntry>>>()
            .context("Unable to get list of plugin js files")?
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
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Unable to read plugin js data")?
            .into_iter()
            .collect();

        let asset_data = WalkDir::new(&assets)
            .into_iter()
            .collect::<walkdir::Result<Vec<walkdir::DirEntry>>>()
            .context("Unable to get list of plugin asset data files")?
            .into_iter()
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .map(|path| {
                let path = path.path();

                let data = std::fs::read(path)
                    .context(format!("Unable to read plugin asset file {:?}", path))?;

                let path = path
                    .strip_prefix(&assets)
                    .expect("assets is a base of dist_path")
                    .to_str()
                    .ok_or(anyhow!("filename is not a valid utf-8"))?
                    .to_owned();

                Ok(DbWritePluginAssetData {
                    path,
                    data,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Unable to read plugin asset data")?
            .into_iter()
            .collect();

        let plugin_manifest_path = plugin_dir.join("gauntlet.toml");
        let plugin_manifest_path_context = plugin_manifest_path.display().to_string();
        let plugin_manifest_content = std::fs::read_to_string(plugin_manifest_path).context(plugin_manifest_path_context)?;
        let plugin_manifest: PluginManifest = toml::from_str(&plugin_manifest_content)
            .context("Unable to read plugin manifest")?;

        tracing::debug!("Plugin config read: {:?}", plugin_manifest);

        let plugin_name = plugin_manifest.gauntlet.name;
        let plugin_description = plugin_manifest.gauntlet.description;

        let entrypoints: Vec<_> = plugin_manifest.entrypoint
            .into_iter()
            .map(|entrypoint| DbWritePluginEntrypoint {
                id: entrypoint.id,
                name: entrypoint.name,
                description: entrypoint.description,
                entrypoint_type: entrypoint_to_str(match entrypoint.entrypoint_type {
                    PluginManifestEntrypointTypes::Command => PluginEntrypointType::Command,
                    PluginManifestEntrypointTypes::View => PluginEntrypointType::View,
                    PluginManifestEntrypointTypes::InlineView => PluginEntrypointType::InlineView
                }).to_owned(),
                preferences: entrypoint.preferences
                    .into_iter()
                    .map(|preference| match preference {
                        PluginManifestPreference::Number { name, default, description } => (name, DbPluginPreference::Number { default, description }),
                        PluginManifestPreference::String { name, default, description } => (name, DbPluginPreference::String { default, description }),
                        PluginManifestPreference::Enum { name, default, description, enum_values } => {
                            let enum_values = enum_values.into_iter()
                                .map(|PluginManifestPreferenceEnumValue { label, value } | DbPreferenceEnumValue { label, value })
                                .collect();

                            (name, DbPluginPreference::Enum { default, description, enum_values })
                        },
                        PluginManifestPreference::Bool { name, default, description } => (name, DbPluginPreference::Bool { default, description }),
                        PluginManifestPreference::ListOfStrings { name, description } => (name, DbPluginPreference::ListOfStrings { default: None, description }),
                        PluginManifestPreference::ListOfNumbers { name, description } => (name, DbPluginPreference::ListOfNumbers { default: None, description }),
                        PluginManifestPreference::ListOfEnums { name, description, enum_values } => {
                            let enum_values = enum_values.into_iter()
                                .map(|PluginManifestPreferenceEnumValue { label, value } | DbPreferenceEnumValue { label, value })
                                .collect();

                            (name, DbPluginPreference::ListOfEnums { default: None, description, enum_values })
                        },
                    })
                    .collect(),
                preferences_user_data: HashMap::new(),
            })
            .collect();

        let plugin_preferences = plugin_manifest.preferences
            .into_iter()
            .map(|preference| match preference {
                PluginManifestPreference::Number { name, default, description } => (name, DbPluginPreference::Number { default, description }),
                PluginManifestPreference::String { name, default, description } => (name, DbPluginPreference::String { default, description }),
                PluginManifestPreference::Enum { name, default, description, enum_values } => {
                    let enum_values = enum_values.into_iter()
                        .map(|PluginManifestPreferenceEnumValue { label, value } | DbPreferenceEnumValue { label, value })
                        .collect();

                    (name, DbPluginPreference::Enum { default, description, enum_values })
                },
                PluginManifestPreference::Bool { name, default, description } => (name, DbPluginPreference::Bool { default, description }),
                PluginManifestPreference::ListOfStrings { name, description } => (name, DbPluginPreference::ListOfStrings { default: None, description }),
                PluginManifestPreference::ListOfNumbers { name, description } => (name, DbPluginPreference::ListOfNumbers { default: None, description }),
                PluginManifestPreference::ListOfEnums { name, description, enum_values } => {
                    let enum_values = enum_values.into_iter()
                        .map(|PluginManifestPreferenceEnumValue { label, value } | DbPreferenceEnumValue { label, value })
                        .collect();

                    (name, DbPluginPreference::ListOfEnums { default: None, description, enum_values })
                },
            })
            .collect();

        let permissions = DbPluginPermissions {
            environment: plugin_manifest.permissions.environment,
            high_resolution_time: plugin_manifest.permissions.high_resolution_time,
            network: plugin_manifest.permissions.network,
            ffi: plugin_manifest.permissions.ffi,
            fs_read_access: plugin_manifest.permissions.fs_read_access,
            fs_write_access: plugin_manifest.permissions.fs_write_access,
            run_subprocess: plugin_manifest.permissions.run_subprocess,
            system: plugin_manifest.permissions.system,
        };

        Ok(PluginDownloadData {
            id: plugin_id.to_string(),
            name: plugin_name,
            description: plugin_description,
            code: DbCode {
                js
            },
            entrypoints,
            asset_data,
            permissions,
            preferences: plugin_preferences,
            preferences_user_data: HashMap::new()
        })
    }
}

struct PluginDownloadData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub code: DbCode,
    pub entrypoints: Vec<DbWritePluginEntrypoint>,
    pub asset_data: Vec<DbWritePluginAssetData>,
    pub permissions: DbPluginPermissions,
    pub preferences: HashMap<String, DbPluginPreference>,
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    gauntlet: PluginManifestMetadata,
    entrypoint: Vec<PluginManifestEntrypoint>,
    #[serde(default)]
    supported_system: Vec<PluginManifestSupportedSystem>,
    #[serde(default)]
    permissions: PluginManifestPermissions,
    #[serde(default)]
    preferences: Vec<PluginManifestPreference>,
}

#[derive(Debug, Deserialize)]
struct PluginManifestEntrypoint {
    id: String,
    name: String,
    description: String,
    #[allow(unused)] // used when building plugin
    path: String,
    #[serde(rename = "type")]
    entrypoint_type: PluginManifestEntrypointTypes,
    #[serde(default)]
    preferences: Vec<PluginManifestPreference>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum PluginManifestPreference {
    #[serde(rename = "number")]
    Number {
        name: String,
        default: Option<f64>,
        description: String,
    },
    #[serde(rename = "string")]
    String {
        name: String,
        default: Option<String>,
        description: String,
    },
    #[serde(rename = "enum")]
    Enum {
        name: String,
        default: Option<String>,
        description: String,
        enum_values: Vec<PluginManifestPreferenceEnumValue>,
    },
    #[serde(rename = "bool")]
    Bool {
        name: String,
        default: Option<bool>,
        description: String,
    },
    #[serde(rename = "list_of_strings")]
    ListOfStrings {
        name: String,
        // default: Option<Vec<String>>,
        description: String,
    },
    #[serde(rename = "list_of_numbers")]
    ListOfNumbers {
        name: String,
        // default: Option<Vec<f64>>,
        description: String,
    },
    #[serde(rename = "list_of_enums")]
    ListOfEnums {
        name: String,
        // default: Option<Vec<String>>,
        enum_values: Vec<PluginManifestPreferenceEnumValue>,
        description: String,
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifestPreferenceEnumValue {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub enum PluginManifestEntrypointTypes {
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "view")]
    View,
    #[serde(rename = "inline-view")]
    InlineView,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "os")]
pub enum PluginManifestSupportedSystem {
    #[serde(rename = "linux")]
    Linux,
}

#[derive(Debug, Deserialize)]
struct PluginManifestMetadata {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct PluginManifestPermissions {
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
