use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use deno_core::error::AnyError;
use deno_core::futures;
use deno_core::futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::types::Json;

use crate::dirs::Dirs;

static MIGRATOR: Migrator = sqlx::migrate!("./db_migrations");

#[derive(Clone)]
pub struct DataDbRepository {
    pool: Pool<Sqlite>,
}

#[derive(sqlx::FromRow)]
pub struct DbReadPlugin {
    pub id: String,
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
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
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
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
}

pub struct DbWritePluginEntrypoint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entrypoint_type: String,
    pub preferences: HashMap<String, DbPluginPreference>,
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
    pub actions: Vec<DbPluginAction>,
    pub actions_user_data: Vec<DbPluginActionUserData>,
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
    CommandGenerator,
}

#[derive(Debug, Clone)]
pub enum DbPluginType {
    Normal,
    Config,
    Bundled,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPluginPermissions {
    pub environment: Vec<String>,
    pub high_resolution_time: bool,
    pub network: Vec<String>,
    pub ffi: Vec<PathBuf>,
    pub fs_read_access: Vec<PathBuf>,
    pub fs_write_access: Vec<PathBuf>,
    pub run_subprocess: Vec<String>,
    pub system: Vec<String>,
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
    pub kind: DbPluginActionShortcutKind
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
        default: Option<f64>,
        description: String,
    },
    #[serde(rename = "string")]
    String {
        default: Option<String>,
        description: String,
    },
    #[serde(rename = "enum")]
    Enum {
        default: Option<String>,
        description: String,
        enum_values: Vec<DbPreferenceEnumValue>,
    },
    #[serde(rename = "bool")]
    Bool {
        default: Option<bool>,
        description: String,
    },
    #[serde(rename = "list_of_strings")]
    ListOfStrings {
        default: Option<Vec<String>>,
        description: String,
    },
    #[serde(rename = "list_of_numbers")]
    ListOfNumbers {
        default: Option<Vec<f64>>,
        description: String,
    },
    #[serde(rename = "list_of_enums")]
    ListOfEnums {
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

impl DataDbRepository {
    pub async fn new(dirs: Dirs) -> anyhow::Result<Self> {
        let conn = SqliteConnectOptions::new()
            .filename(dirs.data_db_file()?)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(conn)
            .await
            .context("Unable to open database connection")?;

        // TODO backup before migration? up to 5 backups?
        MIGRATOR.run(&pool)
            .await
            .context("Unable apply database migration")?;

        Ok(Self {
            pool
        })
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

                Ok::<(DbReadPlugin, Vec<DbReadPluginEntrypoint>), AnyError>((plugin, entrypoints))
            })
            .try_collect::<Vec<(DbReadPlugin, Vec<DbReadPluginEntrypoint>)>>()
            .await?;

        Ok(result)
    }

