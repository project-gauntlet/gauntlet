use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use futures::{StreamExt, TryStreamExt};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Executor, Pool, Row, Sqlite, SqlitePool};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::types::Json;
use typed_path::TypedPathBuf;
use uuid::Uuid;
use gauntlet_common::model::{UiTheme, PhysicalKey, PhysicalShortcut, PluginId};
use gauntlet_common::dirs::Dirs;
use crate::model::ActionShortcutKey;
use crate::plugins::frecency::{FrecencyItemStats, FrecencyMetaParams};
use crate::plugins::loader::PluginManifestActionShortcutKey;

static MIGRATOR: Migrator = sqlx::migrate!("./db_migrations");

#[derive(Clone)]
pub struct DataDbRepository {
    pool: Pool<Sqlite>,
}

#[derive(sqlx::FromRow)]
pub struct DbReadPlugin {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    #[sqlx(json)]
    pub code: DbCode,
    #[sqlx(json)]
    pub permissions: DbPluginPermissions,
    #[sqlx(rename = "type")]
    pub plugin_type: String,
    #[sqlx(json)]
    pub preferences: HashMap<String, DbPluginPreference>,
    #[sqlx(json)]
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
}

#[derive(sqlx::FromRow)]
pub struct DbReadPluginEntrypoint {
    pub id: String,
    pub uuid: String,
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub icon_path: Option<String>,
    #[sqlx(rename = "type")]
    pub entrypoint_type: String,
    #[sqlx(json)]
    pub preferences: HashMap<String, DbPluginPreference>,
    #[sqlx(json)]
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
    #[sqlx(json)]
    pub actions: Vec<DbPluginAction>,
    #[sqlx(json)]
    pub actions_user_data: Vec<DbPluginActionUserData>,
}

#[derive(Deserialize, Serialize)]
pub struct DbCode {
    pub js: HashMap<String, String>,
}

pub struct DbWritePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub code: DbCode,
    pub entrypoints: Vec<DbWritePluginEntrypoint>,
    pub asset_data: Vec<DbWritePluginAssetData>,
    pub permissions: DbPluginPermissions,
    pub plugin_type: String,
    pub preferences: HashMap<String, DbPluginPreference>,
}

pub struct DbWritePluginEntrypoint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_path: Option<String>,
    pub entrypoint_type: String,
    pub preferences: HashMap<String, DbPluginPreference>,
    pub actions: Vec<DbPluginAction>,
}

