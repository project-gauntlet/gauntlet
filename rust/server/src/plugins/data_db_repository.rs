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
use crate::plugins::loader::EnumValue;

static MIGRATOR: Migrator = sqlx::migrate!("./db_migrations");

#[derive(Clone)]
pub struct DataDbRepository {
    pool: Pool<Sqlite>,
}

pub struct GetListPlugin {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub code: Code,
    pub entrypoints: Vec<GetPluginEntrypoint>,
}

#[derive(sqlx::FromRow)]
pub struct GetPlugin {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    #[sqlx(json)]
    pub code: Code,
    #[sqlx(json)]
    pub permissions: PluginPermissions,
    pub from_config: bool,
    #[sqlx(json)]
    pub preferences: HashMap<String, PluginPreference>,
    #[sqlx(json)]
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(sqlx::FromRow)]
pub struct GetPendingPlugin {
    pub id: String,
}

#[derive(sqlx::FromRow)]
pub struct GetPluginEntrypoint {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    pub enabled: bool,
    #[sqlx(rename = "type")]
    pub entrypoint_type: String,
    #[sqlx(json)]
    pub preferences: HashMap<String, PluginPreference>,
    #[sqlx(json)]
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Deserialize, Serialize)]
pub struct Code {
    pub js: HashMap<String, String>,
}

#[derive(sqlx::FromRow)]
pub struct GetPluginPreferences {
    #[sqlx(json)]
    pub preferences: HashMap<String, PluginPreference>,
    #[sqlx(json)]
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(sqlx::FromRow)]
pub struct GetPluginEntrypointPreferences {
    #[sqlx(json)]
    pub preferences: HashMap<String, PluginPreference>,
    #[sqlx(json)]
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

pub struct SavePlugin {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub code: Code,
    pub entrypoints: Vec<SavePluginEntrypoint>,
    pub permissions: PluginPermissions,
    pub from_config: bool,
    pub preferences: HashMap<String, PluginPreference>,
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

pub struct SavePluginEntrypoint {
    pub id: String,
    pub name: String,
    pub entrypoint_type: String,
    pub preferences: HashMap<String, PluginPreference>,
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginPermissions {
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
pub enum PluginPreferenceUserData {
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
#[serde(tag = "type")]
pub enum PluginPreference {
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
        enum_values: Vec<EnumValue>,
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
        enum_values: Vec<EnumValue>,
        description: String,
    }
}

pub struct SavePendingPlugin {
    pub id: String,
}

#[derive(sqlx::FromRow)]
struct PluginEnabled {
    pub enabled: bool,
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

    pub async fn list_plugins(&self) -> anyhow::Result<Vec<GetListPlugin>> {
        // language=SQLite
        let plugins = sqlx::query_as::<_, GetPlugin>("SELECT * FROM plugin")
            .fetch_all(&self.pool)
            .await?;

        let result = futures::stream::iter(plugins)
            .then(|plugin| async move {
                let entrypoints = self.get_entrypoints_by_plugin_id(&plugin.id).await?;

                Ok::<GetListPlugin, AnyError>(GetListPlugin {
                    id: plugin.id,
                    name: plugin.name,
                    enabled: plugin.enabled,
                    code: plugin.code,
                    entrypoints,
                })
            })
            .try_collect::<Vec<GetListPlugin>>()
            .await?;

        Ok(result)
    }

    pub async fn get_plugin_by_id(&self, plugin_id: &str) -> anyhow::Result<GetPlugin> {
        // language=SQLite
        let result = sqlx::query_as::<_, GetPlugin>("SELECT * FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn get_entrypoints_by_plugin_id(&self, plugin_id: &str) -> anyhow::Result<Vec<GetPluginEntrypoint>> {
        // language=SQLite
        let result = sqlx::query_as::<_, GetPluginEntrypoint>("SELECT * FROM plugin_entrypoint WHERE plugin_id = ?1")
            .bind(plugin_id)
            .fetch_all(&self.pool)
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

    pub async fn list_pending_plugins(&self) -> anyhow::Result<Vec<GetPendingPlugin>> {
        // language=SQLite
        let plugins = sqlx::query_as::<_, GetPendingPlugin>("SELECT * FROM pending_plugin")
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
        // language=SQLite
        let result = sqlx::query_as::<_, PluginEnabled>("SELECT enabled FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.enabled)
    }

    pub async fn get_plugin_preferences(&self, plugin_id: &str) -> anyhow::Result<GetPluginPreferences> {
        // language=SQLite
        let result = sqlx::query_as::<_, GetPluginPreferences>("SELECT * FROM plugin WHERE id = ?1")
            .bind(plugin_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    pub async fn get_plugin_entrypoint_preferences(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<GetPluginEntrypointPreferences> {
        // language=SQLite
        let result = sqlx::query_as::<_, GetPluginEntrypointPreferences>("SELECT * FROM plugin_entrypoint WHERE id = ?1 AND plugin_id = ?2")
            .bind(entrypoint_id)
            .bind(plugin_id)
            .fetch_one(&self.pool)
            .await?;

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

    pub async fn save_pending_plugin(&self, plugin: SavePendingPlugin) -> anyhow::Result<()> {
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

    pub async fn save_plugin(&self, plugin: SavePlugin) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // language=SQLite
        sqlx::query("INSERT INTO plugin VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
            .bind(&plugin.id)
            .bind(plugin.name)
            .bind(plugin.enabled)
            .bind(Json(plugin.code))
            .bind(Json(plugin.permissions))
            .bind(false)
            .bind(Json(plugin.preferences))
            .bind(Json(plugin.preferences_user_data))
            .execute(&mut *tx)
            .await?;

        for entrypoint in plugin.entrypoints {
            // language=SQLite
            sqlx::query("INSERT INTO plugin_entrypoint VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                .bind(entrypoint.id)
                .bind(&plugin.id)
                .bind(entrypoint.name)
                .bind(true)
                .bind(entrypoint.entrypoint_type)
                .bind(Json(entrypoint.preferences))
                .bind(Json(entrypoint.preferences_user_data))
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
