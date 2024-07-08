use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::io::{ErrorKind};
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{anyhow, Context};
use include_dir::Dir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use walkdir::WalkDir;

use common::model::{DownloadStatus, PluginId};
use crate::model::ActionShortcutKey;
use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_to_str, db_plugin_type_to_str, DbCode, DbPluginAction, DbPluginActionShortcutKind, DbPluginEntrypointType, DbPluginPermissions, DbPluginPreference, DbPluginPreferenceUserData, DbPluginType, DbPreferenceEnumValue, DbWritePlugin, DbWritePluginAssetData, DbWritePluginEntrypoint};
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

    pub fn download_status(&self) -> HashMap<PluginId, DownloadStatus> {
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
                    enabled: false,
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
                        key: action.shortcut.key.to_model().to_value(),
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

impl PluginManifestActionShortcutKey {
    pub fn to_model(self) -> ActionShortcutKey {
        match self {
            PluginManifestActionShortcutKey::Num0 => ActionShortcutKey::Num0,
            PluginManifestActionShortcutKey::Num1 => ActionShortcutKey::Num1,
            PluginManifestActionShortcutKey::Num2 => ActionShortcutKey::Num2,
            PluginManifestActionShortcutKey::Num3 => ActionShortcutKey::Num3,
            PluginManifestActionShortcutKey::Num4 => ActionShortcutKey::Num4,
            PluginManifestActionShortcutKey::Num5 => ActionShortcutKey::Num5,
            PluginManifestActionShortcutKey::Num6 => ActionShortcutKey::Num6,
            PluginManifestActionShortcutKey::Num7 => ActionShortcutKey::Num7,
            PluginManifestActionShortcutKey::Num8 => ActionShortcutKey::Num8,
            PluginManifestActionShortcutKey::Num9 => ActionShortcutKey::Num9,
            PluginManifestActionShortcutKey::Exclamation => ActionShortcutKey::Exclamation,
            PluginManifestActionShortcutKey::AtSign => ActionShortcutKey::AtSign,
            PluginManifestActionShortcutKey::Hash => ActionShortcutKey::Hash,
            PluginManifestActionShortcutKey::Dollar => ActionShortcutKey::Dollar,
            PluginManifestActionShortcutKey::Percent => ActionShortcutKey::Percent,
            PluginManifestActionShortcutKey::Caret => ActionShortcutKey::Caret,
            PluginManifestActionShortcutKey::Ampersand => ActionShortcutKey::Ampersand,
            PluginManifestActionShortcutKey::Star => ActionShortcutKey::Star,
            PluginManifestActionShortcutKey::LeftParenthesis => ActionShortcutKey::LeftParenthesis,
            PluginManifestActionShortcutKey::RightParenthesis => ActionShortcutKey::RightParenthesis,
            PluginManifestActionShortcutKey::LowerA => ActionShortcutKey::LowerA,
            PluginManifestActionShortcutKey::LowerB => ActionShortcutKey::LowerB,
            PluginManifestActionShortcutKey::LowerC => ActionShortcutKey::LowerC,
            PluginManifestActionShortcutKey::LowerD => ActionShortcutKey::LowerD,
            PluginManifestActionShortcutKey::LowerE => ActionShortcutKey::LowerE,
            PluginManifestActionShortcutKey::LowerF => ActionShortcutKey::LowerF,
            PluginManifestActionShortcutKey::LowerG => ActionShortcutKey::LowerG,
            PluginManifestActionShortcutKey::LowerH => ActionShortcutKey::LowerH,
            PluginManifestActionShortcutKey::LowerI => ActionShortcutKey::LowerI,
            PluginManifestActionShortcutKey::LowerJ => ActionShortcutKey::LowerJ,
            PluginManifestActionShortcutKey::LowerK => ActionShortcutKey::LowerK,
            PluginManifestActionShortcutKey::LowerL => ActionShortcutKey::LowerL,
            PluginManifestActionShortcutKey::LowerM => ActionShortcutKey::LowerM,
            PluginManifestActionShortcutKey::LowerN => ActionShortcutKey::LowerN,
            PluginManifestActionShortcutKey::LowerO => ActionShortcutKey::LowerO,
            PluginManifestActionShortcutKey::LowerP => ActionShortcutKey::LowerP,
            PluginManifestActionShortcutKey::LowerQ => ActionShortcutKey::LowerQ,
            PluginManifestActionShortcutKey::LowerR => ActionShortcutKey::LowerR,
            PluginManifestActionShortcutKey::LowerS => ActionShortcutKey::LowerS,
            PluginManifestActionShortcutKey::LowerT => ActionShortcutKey::LowerT,
            PluginManifestActionShortcutKey::LowerU => ActionShortcutKey::LowerU,
            PluginManifestActionShortcutKey::LowerV => ActionShortcutKey::LowerV,
            PluginManifestActionShortcutKey::LowerW => ActionShortcutKey::LowerW,
            PluginManifestActionShortcutKey::LowerX => ActionShortcutKey::LowerX,
            PluginManifestActionShortcutKey::LowerY => ActionShortcutKey::LowerY,
            PluginManifestActionShortcutKey::LowerZ => ActionShortcutKey::LowerZ,
            PluginManifestActionShortcutKey::UpperA => ActionShortcutKey::UpperA,
            PluginManifestActionShortcutKey::UpperB => ActionShortcutKey::UpperB,
            PluginManifestActionShortcutKey::UpperC => ActionShortcutKey::UpperC,
            PluginManifestActionShortcutKey::UpperD => ActionShortcutKey::UpperD,
            PluginManifestActionShortcutKey::UpperE => ActionShortcutKey::UpperE,
            PluginManifestActionShortcutKey::UpperF => ActionShortcutKey::UpperF,
            PluginManifestActionShortcutKey::UpperG => ActionShortcutKey::UpperG,
            PluginManifestActionShortcutKey::UpperH => ActionShortcutKey::UpperH,
            PluginManifestActionShortcutKey::UpperI => ActionShortcutKey::UpperI,
            PluginManifestActionShortcutKey::UpperJ => ActionShortcutKey::UpperJ,
            PluginManifestActionShortcutKey::UpperK => ActionShortcutKey::UpperK,
            PluginManifestActionShortcutKey::UpperL => ActionShortcutKey::UpperL,
            PluginManifestActionShortcutKey::UpperM => ActionShortcutKey::UpperM,
            PluginManifestActionShortcutKey::UpperN => ActionShortcutKey::UpperN,
            PluginManifestActionShortcutKey::UpperO => ActionShortcutKey::UpperO,
            PluginManifestActionShortcutKey::UpperP => ActionShortcutKey::UpperP,
            PluginManifestActionShortcutKey::UpperQ => ActionShortcutKey::UpperQ,
            PluginManifestActionShortcutKey::UpperR => ActionShortcutKey::UpperR,
            PluginManifestActionShortcutKey::UpperS => ActionShortcutKey::UpperS,
            PluginManifestActionShortcutKey::UpperT => ActionShortcutKey::UpperT,
            PluginManifestActionShortcutKey::UpperU => ActionShortcutKey::UpperU,
            PluginManifestActionShortcutKey::UpperV => ActionShortcutKey::UpperV,
            PluginManifestActionShortcutKey::UpperW => ActionShortcutKey::UpperW,
            PluginManifestActionShortcutKey::UpperX => ActionShortcutKey::UpperX,
            PluginManifestActionShortcutKey::UpperY => ActionShortcutKey::UpperY,
            PluginManifestActionShortcutKey::UpperZ => ActionShortcutKey::UpperZ,
            PluginManifestActionShortcutKey::Minus => ActionShortcutKey::Minus,
            PluginManifestActionShortcutKey::Equals => ActionShortcutKey::Equals,
            PluginManifestActionShortcutKey::Comma => ActionShortcutKey::Comma,
            PluginManifestActionShortcutKey::Dot => ActionShortcutKey::Dot,
            PluginManifestActionShortcutKey::Slash => ActionShortcutKey::Slash,
            PluginManifestActionShortcutKey::OpenSquareBracket => ActionShortcutKey::OpenSquareBracket,
            PluginManifestActionShortcutKey::CloseSquareBracket => ActionShortcutKey::CloseSquareBracket,
            PluginManifestActionShortcutKey::Semicolon => ActionShortcutKey::Semicolon,
            PluginManifestActionShortcutKey::Quote => ActionShortcutKey::Quote,
            PluginManifestActionShortcutKey::Backslash => ActionShortcutKey::Backslash,
            PluginManifestActionShortcutKey::Underscore => ActionShortcutKey::Underscore,
            PluginManifestActionShortcutKey::Plus => ActionShortcutKey::Plus,
            PluginManifestActionShortcutKey::LessThan => ActionShortcutKey::LessThan,
            PluginManifestActionShortcutKey::GreaterThan => ActionShortcutKey::GreaterThan,
            PluginManifestActionShortcutKey::QuestionMark => ActionShortcutKey::QuestionMark,
            PluginManifestActionShortcutKey::LeftBrace => ActionShortcutKey::LeftBrace,
            PluginManifestActionShortcutKey::RightBrace => ActionShortcutKey::RightBrace,
            PluginManifestActionShortcutKey::Colon => ActionShortcutKey::Colon,
            PluginManifestActionShortcutKey::DoubleQuotes => ActionShortcutKey::DoubleQuotes,
            PluginManifestActionShortcutKey::Pipe => ActionShortcutKey::Pipe,
        }
    }
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