pub struct DbWritePluginAssetData {
    pub path: String,
    pub data: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum DbPluginEntrypointType {
    Command,
    View,
    InlineView,
    EntrypointGenerator,
}

#[derive(Debug, Clone)]
pub enum DbPluginType {
    Normal,
    Config,
    Bundled,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPluginPermissions {
    #[serde(default)]
    pub environment: Vec<String>,
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub filesystem: DbPluginPermissionsFileSystem,
    #[serde(default)]
    pub exec: DbPluginPermissionsExec,
    #[serde(default)]
    pub system: Vec<String>,
    #[serde(default)]
    pub clipboard: Vec<DbPluginClipboardPermissions>,
    #[serde(default)]
    pub main_search_bar: Vec<DbPluginMainSearchBarPermissions>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DbPluginPermissionsFileSystem {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DbPluginPermissionsExec {
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub executable: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DbPluginClipboardPermissions {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "clear")]
    Clear
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DbPluginMainSearchBarPermissions {
    #[serde(rename = "read")]
    Read,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DbPluginPreferenceUserData {
    #[serde(rename = "number")]
    Number {
        value: Option<f64>,
    },
    #[serde(rename = "string")]
    String {
        value: Option<String>,
    },
    #[serde(rename = "enum")]
    Enum {
        value: Option<String>,
    },
    #[serde(rename = "bool")]
    Bool {
        value: Option<bool>,
    },
    #[serde(rename = "list_of_strings")]
    ListOfStrings {
        value: Option<Vec<String>>,
    },
    #[serde(rename = "list_of_numbers")]
    ListOfNumbers {
        value: Option<Vec<f64>>,
    },
    #[serde(rename = "list_of_enums")]
    ListOfEnums {
        value: Option<Vec<String>>,
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPluginAction {
    pub id: String,
    pub description: String,
    pub key: String,
    pub kind: DbPluginActionShortcutKind
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPluginActionUserData {
    pub id: String,
    pub key: String,
    pub modifier_shift: bool,
    pub modifier_control: bool,
    pub modifier_alt: bool,
    pub modifier_meta: bool
}

#[derive(sqlx::FromRow)]
struct DbSettingsDataContainer {
    #[sqlx(json)]
    pub global_shortcut: DbSettingsGlobalShortcutData, // separate field because legacy
    // #[sqlx(json)] // https://github.com/launchbadge/sqlx/issues/2849
    pub settings: Option<Json<DbSettings>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbSettingsGlobalShortcutData {
    pub physical_key: String,
    pub modifier_shift: bool,
    pub modifier_control: bool,
    pub modifier_alt: bool,
    pub modifier_meta: bool,
    #[serde(default)]
    pub unset: bool,
    #[serde(default)]
    pub error: Option<String>
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DbSettings {
    // none means auto-detect
    pub theme: Option<DbTheme>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DbTheme {
    #[serde(rename = "macos_light")]
    MacOSLight,
    #[serde(rename = "macos_dark")]
    MacOSDark,
    #[serde(rename = "legacy")]
    Legacy
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DbPluginActionShortcutKind {
    #[serde(rename = "main")]
    Main,
    #[serde(rename = "alternative")]
    Alternative,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DbPluginPreference {
    #[serde(rename = "number")]
    Number {
        name: Option<String>, // optional for db backwards compatibility, in settings id will be shown
        default: Option<f64>,
        description: String,
    },
    #[serde(rename = "string")]
    String {
        name: Option<String>,
        default: Option<String>,
        description: String,
    },
    #[serde(rename = "enum")]
    Enum {
        name: Option<String>,
        default: Option<String>,
        description: String,
        enum_values: Vec<DbPreferenceEnumValue>,
    },
    #[serde(rename = "bool")]
    Bool {
        name: Option<String>,
        default: Option<bool>,
        description: String,
    },
    #[serde(rename = "list_of_strings")]
    ListOfStrings {
        name: Option<String>,
        default: Option<Vec<String>>,
        description: String,
    },
    #[serde(rename = "list_of_numbers")]
    ListOfNumbers {
        name: Option<String>,
        default: Option<Vec<f64>>,
        description: String,
    },
    #[serde(rename = "list_of_enums")]
    ListOfEnums {
        name: Option<String>,
        default: Option<Vec<String>>,
        enum_values: Vec<DbPreferenceEnumValue>,
        description: String,
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPreferenceEnumValue {
    pub label: String,
    pub value: String,
}


#[derive(sqlx::FromRow)]
pub struct DbReadPendingPlugin {
    pub id: String,
}

pub struct DbWritePendingPlugin {
    pub id: String,
}

#[derive(sqlx::FromRow)]
pub struct DbPluginEntrypointFrecencyStats {
    pub plugin_id: String,
    pub entrypoint_id: String,

    pub reference_time: f64,
    pub half_life: f64,
    pub last_accessed: f64,
    pub frecency: f64,
    pub num_accesses: i32,
}

const SETTINGS_DATA_ID: &str = "settings_data"; // only one row in the table

impl DataDbRepository {
    pub async fn new(dirs: Dirs) -> anyhow::Result<Self> {
        let data_db_file = dirs.data_db_file()?;

        std::fs::create_dir_all(&data_db_file.parent().unwrap())
            .context("Unable to create data directory")?;

        let conn = SqliteConnectOptions::new()
            .filename(data_db_file)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(conn)
            .await
            .context("Unable to open database connection")?;

        // TODO backup before migration? up to 5 backups?
        MIGRATOR.run(&pool)
            .await
            .context("Unable apply database migration")?;

        let db_repository = Self { pool };

        db_repository.apply_uuid_default_value().await?;
        db_repository.remove_legacy_bundled_plugins().await?;

        Ok(db_repository)
    }

    async fn apply_uuid_default_value(&self) -> anyhow::Result<()> {
        // language=SQLite
        let mut stream = self.pool.fetch(sqlx::query("SELECT id FROM plugin WHERE uuid IS NULL"));
        while let Some(row) = stream.next().await {
            let row = row?;
            let id: &str = row.get("id");

            // language=SQLite
            sqlx::query("UPDATE plugin SET uuid = ?1 WHERE id = ?2")
                .bind(Uuid::new_v4().to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        // language=SQLite
        let mut stream = self.pool.fetch(sqlx::query("SELECT id FROM plugin_entrypoint WHERE uuid IS NULL"));
        while let Some(row) = stream.next().await {
            let row = row?;
            let id: &str = row.get("id");

            // language=SQLite
            sqlx::query("UPDATE plugin_entrypoint SET uuid = ?1 WHERE id = ?2")
                .bind(Uuid::new_v4().to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    async fn remove_legacy_bundled_plugins(&self) -> anyhow::Result<()> {
        self.remove_plugin("builtin://applications").await?;
        self.remove_plugin("builtin://calculator").await?;
        self.remove_plugin("builtin://settings").await?;

        Ok(())
    }

    pub async fn list_plugins(&self) -> anyhow::Result<Vec<DbReadPlugin>> {
        // language=SQLite
        let plugins = sqlx::query_as::<_, DbReadPlugin>("SELECT * FROM plugin")
            .fetch_all(&self.pool)
            .await?;

        Ok(plugins)
    }

    pub async fn list_plugins_and_entrypoints(&self) -> anyhow::Result<Vec<(DbReadPlugin, Vec<DbReadPluginEntrypoint>)>> {
        // language=SQLite
        let plugins = self.list_plugins().await?;

        let result = futures::stream::iter(plugins)
            .then(|plugin| async move {
                let entrypoints = self.get_entrypoints_by_plugin_id(&plugin.id).await?;

                Ok::<(DbReadPlugin, Vec<DbReadPluginEntrypoint>), anyhow::Error>((plugin, entrypoints))
            })
            .try_collect::<Vec<(DbReadPlugin, Vec<DbReadPluginEntrypoint>)>>()
            .await?;

        Ok(result)
    }

    pub async fn get_plugin_by_id(&self, plugin_id: &str) -> anyhow::Result<DbReadPlugin> {
        self.get_plugin_by_id_with_executor(plugin_id, &self.pool).await
    }

    async fn get_plugin_by_id_with_executor<'a, E>(&self, plugin_id: &str, executor: E) -> anyhow::Result<DbReadPlugin>
        where
            E: Executor<'a, Database=Sqlite>,
    {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPlugin>("SELECT * FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_one(executor)
            .await?;

        Ok(result)
    }

    pub async fn get_plugin_by_id_option(&self, plugin_id: &str) -> anyhow::Result<Option<DbReadPlugin>> {
        self.get_plugin_by_id_option_with_executor(plugin_id, &self.pool).await
    }

    async fn get_plugin_by_id_option_with_executor<'a, E>(&self, plugin_id: &str, executor: E) -> anyhow::Result<Option<DbReadPlugin>>
        where
            E: Executor<'a, Database=Sqlite>,
    {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPlugin>("SELECT * FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_optional(executor)
            .await?;

        Ok(result)
    }

    pub async fn get_entrypoints_by_plugin_id(&self, plugin_id: &str) -> anyhow::Result<Vec<DbReadPluginEntrypoint>> {
        self.get_entrypoints_by_plugin_id_with_executor(plugin_id, &self.pool).await
    }

    async fn get_entrypoints_by_plugin_id_with_executor<'a, E>(&self, plugin_id: &str, executor: E) -> anyhow::Result<Vec<DbReadPluginEntrypoint>>
        where
            E: Executor<'a, Database=Sqlite>
    {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginEntrypoint>("SELECT * FROM plugin_entrypoint WHERE plugin_id = ?1")
            .bind(plugin_id)
            .fetch_all(executor)
            .await?;

        Ok(result)
    }

    pub async fn get_entrypoint_by_id(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<DbReadPluginEntrypoint> {
        self.get_entrypoint_by_id_with_executor(plugin_id, entrypoint_id, &self.pool).await
    }

    async fn get_entrypoint_by_id_with_executor<'a, E>(&self, plugin_id: &str, entrypoint_id: &str, executor: E) -> anyhow::Result<DbReadPluginEntrypoint>
        where
            E: Executor<'a, Database=Sqlite>,
    {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginEntrypoint>("SELECT * FROM plugin_entrypoint WHERE id = ?1 AND plugin_id = ?2")
            .bind(entrypoint_id)
            .bind(plugin_id)
            .fetch_one(executor)
            .await?;

        Ok(result)
    }

    pub async fn get_entrypoint_by_id_option(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<Option<DbReadPluginEntrypoint>> {
        self.get_entrypoint_by_id_option_with_executor(plugin_id, entrypoint_id, &self.pool).await
    }

    async fn get_entrypoint_by_id_option_with_executor<'a, E>(&self, plugin_id: &str, entrypoint_id: &str, executor: E) -> anyhow::Result<Option<DbReadPluginEntrypoint>>
        where
            E: Executor<'a, Database=Sqlite>,
    {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginEntrypoint>("SELECT * FROM plugin_entrypoint WHERE id = ?1 AND plugin_id = ?2")
            .bind(entrypoint_id)
            .bind(plugin_id)
            .fetch_optional(executor)
            .await?;

        Ok(result)
    }

    pub async fn get_inline_view_entrypoint_id_for_plugin(&self, plugin_id: &str) -> anyhow::Result<Option<String>> {
        // language=SQLite
        let entrypoint_id = sqlx::query_as::<_, (String, )>("SELECT id FROM plugin_entrypoint WHERE plugin_id = ?1 AND type = 'inline-view'")
            .bind(plugin_id)
            .fetch_optional(&self.pool)
            .await?
            .map(|result| result.0);

        Ok(entrypoint_id)
    }

    pub async fn action_shortcuts(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        let DbReadPluginEntrypoint { actions, actions_user_data, .. } = self.get_entrypoint_by_id(plugin_id, entrypoint_id)
            .await?;

        let actions_user_data: HashMap<_, _> = actions_user_data.into_iter()
            .map(|data| (data.id, (data.key, data.modifier_shift, data.modifier_control, data.modifier_alt, data.modifier_meta)))
            .collect();

        let action_shortcuts = actions.into_iter()
            .map(|action| {
                let id = action.id;

                let shortcut = match actions_user_data.get(&id) {
                    None => {
                        let (physical_key, modifier_shift) = match ActionShortcutKey::from_value(&action.key) {
                            Some(key) => key.to_physical_key(),
                            None => {
                                return Err(anyhow!("unknown key: {}", &action.key))
                            },
                        };

                        let (modifier_control, modifier_alt, modifier_meta) = match action.kind {
                            DbPluginActionShortcutKind::Main => {
                                if cfg!(target_os = "macos") {
                                    (false, false, true)
                                } else {
                                    (true, false, false)
                                }
                            },
                            DbPluginActionShortcutKind::Alternative => {
                                (false, true, false)
                            },
                        };

                        PhysicalShortcut {
                            physical_key,
                            modifier_shift,
                            modifier_control,
                            modifier_alt,
                            modifier_meta,
                        }
                    }
                    Some(&(ref key, modifier_shift, modifier_control, modifier_alt, modifier_meta)) => {
                        PhysicalShortcut {
                            physical_key: PhysicalKey::from_value(key.to_owned()),
                            modifier_shift,
                            modifier_control,
                            modifier_alt,
                            modifier_meta,
                        }
                    }
                };

                Ok((id, shortcut))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(action_shortcuts)
    }

    pub async fn get_action_id_for_shortcut(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<Option<String>> {
        // language=SQLite
        let sql = r#"SELECT json_each.value ->> 'id' FROM plugin_entrypoint e, json_each(actions_user_data) WHERE e.plugin_id = ?1 AND e.id = ?2  AND json_each.value ->> 'key' = ?3 AND json_each.value ->> 'modifier_shift' = ?4 AND json_each.value ->> 'modifier_control' = ?5 AND json_each.value ->> 'modifier_alt' = ?6 AND json_each.value ->> 'modifier_meta' = ?6"#;

        let action_id = sqlx::query_as::<_, (String, )>(sql)
            .bind(plugin_id)
            .bind(entrypoint_id)
            .bind(key.to_value())
            .bind(modifier_shift)
            .bind(modifier_control)
            .bind(modifier_alt)
            .bind(modifier_meta)
            .fetch_optional(&self.pool)
            .await?
            .map(|result| result.0);

        match action_id {
            Some(action_id) => Ok(Some(action_id)),
            None => {
                let kind = if cfg!(target_os = "macos") {
                    match (modifier_control, modifier_alt, modifier_meta) {
                        (false, false, true) => DbPluginActionShortcutKind::Main,
                        (false, true, false) => DbPluginActionShortcutKind::Alternative,
                        _ => return Ok(None)
                    }
                } else {
                    match (modifier_control, modifier_alt, modifier_meta) {
                        (true, false, false) => DbPluginActionShortcutKind::Main,
                        (false, true, false) => DbPluginActionShortcutKind::Alternative,
                        _ => return Ok(None)
                    }
                };

                let kind = match kind {
                    DbPluginActionShortcutKind::Main => "main".to_owned(),
                    DbPluginActionShortcutKind::Alternative => "alternative".to_owned(),
                };

                // language=SQLite
                let sql = r#"SELECT json_each.value ->> 'id' FROM plugin_entrypoint e, json_each(actions) WHERE e.plugin_id = ?1 AND e.id = ?2  AND json_each.value ->> 'key' = ?3 AND json_each.value ->> 'kind' = ?4"#;

                let Some(logical_key) = ActionShortcutKey::from_physical_key(key, modifier_shift) else {
                    return Ok(None);
                };

                let action_id = sqlx::query_as::<_, (String, )>(sql)
                    .bind(plugin_id)
                    .bind(entrypoint_id)
                    .bind(logical_key.to_value())
                    .bind(&kind)
                    .fetch_optional(&self.pool)
                    .await?
                    .map(|result| result.0);

                Ok(action_id)
            }
        }
    }

    pub async fn list_pending_plugins(&self) -> anyhow::Result<Vec<DbReadPendingPlugin>> {
        // language=SQLite
        let plugins = sqlx::query_as::<_, DbReadPendingPlugin>("SELECT * FROM pending_plugin")
            .fetch_all(&self.pool)
            .await?;

        Ok(plugins)
    }

    pub async fn is_plugin_pending(&self, plugin_id: &str) -> anyhow::Result<bool> {
        // language=SQLite
        let result = sqlx::query_as::<_, (u8, )>("SELECT 1 FROM pending_plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.is_some())
    }

    pub async fn does_plugin_exist(&self, plugin_id: &str) -> anyhow::Result<bool> {
        // language=SQLite
        let result = sqlx::query_as::<_, (u8, )>("SELECT 1 FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.is_some())
    }

    pub async fn is_plugin_enabled(&self, plugin_id: &str) -> anyhow::Result<bool> {
        #[derive(sqlx::FromRow)]
        struct DbReadPluginEnabled {
            pub enabled: bool,
        }

        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginEnabled>("SELECT enabled FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.enabled)
    }

    pub async fn get_asset_data(&self, plugin_id: &str, path: &str) -> anyhow::Result<Vec<u8>> {
        #[derive(sqlx::FromRow)]
        struct DbReadPluginAssetData {
            pub data: Vec<u8>,
        }

        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginAssetData>("SELECT data FROM plugin_asset_data WHERE plugin_id = ?1 and path = ?2")
            .bind(plugin_id)
            .bind(path)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.data)
    }

    async fn get_all_asset_data_paths<'a, E>(&self, plugin_id: &str, executor: E) -> anyhow::Result<HashSet<String>>
        where
            E: Executor<'a, Database=Sqlite>,
    {
        // language=SQLite
        let result = sqlx::query_as::<_, (String, )>("SELECT path FROM plugin_asset_data WHERE plugin_id = ?1")
            .bind(plugin_id)
            .fetch_all(executor)
            .await?
            .into_iter()
            .map(|result| result.0)
            .collect();

        Ok(result)
    }

    pub async fn inline_view_shortcuts(&self) -> anyhow::Result<HashMap<String, HashMap<String, PhysicalShortcut>>> {
        // language=SQLite
        let shortcuts: Vec<_> = sqlx::query_as::<_, (String, String)>("SELECT id, plugin_id FROM plugin_entrypoint WHERE type = 'inline-view'")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|(entrypoint_id, plugin_id)| async move {
                let shortcuts = self.action_shortcuts(&plugin_id, &entrypoint_id).await?;

                Ok((plugin_id, shortcuts))
            })
            .collect();

        join_all(shortcuts)
            .await
            .into_iter()
            .collect()
    }

    pub async fn mark_entrypoint_frecency(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // TODO reset time after 5 half lives
        //  https://github.com/camdencheek/fre/blob/6574ee7045061957de24855567e0abf05f2778d9/src/main.rs#L23
        //  why? dunno

        #[derive(sqlx::FromRow)]
        struct DbFrecencyMetaParams {
            pub reference_time: f64,
            pub half_life: f64,
        }

        // language=SQLite
        let meta_params = sqlx::query_as::<_, DbFrecencyMetaParams>("SELECT reference_time, half_life FROM plugin_entrypoint_frecency_stats")
            .fetch_optional(&mut *tx)
            .await?;

        let meta_params = match meta_params {
            None => FrecencyMetaParams::default(),
            Some(meta_params) => FrecencyMetaParams {
                reference_time: meta_params.reference_time,
                half_life: meta_params.half_life,
            }
        };

        // language=SQLite
        let stats = sqlx::query_as::<_, DbPluginEntrypointFrecencyStats>("SELECT plugin_id, entrypoint_id, reference_time, half_life, last_accessed, frecency, num_accesses FROM plugin_entrypoint_frecency_stats WHERE plugin_id = ?1 and entrypoint_id = ?2")
            .bind(plugin_id)
            .bind(entrypoint_id)
            .fetch_optional(&mut *tx)
            .await?;

        let mut new_stats = match stats {
            None => {
                FrecencyItemStats::new(meta_params.reference_time, meta_params.half_life)
            }
            Some(stats) => {
                FrecencyItemStats {
                    half_life: stats.half_life,
                    reference_time: stats.reference_time,
                    last_accessed: stats.last_accessed,
                    frecency: stats.frecency,
                    num_accesses: stats.num_accesses,
                }
            }
        };

        new_stats.mark_used();

        // language=SQLite
        let sql = r#"
            INSERT OR REPLACE INTO plugin_entrypoint_frecency_stats (plugin_id, entrypoint_id, reference_time, half_life, last_accessed, frecency, num_accesses)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#;

        sqlx::query(sql)
            .bind(plugin_id)
            .bind(entrypoint_id)
            .bind(new_stats.reference_time)
            .bind(new_stats.half_life)
            .bind(new_stats.last_accessed)
            .bind(new_stats.frecency)
            .bind(new_stats.num_accesses)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_frecency_for_plugin(&self, plugin_id: &str) -> anyhow::Result<HashMap<String, f64>> {
        // language=SQLite
        let result = sqlx::query_as::<_, (String, f64)>("SELECT entrypoint_id, frecency FROM plugin_entrypoint_frecency_stats WHERE plugin_id = ?1")
            .bind(plugin_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .collect();

        Ok(result)
    }

    pub async fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> anyhow::Result<()> {
        // language=SQLite
        sqlx::query("UPDATE plugin SET enabled = ?1 WHERE id = ?2")
            .bind(enabled)
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn set_plugin_entrypoint_enabled(&self, plugin_id: &str, entrypoint_id: &str, enabled: bool) -> anyhow::Result<()> {
        // language=SQLite
        sqlx::query("UPDATE plugin_entrypoint SET enabled = ?1 WHERE id = ?2 AND plugin_id = ?3")
            .bind(enabled)
            .bind(entrypoint_id)
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>, error: Option<String>) -> anyhow::Result<()> {
        // language=SQLite
        let sql = r#"
            INSERT INTO settings_data (id, global_shortcut)
                VALUES(?1, ?2)
                    ON CONFLICT (id)
                        DO UPDATE SET global_shortcut = ?2
        "#;

        let shortcut_data = match shortcut {
            None => {
                DbSettingsGlobalShortcutData {
                    physical_key: "".to_string(),
                    modifier_shift: false,
                    modifier_control: false,
                    modifier_alt: false,
                    modifier_meta: false,
                    unset: true,
                    error,
                }
            }
            Some(shortcut) => {
                DbSettingsGlobalShortcutData {
                    physical_key: shortcut.physical_key.to_value(),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                    unset: false,
                    error,
                }
            }
        };

        sqlx::query(sql)
            .bind(SETTINGS_DATA_ID)
            .bind(Json(shortcut_data))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_global_shortcut(&self) -> anyhow::Result<Option<(Option<PhysicalShortcut>, Option<String>)>> {
        // language=SQLite
        let data = sqlx::query_as::<_, DbSettingsDataContainer>("SELECT * FROM settings_data")
            .fetch_optional(&self.pool)
            .await;

        match data {
            Ok(Some(data)) => {
                let shortcut_data = data.global_shortcut;

                let shortcut = if shortcut_data.unset {
                    None
                } else {
                    Some(PhysicalShortcut {
                        physical_key: PhysicalKey::from_value(shortcut_data.physical_key),
                        modifier_shift: shortcut_data.modifier_shift,
                        modifier_control: shortcut_data.modifier_control,
                        modifier_alt: shortcut_data.modifier_alt,
                        modifier_meta: shortcut_data.modifier_meta,
                    })
                };

                Ok(Some((
                    shortcut,
                    shortcut_data.error,
                )))
            },
            Ok(None) => Ok(None),
            Err(err) => Err(anyhow!("Unable to get global shortcut from db: {:?}", err))
        }
    }

    pub async fn get_settings(&self) -> anyhow::Result<DbSettings> {
        // language=SQLite
        let settings = sqlx::query_as::<_, DbSettingsDataContainer>("SELECT * FROM settings_data")
            .fetch_optional(&self.pool)
            .await?;

        let theme = settings
            .map(|data| data.settings)
            .flatten()
            .unwrap_or_default();

        Ok(theme.0)
    }

    pub async fn set_settings(&self, value: DbSettings) -> anyhow::Result<()> {
        // language=SQLite
        let sql = r#"
            INSERT INTO settings_data (id, global_shortcut, settings)
                VALUES(?1, ?2, ?3)
                    ON CONFLICT (id)
                        DO UPDATE SET settings = ?2
        "#;

        sqlx::query(sql)
            .bind(SETTINGS_DATA_ID)
            .bind(Json(value))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn set_preference_value(&self, plugin_id: String, entrypoint_id: Option<String>, preference_id: String, value: DbPluginPreferenceUserData) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        match entrypoint_id {
            None => {
                let mut user_data = self.get_plugin_by_id_with_executor(&plugin_id, &mut *tx)
                    .await?
                    .preferences_user_data;

                user_data.insert(preference_id, value);

                // language=SQLite
                sqlx::query("UPDATE plugin SET preferences_user_data = ?1 WHERE id = ?2")
                    .bind(Json(user_data))
                    .bind(&plugin_id)
                    .execute(&mut *tx)
                    .await?;
            }
            Some(entrypoint_id) => {
                let mut user_data = self.get_entrypoint_by_id_with_executor(&plugin_id, &entrypoint_id, &mut *tx)
                    .await?
                    .preferences_user_data;

                user_data.insert(preference_id, value);

                // language=SQLite
                sqlx::query("UPDATE plugin_entrypoint SET preferences_user_data = ?1 WHERE id = ?2 AND plugin_id = ?3")
                    .bind(Json(user_data))
                    .bind(&entrypoint_id)
                    .bind(&plugin_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn save_pending_plugin(&self, plugin: DbWritePendingPlugin) -> anyhow::Result<()> {
        // language=SQLite
        sqlx::query("INSERT INTO pending_plugin VALUES(?1)")
            .bind(&plugin.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn remove_plugin(&self, plugin_id: &str) -> anyhow::Result<()> {
        // language=SQLite
        sqlx::query("DELETE FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_plugin(&self, new_plugin: DbWritePlugin) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let (uuid, enabled, preferences_user_data) = self.get_plugin_by_id_option_with_executor(&new_plugin.id, &mut *tx).await?
            .map(|plugin| (plugin.uuid, plugin.enabled, plugin.preferences_user_data))
            .unwrap_or((Uuid::new_v4().to_string(), new_plugin.enabled, HashMap::new()));

        // language=SQLite
        let sql = r#"
            INSERT INTO plugin (id, name, enabled, code, permissions, preferences, preferences_user_data, description, type, uuid)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ON CONFLICT (id)
                        DO UPDATE SET name = ?2, enabled = ?3, code = ?4, permissions = ?5, preferences = ?6, preferences_user_data = ?7, description = ?8, type = ?9, uuid = ?10
        "#;

        sqlx::query(sql)
            .bind(&new_plugin.id)
            .bind(new_plugin.name)
            .bind(enabled)
            .bind(Json(new_plugin.code))
            .bind(Json(new_plugin.permissions))
            .bind(Json(new_plugin.preferences))
            .bind(Json(preferences_user_data))
            .bind(new_plugin.description)
            .bind(new_plugin.plugin_type)
            .bind(uuid)
            .execute(&mut *tx)
            .await?;

        let mut old_entrypoint_ids = self.get_entrypoints_by_plugin_id_with_executor(&new_plugin.id, &mut *tx).await?
            .into_iter()
            .map(|entrypoint| entrypoint.id)
            .collect::<HashSet<_>>();

        for new_entrypoint in new_plugin.entrypoints {
            old_entrypoint_ids.remove(&new_entrypoint.id);

            let (uuid, preferences_user_data, actions_user_data, enabled) = self.get_entrypoint_by_id_option_with_executor(&new_plugin.id, &new_entrypoint.id, &mut *tx).await?
                .map(|entrypoint| (entrypoint.uuid, entrypoint.preferences_user_data, entrypoint.actions_user_data, entrypoint.enabled))
                .unwrap_or((Uuid::new_v4().to_string(), HashMap::new(), vec![], true));

            // language=SQLite
            sqlx::query("INSERT OR REPLACE INTO plugin_entrypoint (id, plugin_id, name, enabled, type, preferences, preferences_user_data, description, actions, actions_user_data, icon_path, uuid) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)")
                .bind(&new_entrypoint.id)
                .bind(&new_plugin.id)
                .bind(new_entrypoint.name)
                .bind(enabled)
                .bind(new_entrypoint.entrypoint_type)
                .bind(Json(new_entrypoint.preferences))
                .bind(Json(preferences_user_data))
                .bind(new_entrypoint.description)
                .bind(Json(new_entrypoint.actions))
                .bind(Json(actions_user_data))
                .bind(new_entrypoint.icon_path)
                .bind(uuid)
                .execute(&mut *tx)
                .await?;
        }

        for old_entrypoint_id in old_entrypoint_ids {
            // language=SQLite
            sqlx::query("DELETE FROM plugin_entrypoint WHERE id = ?1")
                .bind(&old_entrypoint_id)
                .execute(&mut *tx)
                .await?;
        }


        let mut old_asset_data_paths = self.get_all_asset_data_paths(&new_plugin.id, &mut *tx).await?;

        for data in new_plugin.asset_data {
            old_asset_data_paths.remove(&data.path);

            // language=SQLite
            sqlx::query("INSERT OR REPLACE INTO plugin_asset_data (plugin_id, path, data) VALUES(?1, ?2, ?3)")
                .bind(&new_plugin.id)
                .bind(&data.path)
                .bind(&data.data)
                .execute(&mut *tx)
                .await?;
        }

        for old_asset_data_path in old_asset_data_paths {
            // language=SQLite
            sqlx::query("DELETE FROM plugin_asset_data WHERE plugin_id = ?1 AND path = ?2")
                .bind(&new_plugin.id)
                .bind(&old_asset_data_path)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}


pub fn db_entrypoint_to_str(value: DbPluginEntrypointType) -> &'static str {
    match value {
        DbPluginEntrypointType::Command => "command",
        DbPluginEntrypointType::View => "view",
        DbPluginEntrypointType::InlineView => "inline-view",
        DbPluginEntrypointType::EntrypointGenerator => "command-generator" // command-generator in db for backwards compatibility
    }
}

pub fn db_entrypoint_from_str(value: &str) -> DbPluginEntrypointType {
    match value {
        "command" => DbPluginEntrypointType::Command,
        "view" => DbPluginEntrypointType::View,
        "inline-view" => DbPluginEntrypointType::InlineView,
        "command-generator" => DbPluginEntrypointType::EntrypointGenerator,
        _ => panic!("illegal entrypoint_type: {}", value)
    }
}


pub fn db_plugin_type_to_str(value: DbPluginType) -> &'static str {
    match value {
        DbPluginType::Normal => "normal",
        DbPluginType::Config => "config",
        DbPluginType::Bundled => "bundled"
    }
}

pub fn db_plugin_type_from_str(value: &str) -> DbPluginType {
    match value {
        "normal" => DbPluginType::Normal,
        "config" => DbPluginType::Config,
        "bundled" => DbPluginType::Bundled,
        _ => panic!("illegal plugin_type: {}", value)
    }
}
