use std::collections::HashMap;

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
    pub from_config: bool,
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
}

#[derive(Deserialize, Serialize)]
pub struct Code {
    pub js: HashMap<String, String>,
}

pub struct SavePlugin {
    pub id: String,
    pub name: String,
    pub code: Code,
    pub entrypoints: Vec<SavePluginEntrypoint>,
    pub from_config: bool,
}

pub struct SavePluginEntrypoint {
    pub id: String,
    pub name: String,
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
        sqlx::query("INSERT INTO plugin VALUES(?1, ?2, ?3, ?4, false)")
            .bind(&plugin.id)
            .bind(plugin.name)
            .bind(true)
            .bind(Json(plugin.code))
            .execute(&mut *tx)
            .await?;

        for entrypoint in plugin.entrypoints {
            // language=SQLite
            sqlx::query("INSERT INTO plugin_entrypoint VALUES(?1, ?2, ?3, ?4)")
                .bind(entrypoint.id)
                .bind(&plugin.id)
                .bind(entrypoint.name)
                .bind(true)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
