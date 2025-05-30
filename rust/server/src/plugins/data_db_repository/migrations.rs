use rusqlite::Transaction;
use rusqlite::named_params;
use rusqlite_migration::HookError;
use rusqlite_migration::HookResult;
use rusqlite_migration::M;
use rusqlite_migration::MigrationHook;
use rusqlite_migration::Migrations;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::plugins::data_db_repository::DbSettings;
use crate::plugins::data_db_repository::DbSettingsGlobalShortcutData;
use crate::plugins::data_db_repository::DbSettingsShortcut;
use crate::plugins::data_db_repository::RusqliteFromRow;
use crate::plugins::data_db_repository::SETTINGS_DATA_ID;

fn migration_needed(conn: &Transaction, description: &str) -> Result<bool, rusqlite::Error> {
    let result = conn.query_row(
        "SELECT 1 FROM _sqlx_migrations WHERE description = :description",
        named_params! {
            ":description": description
        },
        |_| Ok(()),
    );

    let needed = match result {
        Ok(()) => false, // already applied by previous system
        Err(err) => {
            match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    // migration not applied by previous system, but db exists
                    true
                }
                _ => {
                    // possibly, no _sqlx_migrations table, meaning database is new,
                    // or migration to new system is already done which shouldn't happen because migration was already applied
                    true
                }
            }
        }
    };

    Ok(needed)
}

fn legacy_migration(legacy_description: &'static str, sql: &'static str) -> impl MigrationHook + 'static {
    move |tx| -> HookResult {
        if migration_needed(tx, legacy_description)? {
            tx.execute_batch(sql)?;
        }

        Ok(())
    }
}

fn legacy_migration_fn(
    legacy_description: &'static str,
    hook: impl MigrationHook + Clone + 'static,
) -> impl MigrationHook + 'static {
    move |tx| {
        if migration_needed(&tx, legacy_description)? {
            hook(&tx)?;
        }

        Ok(())
    }
}