    pub async fn get_plugin_by_id(&self, plugin_id: &str) -> anyhow::Result<DbReadPlugin> {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPlugin>("SELECT * FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn get_entrypoints_by_plugin_id(&self, plugin_id: &str) -> anyhow::Result<Vec<DbReadPluginEntrypoint>> {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginEntrypoint>("SELECT * FROM plugin_entrypoint WHERE plugin_id = ?1")
            .bind(plugin_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn get_entrypoint_by_id(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<DbReadPluginEntrypoint> {
        // language=SQLite
        let result = sqlx::query_as::<_, DbReadPluginEntrypoint>("SELECT * FROM plugin_entrypoint WHERE id = ?1 AND plugin_id = ?2")
            .bind(entrypoint_id)
            .bind(plugin_id)
            .fetch_one(&self.pool)
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

    pub async fn get_action_id_for_shortcut(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        key: &str,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<Option<String>> {
        let kind = if cfg!(target_os = "macos") {
            match (modifier_shift, modifier_control, modifier_alt, modifier_meta) {
                (_, false, false, true) => DbPluginActionShortcutKind::Main,
                (_, false, true, false) => DbPluginActionShortcutKind::Alternative,
                _ => return Ok(None)
            }
        } else {
            match (modifier_shift, modifier_control, modifier_alt, modifier_meta) {
                (_, true, false, false) => DbPluginActionShortcutKind::Main,
                (_, false, true, false) => DbPluginActionShortcutKind::Alternative,
                _ => return Ok(None)
            }
        };

        let kind = match kind {
            DbPluginActionShortcutKind::Main => "main".to_owned(),
            DbPluginActionShortcutKind::Alternative => "alternative".to_owned(),
        };

        // language=SQLite
        let sql = r#"SELECT json_each.value ->> 'id' FROM plugin_entrypoint e, json_each(actions_user_data) WHERE e.plugin_id = ?1 AND e.id = ?2  AND json_each.value ->> 'key' = ?3 AND json_each.value ->> 'kind' = ?4"#;

        let mut action_id = sqlx::query_as::<_, (String, )>(sql)
            .bind(plugin_id)
            .bind(entrypoint_id)
            .bind(key)
            .bind(&kind)
            .fetch_optional(&self.pool)
            .await?
            .map(|result| result.0);

        match action_id {
            Some(action_id) => Ok(Some(action_id)),
            None => {
                // language=SQLite
                let sql = r#"SELECT json_each.value ->> 'id' FROM plugin_entrypoint e, json_each(actions) WHERE e.plugin_id = ?1 AND e.id = ?2  AND json_each.value ->> 'key' = ?3 AND json_each.value ->> 'kind' = ?4"#;

                let action_id = sqlx::query_as::<_, (String, )>(sql)
                    .bind(plugin_id)
                    .bind(entrypoint_id)
                    .bind(key)
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

    pub async fn set_preference_value(&self, plugin_id: String, entrypoint_id: Option<String>, user_data_name: String, user_data_value: DbPluginPreferenceUserData) -> anyhow::Result<()> {
        // should probably json_patch in database for atomic update,
        // but that doesn't matter in this app

        match entrypoint_id {
            None => {
                let mut user_data = self.get_plugin_by_id(&plugin_id)
                    .await?
                    .preferences_user_data;

                user_data.insert(user_data_name, user_data_value);

                // language=SQLite
                sqlx::query("UPDATE plugin SET preferences_user_data = ?1 WHERE id = ?2")
                    .bind(Json(user_data))
                    .bind(&plugin_id)
                    .execute(&self.pool)
                    .await?;
            }
            Some(entrypoint_id) => {
                let mut user_data = self.get_entrypoint_by_id(&plugin_id, &entrypoint_id)
                    .await?
                    .preferences_user_data;

                user_data.insert(user_data_name, user_data_value);

                // language=SQLite
                sqlx::query("UPDATE plugin_entrypoint SET preferences_user_data = ?1 WHERE id = ?2 AND plugin_id = ?3")
                    .bind(Json(user_data))
                    .bind(&entrypoint_id)
                    .bind(&plugin_id)
                    .execute(&self.pool)
                    .await?;
            }
        }

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

    pub async fn save_plugin(&self, plugin: DbWritePlugin) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // language=SQLite
        sqlx::query("INSERT INTO plugin VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
            .bind(&plugin.id)
            .bind(plugin.name)
            .bind(plugin.enabled)
            .bind(Json(plugin.code))
            .bind(Json(plugin.permissions))
            .bind(Json(plugin.preferences))
            .bind(Json(plugin.preferences_user_data))
            .bind(plugin.description)
            .bind(plugin.plugin_type)
            .execute(&mut *tx)
            .await?;

        for entrypoint in plugin.entrypoints {
            // language=SQLite
            sqlx::query("INSERT INTO plugin_entrypoint VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)")
                .bind(entrypoint.id)
                .bind(&plugin.id)
                .bind(entrypoint.name)
                .bind(true)
                .bind(entrypoint.entrypoint_type)
                .bind(Json(entrypoint.preferences))
                .bind(Json(entrypoint.preferences_user_data))
                .bind(entrypoint.description)
                .bind(Json(entrypoint.actions))
                .bind(Json(entrypoint.actions_user_data))
                .execute(&mut *tx)
                .await?;
        }

        for data in plugin.asset_data {
            // language=SQLite
            sqlx::query("INSERT INTO plugin_asset_data VALUES(?1, ?2, ?3)")
                .bind(&plugin.id)
                .bind(&data.path)
                .bind(&data.data)
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
        DbPluginEntrypointType::CommandGenerator => "command-generator"
    }
}

pub fn db_entrypoint_from_str(value: &str) -> DbPluginEntrypointType {
    match value {
        "command" => DbPluginEntrypointType::Command,
        "view" => DbPluginEntrypointType::View,
        "inline-view" => DbPluginEntrypointType::InlineView,
        "command-generator" => DbPluginEntrypointType::CommandGenerator,
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
