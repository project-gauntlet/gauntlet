use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

use anyhow::{anyhow, Context};
use include_dir::Dir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use walkdir::WalkDir;

use common::model::{DownloadStatus, PluginId};

use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_to_str, db_plugin_type_to_str, DbCode, DbPluginAction, DbPluginActionShortcutKind, DbPluginEntrypointType, DbPluginPermissions, DbPluginPreference, DbPluginPreferenceUserData, DbPluginType, DbPreferenceEnumValue, DbWritePlugin, DbWritePluginAssetData, DbWritePluginEntrypoint};
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

        let plugin_id_clone = plugin_id.clone();
        thread::spawn(move || {
            let result = handle.block_on(async move {
                let temp_dir = tempfile::tempdir()?;

                PluginLoader::download(temp_dir.path(), plugin_id_clone.clone())?;

                let plugin_data = PluginLoader::read_plugin_dir(temp_dir.path(), plugin_id_clone.clone())
                    .await?;

                data_db_repository.save_plugin(DbWritePlugin {
                    id: plugin_data.id,
                    uuid: Uuid::new_v4().to_string(),
                    name: plugin_data.name,
                    description: plugin_data.description,
                    enabled: true,
                    code: plugin_data.code,
                    entrypoints: plugin_data.entrypoints,
                    asset_data: plugin_data.asset_data,
                    permissions: plugin_data.permissions,
                    plugin_type: db_plugin_type_to_str(DbPluginType::Normal).to_owned(),
                    preferences: plugin_data.preferences,
                }).await?;

                anyhow::Ok(())
            });

            handle.block_on(async move {
                match result {
                    Ok(()) => {
                        tracing::info!("Finished download of plugin: {:?}", plugin_id);
                        download_status_guard.download_finished()
                    },
                    Err(err) => {
                        tracing::warn!("Download of plugin {:?} returned an error {:?}", plugin_id, err);
                        download_status_guard.download_failed(format!("{:?}", err))
                    }
                }
            })
        });

        Ok(())
    }

    pub async fn save_local_plugin(&self, path: &str) -> anyhow::Result<PluginId> {
        let plugin_id = PluginId::from_string(format!("file://{path}"));
        let plugin_dir = plugin_id.try_to_path()?;

        let plugin_data = PluginLoader::read_plugin_dir(plugin_dir.as_path(), plugin_id.clone())
            .await
            .context("Unable to read plugin directory")?;

        self.db_repository.save_plugin(DbWritePlugin {
            id: plugin_data.id,
            uuid: Uuid::new_v4().to_string(),
            name: plugin_data.name,
            description: plugin_data.description,
            enabled: true,
            code: plugin_data.code,
            entrypoints: plugin_data.entrypoints,
            asset_data: plugin_data.asset_data,
            permissions: plugin_data.permissions,
            plugin_type: db_plugin_type_to_str(DbPluginType::Normal).to_owned(),
            preferences: plugin_data.preferences,
        }).await?;

        Ok(plugin_id)
    }

    pub async fn save_builtin_plugin(&self, id: &str, dir: &Dir<'_>) -> anyhow::Result<PluginId> {
        let plugin_id = PluginId::from_string(format!("builtin://{id}"));
        let temp_dir = tempfile::tempdir()?;

        dir.extract(&temp_dir)?;

        let plugin_data = PluginLoader::read_plugin_dir(temp_dir.path(), plugin_id.clone())
            .await
            .context("Unable to read plugin directory")?;

        self.db_repository.save_plugin(DbWritePlugin {
            id: plugin_data.id,
            uuid: Uuid::new_v4().to_string(),
            name: plugin_data.name,
            description: plugin_data.description,
            enabled: true,
            code: plugin_data.code,
            entrypoints: plugin_data.entrypoints,
            asset_data: plugin_data.asset_data,
            permissions: plugin_data.permissions,
            plugin_type: db_plugin_type_to_str(DbPluginType::Bundled).to_owned(),
            preferences: plugin_data.preferences,
        }).await?;

        Ok(plugin_id)
    }

    fn download(target_dir: &Path, plugin_id: PluginId) -> anyhow::Result<()> {
        let url = plugin_id.try_to_git_url()?;

        let _ = git2::build::RepoBuilder::new()
            .branch("gauntlet/release")
            .clone(&url, target_dir)?;

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
            .or_else(|err| match err.io_error() {
                Some(err) if matches!(err.kind(), ErrorKind::NotFound) => Ok(vec![]),
                _ => Err(err),
            })
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
                uuid: Uuid::new_v4().to_string(),
                name: entrypoint.name,
                description: entrypoint.description,
                icon_path: entrypoint.icon,
                entrypoint_type: db_entrypoint_to_str(match entrypoint.entrypoint_type {
                    PluginManifestEntrypointTypes::Command => DbPluginEntrypointType::Command,
                    PluginManifestEntrypointTypes::View => DbPluginEntrypointType::View,
                    PluginManifestEntrypointTypes::InlineView => DbPluginEntrypointType::InlineView,
                    PluginManifestEntrypointTypes::CommandGenerator => DbPluginEntrypointType::CommandGenerator,
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
                actions: entrypoint.actions.into_iter()
                    .map(|action| DbPluginAction {
                        id: action.id,
                        description: action.description,
                        key: match action.shortcut.key {
                            PluginManifestActionShortcutKey::Num0 => "0".to_owned(),
                            PluginManifestActionShortcutKey::Num1 => "1".to_owned(),
                            PluginManifestActionShortcutKey::Num2 => "2".to_owned(),
                            PluginManifestActionShortcutKey::Num3 => "3".to_owned(),
                            PluginManifestActionShortcutKey::Num4 => "4".to_owned(),
                            PluginManifestActionShortcutKey::Num5 => "5".to_owned(),
                            PluginManifestActionShortcutKey::Num6 => "6".to_owned(),
                            PluginManifestActionShortcutKey::Num7 => "7".to_owned(),
                            PluginManifestActionShortcutKey::Num8 => "8".to_owned(),
                            PluginManifestActionShortcutKey::Num9 => "9".to_owned(),
                            PluginManifestActionShortcutKey::Exclamation => "!".to_owned(),
                            PluginManifestActionShortcutKey::AtSign => "@".to_owned(),
                            PluginManifestActionShortcutKey::Hash => "#".to_owned(),
                            PluginManifestActionShortcutKey::Dollar => "$".to_owned(),
                            PluginManifestActionShortcutKey::Percent => "%".to_owned(),
                            PluginManifestActionShortcutKey::Caret => "^".to_owned(),
                            PluginManifestActionShortcutKey::Ampersand => "&".to_owned(),
                            PluginManifestActionShortcutKey::Star => "*".to_owned(),
                            PluginManifestActionShortcutKey::LeftParenthesis => "(".to_owned(),
                            PluginManifestActionShortcutKey::RightParenthesis => ")".to_owned(),
                            PluginManifestActionShortcutKey::LowerA => "a".to_owned(),
                            PluginManifestActionShortcutKey::LowerB => "b".to_owned(),
                            PluginManifestActionShortcutKey::LowerC => "c".to_owned(),
                            PluginManifestActionShortcutKey::LowerD => "d".to_owned(),
                            PluginManifestActionShortcutKey::LowerE => "e".to_owned(),
                            PluginManifestActionShortcutKey::LowerF => "f".to_owned(),
                            PluginManifestActionShortcutKey::LowerG => "g".to_owned(),
                            PluginManifestActionShortcutKey::LowerH => "h".to_owned(),
                            PluginManifestActionShortcutKey::LowerI => "i".to_owned(),
                            PluginManifestActionShortcutKey::LowerJ => "j".to_owned(),
                            PluginManifestActionShortcutKey::LowerK => "k".to_owned(),
                            PluginManifestActionShortcutKey::LowerL => "l".to_owned(),
                            PluginManifestActionShortcutKey::LowerM => "m".to_owned(),
                            PluginManifestActionShortcutKey::LowerN => "n".to_owned(),
                            PluginManifestActionShortcutKey::LowerO => "o".to_owned(),
                            PluginManifestActionShortcutKey::LowerP => "p".to_owned(),
                            PluginManifestActionShortcutKey::LowerQ => "q".to_owned(),
                            PluginManifestActionShortcutKey::LowerR => "r".to_owned(),
                            PluginManifestActionShortcutKey::LowerS => "s".to_owned(),
                            PluginManifestActionShortcutKey::LowerT => "t".to_owned(),
                            PluginManifestActionShortcutKey::LowerU => "u".to_owned(),
                            PluginManifestActionShortcutKey::LowerV => "v".to_owned(),
                            PluginManifestActionShortcutKey::LowerW => "w".to_owned(),
                            PluginManifestActionShortcutKey::LowerX => "x".to_owned(),
                            PluginManifestActionShortcutKey::LowerY => "y".to_owned(),
                            PluginManifestActionShortcutKey::LowerZ => "z".to_owned(),
                            PluginManifestActionShortcutKey::UpperA => "A".to_owned(),
                            PluginManifestActionShortcutKey::UpperB => "B".to_owned(),
                            PluginManifestActionShortcutKey::UpperC => "C".to_owned(),
                            PluginManifestActionShortcutKey::UpperD => "D".to_owned(),
                            PluginManifestActionShortcutKey::UpperE => "E".to_owned(),
                            PluginManifestActionShortcutKey::UpperF => "F".to_owned(),
                            PluginManifestActionShortcutKey::UpperG => "G".to_owned(),
                            PluginManifestActionShortcutKey::UpperH => "H".to_owned(),
                            PluginManifestActionShortcutKey::UpperI => "I".to_owned(),
                            PluginManifestActionShortcutKey::UpperJ => "J".to_owned(),
                            PluginManifestActionShortcutKey::UpperK => "K".to_owned(),
                            PluginManifestActionShortcutKey::UpperL => "L".to_owned(),
                            PluginManifestActionShortcutKey::UpperM => "M".to_owned(),
                            PluginManifestActionShortcutKey::UpperN => "N".to_owned(),
                            PluginManifestActionShortcutKey::UpperO => "O".to_owned(),
                            PluginManifestActionShortcutKey::UpperP => "P".to_owned(),
                            PluginManifestActionShortcutKey::UpperQ => "Q".to_owned(),
                            PluginManifestActionShortcutKey::UpperR => "R".to_owned(),
                            PluginManifestActionShortcutKey::UpperS => "S".to_owned(),
                            PluginManifestActionShortcutKey::UpperT => "T".to_owned(),
                            PluginManifestActionShortcutKey::UpperU => "U".to_owned(),
                            PluginManifestActionShortcutKey::UpperV => "V".to_owned(),
                            PluginManifestActionShortcutKey::UpperW => "W".to_owned(),
                            PluginManifestActionShortcutKey::UpperX => "X".to_owned(),
                            PluginManifestActionShortcutKey::UpperY => "Y".to_owned(),
                            PluginManifestActionShortcutKey::UpperZ => "Z".to_owned(),
                            PluginManifestActionShortcutKey::Minus => "-".to_owned(),
                            PluginManifestActionShortcutKey::Equals => "=".to_owned(),
                            PluginManifestActionShortcutKey::Comma => ",".to_owned(),
                            PluginManifestActionShortcutKey::Dot => ".".to_owned(),
                            PluginManifestActionShortcutKey::Slash => "/".to_owned(),
                            PluginManifestActionShortcutKey::OpenSquareBracket => "[".to_owned(),
                            PluginManifestActionShortcutKey::CloseSquareBracket => "]".to_owned(),
                            PluginManifestActionShortcutKey::Semicolon => ";".to_owned(),
                            PluginManifestActionShortcutKey::Quote => "'".to_owned(),
                            PluginManifestActionShortcutKey::Backslash => "\"".to_owned(),
                            PluginManifestActionShortcutKey::Underscore => "_".to_owned(),
                            PluginManifestActionShortcutKey::Plus => "+".to_owned(),
                            PluginManifestActionShortcutKey::LessThan => "<".to_owned(),
                            PluginManifestActionShortcutKey::GreaterThan => ">".to_owned(),
                            PluginManifestActionShortcutKey::QuestionMark => "?".to_owned(),
                            PluginManifestActionShortcutKey::LeftBrace =>"{".to_owned(),
                            PluginManifestActionShortcutKey::RightBrace => "}".to_owned(),
                            PluginManifestActionShortcutKey::Colon => ":".to_owned(),
                            PluginManifestActionShortcutKey::DoubleQuotes => "\"".to_owned(),
                            PluginManifestActionShortcutKey::Pipe => "|".to_owned(),
                        },
                        kind: match action.shortcut.kind {
                            PluginManifestActionShortcutKind::Main => DbPluginActionShortcutKind::Main,
                            PluginManifestActionShortcutKind::Alternative => DbPluginActionShortcutKind::Alternative,
                        },
                    })
                    .collect(),
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
    icon: Option<String>,
    #[serde(rename = "type")]
    entrypoint_type: PluginManifestEntrypointTypes,
    #[serde(default)]
    preferences: Vec<PluginManifestPreference>,
    #[serde(default)]
    actions: Vec<PluginManifestAction>,
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
    #[serde(rename = "command-generator")]
    CommandGenerator,
}

#[derive(Debug, Deserialize)]
pub struct PluginManifestAction {
    id: String,
    description: String,
    shortcut: PluginManifestActionShortcut
}

#[derive(Debug, Deserialize)]
pub struct PluginManifestActionShortcut {
    key: PluginManifestActionShortcutKey,
    kind: PluginManifestActionShortcutKind,
}

// only stuff that is present on 60% keyboard
#[derive(Debug, Deserialize)]
pub enum PluginManifestActionShortcutKey {
    #[serde(rename = "0")]
    Num0,
    #[serde(rename = "1")]
    Num1,
    #[serde(rename = "2")]
    Num2,
    #[serde(rename = "3")]
    Num3,
    #[serde(rename = "4")]
    Num4,
    #[serde(rename = "5")]
    Num5,
    #[serde(rename = "6")]
    Num6,
    #[serde(rename = "7")]
    Num7,
    #[serde(rename = "8")]
    Num8,
    #[serde(rename = "9")]
    Num9,

    #[serde(rename = "!")]
    Exclamation,
    #[serde(rename = "@")]
    AtSign,
    #[serde(rename = "#")]
    Hash,
    #[serde(rename = "$")]
    Dollar,
    #[serde(rename = "%")]
    Percent,
    #[serde(rename = "^")]
    Caret,
    #[serde(rename = "&")]
    Ampersand,
    #[serde(rename = "*")]
    Star,
    #[serde(rename = "(")]
    LeftParenthesis,
    #[serde(rename = ")")]
    RightParenthesis,

    #[serde(rename = "a")]
    LowerA,
    #[serde(rename = "b")]
    LowerB,
    #[serde(rename = "c")]
    LowerC,
    #[serde(rename = "d")]
    LowerD,
    #[serde(rename = "e")]
    LowerE,
    #[serde(rename = "f")]
    LowerF,
    #[serde(rename = "g")]
    LowerG,
    #[serde(rename = "h")]
    LowerH,
    #[serde(rename = "i")]
    LowerI,
    #[serde(rename = "j")]
    LowerJ,
    #[serde(rename = "k")]
    LowerK,
    #[serde(rename = "l")]
    LowerL,
    #[serde(rename = "m")]
    LowerM,
    #[serde(rename = "n")]
    LowerN,
    #[serde(rename = "o")]
    LowerO,
    #[serde(rename = "p")]
    LowerP,
    #[serde(rename = "q")]
    LowerQ,
    #[serde(rename = "r")]
    LowerR,
    #[serde(rename = "s")]
    LowerS,
    #[serde(rename = "t")]
    LowerT,
    #[serde(rename = "u")]
    LowerU,
    #[serde(rename = "v")]
    LowerV,
    #[serde(rename = "w")]
    LowerW,
    #[serde(rename = "x")]
    LowerX,
    #[serde(rename = "y")]
    LowerY,
    #[serde(rename = "z")]
    LowerZ,

    #[serde(rename = "A")]
    UpperA,
    #[serde(rename = "B")]
    UpperB,
    #[serde(rename = "C")]
    UpperC,
    #[serde(rename = "D")]
    UpperD,
    #[serde(rename = "E")]
    UpperE,
    #[serde(rename = "F")]
    UpperF,
    #[serde(rename = "G")]
    UpperG,
    #[serde(rename = "H")]
    UpperH,
    #[serde(rename = "I")]
    UpperI,
    #[serde(rename = "J")]
    UpperJ,
    #[serde(rename = "K")]
    UpperK,
    #[serde(rename = "L")]
    UpperL,
    #[serde(rename = "M")]
    UpperM,
    #[serde(rename = "N")]
    UpperN,
    #[serde(rename = "O")]
    UpperO,
    #[serde(rename = "P")]
    UpperP,
    #[serde(rename = "Q")]
    UpperQ,
    #[serde(rename = "R")]
    UpperR,
    #[serde(rename = "S")]
    UpperS,
    #[serde(rename = "T")]
    UpperT,
    #[serde(rename = "U")]
    UpperU,
    #[serde(rename = "V")]
    UpperV,
    #[serde(rename = "W")]
    UpperW,
    #[serde(rename = "X")]
    UpperX,
    #[serde(rename = "Y")]
    UpperY,
    #[serde(rename = "Z")]
    UpperZ,

    #[serde(rename = "-")]
    Minus,
    #[serde(rename = "=")]
    Equals,
    #[serde(rename = ",")]
    Comma,
    #[serde(rename = ".")]
    Dot,
    #[serde(rename = "/")]
    Slash,
    #[serde(rename = "[")]
    OpenSquareBracket,
    #[serde(rename = "]")]
    CloseSquareBracket,
    #[serde(rename = ";")]
    Semicolon,
    #[serde(rename = "'")]
    Quote,
    #[serde(rename = "\\")]
    Backslash,

    #[serde(rename = "_")]
    Underscore,
    #[serde(rename = "+")]
    Plus,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "?")]
    QuestionMark,
    #[serde(rename = "{")]
    LeftBrace,
    #[serde(rename = "}")]
    RightBrace,
    #[serde(rename = ":")]
    Colon,
    #[serde(rename = "\"")]
    DoubleQuotes,
    #[serde(rename = "|")]
    Pipe,
}

#[derive(Debug, Deserialize)]
pub enum PluginManifestActionShortcutKind {
    #[serde(rename = "main")]
    Main,
    #[serde(rename = "alternative")]
    Alternative,
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