fn migrate_global_shortcut_to_settings(tx: &Transaction) -> HookResult {
    #[derive(RusqliteFromRow)]
    struct DbSettingsDataOldContainer {
        #[rusqlite(json)]
        pub global_shortcut: DbSettingsGlobalShortcutOldData,
        #[rusqlite(json)]
        pub settings: Option<DbSettings>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DbSettingsGlobalShortcutOldData {
        pub physical_key: String,
        pub modifier_shift: bool,
        pub modifier_control: bool,
        pub modifier_alt: bool,
        pub modifier_meta: bool,
        #[serde(default)]
        pub unset: bool,
        #[serde(default)]
        pub error: Option<String>,
    }

    // language=SQLite
    let query = "SELECT * FROM settings_data";

    let mut data = tx.prepare(query)?.query_row([], DbSettingsDataOldContainer::from_row)?;

    let shortcut_data = &data.global_shortcut;

    let shortcut = if shortcut_data.unset {
        None
    } else {
        Some(DbSettingsGlobalShortcutData {
            shortcut: DbSettingsShortcut {
                physical_key: shortcut_data.physical_key.clone(),
                modifier_shift: shortcut_data.modifier_shift,
                modifier_control: shortcut_data.modifier_control,
                modifier_alt: shortcut_data.modifier_alt,
                modifier_meta: shortcut_data.modifier_meta,
            },
            error: None,
        })
    };

    if let Some(settings) = &mut data.settings {
        settings.global_shortcut = shortcut;
    }

    // language=SQLite
    let query = r#"
        INSERT INTO settings_data (id, global_shortcut, settings)
        VALUES(:id, :global_shortcut, :settings)
            ON CONFLICT (id)
                DO UPDATE SET settings = :settings
    "#;

    tx.execute(
        query,
        named_params! {
            ":id": SETTINGS_DATA_ID,
            ":global_shortcut": serde_json::to_value(data.global_shortcut).map_err(|err| HookError::Hook(format!("{:?}", err)))?,
            ":settings": serde_json::to_value(data.settings).map_err(|err| HookError::Hook(format!("{:?}", err)))?,
        },
    )?;

    Ok(())
}

fn remove_legacy_bundled_plugins(tx: &Transaction) -> HookResult {
    let remove_plugin = |plugin_id: &str| -> HookResult {
        // language=SQLite
        let query = "DELETE FROM plugin WHERE id = :id";

        tx.execute(
            query,
            named_params! {
                ":id": plugin_id
            },
        )?;

        Ok(())
    };

    remove_plugin("builtin://applications")?;
    remove_plugin("builtin://calculator")?;
    remove_plugin("builtin://settings")?;

    Ok(())
}

fn apply_uuid_default_value(tx: &Transaction) -> HookResult {
    #[derive(RusqliteFromRow)]
    struct DbResultId {
        pub id: String,
    }

    // language=SQLite
    let query = "SELECT id FROM plugin WHERE uuid IS NULL";

    let results = tx
        .prepare(query)?
        .query_and_then([], DbResultId::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    for result in results {
        // language=SQLite
        let query = "UPDATE plugin SET uuid = :uuid WHERE id = :id";

        tx.execute(
            query,
            named_params! {
                ":id": result.id,
                ":uuid": Uuid::new_v4().to_string()
            },
        )?;
    }

    // language=SQLite
    let query = "SELECT id FROM plugin_entrypoint WHERE uuid IS NULL";

    let results = tx
        .prepare(query)?
        .query_and_then([], DbResultId::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    for result in results {
        // language=SQLite
        let query = "UPDATE plugin_entrypoint SET uuid = :uuid WHERE id = :id";

        tx.execute(
            query,
            named_params! {
                ":id": result.id,
                ":uuid": Uuid::new_v4().to_string()
            },
        )?;
    }

    Ok(())
}

#[rustfmt::skip]
pub fn setup_migrator() -> Migrations<'static> {
    Migrations::new(vec![
        M::up_with_hook("-- 1", legacy_migration("initial", include_str!("migrations/1_initial.sql"))),
        M::up_with_hook("-- 2", legacy_migration("preferences", include_str!("migrations/2_preferences.sql"))),
        M::up_with_hook("-- 3", legacy_migration("plugin description", include_str!("migrations/3_plugin_description.sql"))),
        M::up_with_hook("-- 4", legacy_migration("plugin asset data", include_str!("migrations/4_plugin_asset_data.sql"))),
        M::up_with_hook("-- 5", legacy_migration("plugin action shortcuts", include_str!("migrations/5_plugin_action_shortcuts.sql"))),
        M::up_with_hook("-- 6", legacy_migration("plugin type", include_str!("migrations/6_plugin_type.sql"))),
        M::up_with_hook("-- 7", legacy_migration("plugin entrypoint icon", include_str!("migrations/7_plugin_entrypoint_icon.sql"))),
        M::up_with_hook("-- 8", legacy_migration("uuids", include_str!("migrations/8_uuids.sql"))),
        M::up_with_hook("-- 9", apply_uuid_default_value),
        M::up_with_hook("-- 10", legacy_migration("frecency", include_str!("migrations/9_frecency.sql"))),
        M::up_with_hook("-- 11", legacy_migration("frecency fix primary keys", include_str!("migrations/10_frecency_fix_primary_keys.sql"))),
        M::up_with_hook("-- 12", legacy_migration("settings data", include_str!("migrations/11_settings_data.sql"))),
        M::up_with_hook("-- 13", remove_legacy_bundled_plugins),
        M::up_with_hook("-- 14", legacy_migration("settings theme", include_str!("migrations/12_settings_theme.sql"))),
        M::up_with_hook("-- 15", legacy_migration_fn("remove old global shortcut", |tx| {
            let _ = migrate_global_shortcut_to_settings(tx);
            Ok(())
        })),
        M::up_with_hook("-- 16", legacy_migration("remove old global shortcut", include_str!("migrations/13_remove_old_global_shortcut.sql"))),
        M::up(include_str!("migrations/14_migrate_to_rusqlite_migration.sql"))
    ])
}
