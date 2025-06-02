mod migrations;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::anyhow;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_utils_macros::RusqliteFromRow;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::Row;
use rusqlite::named_params;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::model::ActionShortcutKey;
use crate::plugins::data_db_repository::migrations::setup_migrator;
use crate::plugins::frecency::FrecencyItemStats;
use crate::plugins::frecency::FrecencyMetaParams;

#[derive(Clone)]
pub struct DataDbRepository {
    connection: Arc<Mutex<Connection>>,
}

#[derive(RusqliteFromRow)]
pub struct DbReadPlugin {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    #[rusqlite(json)]
    pub code: DbCode,
    #[rusqlite(json)]
    pub permissions: DbPluginPermissions,
    #[rusqlite(rename = "type")]
    pub plugin_type: String,
    #[rusqlite(json)]
    pub preferences: HashMap<String, DbPluginPreference>,
    #[rusqlite(json)]
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
}

#[derive(RusqliteFromRow)]
pub struct DbReadPluginEntrypoint {
    pub id: String,
    pub uuid: String,
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub icon_path: Option<String>,
    #[rusqlite(rename = "type")]
    pub entrypoint_type: String,
    #[rusqlite(json)]
    pub preferences: HashMap<String, DbPluginPreference>,
    #[rusqlite(json)]
    pub preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
    #[rusqlite(json)]
    pub actions: Vec<DbPluginAction>,
    #[rusqlite(json)]
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
    pub data: Vec<u8>,
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
    #[allow(unused)]
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
    Clear,
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
    Number { value: Option<f64> },
    #[serde(rename = "string")]
    String { value: Option<String> },
    #[serde(rename = "enum")]
    Enum { value: Option<String> },
    #[serde(rename = "bool")]
    Bool { value: Option<bool> },
    #[serde(rename = "list_of_strings")]
    ListOfStrings { value: Option<Vec<String>> },
    #[serde(rename = "list_of_numbers")]
    ListOfNumbers { value: Option<Vec<f64>> },
    #[serde(rename = "list_of_enums")]
    ListOfEnums { value: Option<Vec<String>> },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPluginAction {
    pub id: String,
    pub description: String,
    pub key: Option<String>,
    pub kind: Option<DbPluginActionShortcutKind>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPluginActionUserData {
    pub id: String,
    pub key: String,
    pub modifier_shift: bool,
    pub modifier_control: bool,
    pub modifier_alt: bool,
    pub modifier_meta: bool,
}

#[derive(RusqliteFromRow)]
struct DbSettingsDataContainer {
    #[rusqlite(json)]
    pub settings: Option<DbSettings>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbSettingsShortcut {
    pub physical_key: String,
    pub modifier_shift: bool,
    pub modifier_control: bool,
    pub modifier_alt: bool,
    pub modifier_meta: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbSettingsGlobalShortcutData {
    pub shortcut: DbSettingsShortcut,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbSettingsGlobalEntrypointShortcutData {
    pub plugin_id: String,
    pub entrypoint_id: String,
    pub shortcut: DbSettingsGlobalShortcutData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbSettingsEntrypointSearchAliasData {
    pub plugin_id: String,
    pub entrypoint_id: String,
    pub alias: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbSettings {
    // none means auto-detect
    pub theme: Option<DbTheme>,
    // none is static mode
    pub window_position_mode: Option<DbWindowPositionMode>,
    // none is unset, if whole settings object is unset, it is likely a first start and default shortcut will be used
    pub global_shortcut: Option<DbSettingsGlobalShortcutData>,
    pub global_entrypoint_shortcuts: Option<Vec<DbSettingsGlobalEntrypointShortcutData>>,
    pub entrypoint_search_aliases: Option<Vec<DbSettingsEntrypointSearchAliasData>>,
}

impl Default for DbSettings {
    fn default() -> Self {
        let default_global_shortcut = if cfg!(target_os = "windows") {
            DbSettingsShortcut {
                physical_key: PhysicalKey::Space.to_value(),
                modifier_shift: false,
                modifier_control: false,
                modifier_alt: true,
                modifier_meta: false,
            }
        } else {
            DbSettingsShortcut {
                physical_key: PhysicalKey::Space.to_value(),
                modifier_shift: false,
                modifier_control: false,
                modifier_alt: false,
                modifier_meta: true,
            }
        };

        DbSettings {
            theme: None,
            window_position_mode: None,
            global_shortcut: Some(DbSettingsGlobalShortcutData {
                shortcut: default_global_shortcut,
                error: None,
            }),
            global_entrypoint_shortcuts: None,
            entrypoint_search_aliases: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DbTheme {
    #[serde(rename = "macos_light")]
    MacOSLight,
    #[serde(rename = "macos_dark")]
    MacOSDark,
    #[serde(rename = "legacy")]
    Legacy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DbWindowPositionMode {
    #[serde(rename = "active_monitor")]
    ActiveMonitor,
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
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbPreferenceEnumValue {
    pub label: String,
    pub value: String,
}

#[derive(RusqliteFromRow)]
pub struct DbPluginEntrypointFrecencyStats {
    #[allow(unused)]
    pub plugin_id: String,
    #[allow(unused)]
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

        std::fs::create_dir_all(&data_db_file.parent().unwrap()).context("Unable to create data directory")?;

        let mut connection = Connection::open(&data_db_file).context("Unable to open database connection")?;

        setup_migrator()
            .to_latest(&mut connection)
            .context("Unable apply database migration")?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn list_plugins(&self) -> anyhow::Result<Vec<DbReadPlugin>> {
        // language=SQLite
        let query = "SELECT * FROM plugin";

        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        let plugins = connection
            .prepare(query)?
            .query_map([], DbReadPlugin::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(plugins)
    }

    pub fn list_plugins_and_entrypoints(&self) -> anyhow::Result<Vec<(DbReadPlugin, Vec<DbReadPluginEntrypoint>)>> {
        // language=SQLite
        let plugins = self.list_plugins()?;

        let result = plugins
            .into_iter()
            .map(|plugin| {
                let entrypoints = self.get_entrypoints_by_plugin_id(&plugin.id)?;

                Ok::<(DbReadPlugin, Vec<DbReadPluginEntrypoint>), anyhow::Error>((plugin, entrypoints))
            })
            .collect::<Result<Vec<(DbReadPlugin, Vec<DbReadPluginEntrypoint>)>, _>>()?;

        Ok(result)
    }

    pub fn get_plugin_by_id(&self, plugin_id: &str) -> anyhow::Result<DbReadPlugin> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        self.get_plugin_by_id_with_executor(plugin_id, &connection)
    }

    fn get_plugin_by_id_with_executor(&self, plugin_id: &str, connection: &Connection) -> anyhow::Result<DbReadPlugin> {
        // language=SQLite
        let query = "SELECT * FROM plugin WHERE id = :id";

        // todo change into query_one when updating to rusqlite 0.36
        let result = connection.query_row(
            query,
            named_params! {
                ":id": plugin_id
            },
            DbReadPlugin::from_row,
        )?;

        Ok(result)
    }

    pub fn get_plugin_by_id_option(&self, plugin_id: &str) -> anyhow::Result<Option<DbReadPlugin>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        self.get_plugin_by_id_option_with_executor(plugin_id, &connection)
    }

    fn get_plugin_by_id_option_with_executor(
        &self,
        plugin_id: &str,
        connection: &Connection,
    ) -> anyhow::Result<Option<DbReadPlugin>> {
        // language=SQLite
        let query = "SELECT * FROM plugin WHERE id = :id";

        let result = connection
            .query_row(
                query,
                named_params! {
                    ":id": plugin_id
                },
                DbReadPlugin::from_row,
            )
            .optional()?;

        Ok(result)
    }

    pub fn get_entrypoints_by_plugin_id(&self, plugin_id: &str) -> anyhow::Result<Vec<DbReadPluginEntrypoint>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        self.get_entrypoints_by_plugin_id_with_executor(plugin_id, &connection)
    }

    fn get_entrypoints_by_plugin_id_with_executor(
        &self,
        plugin_id: &str,
        connection: &Connection,
    ) -> anyhow::Result<Vec<DbReadPluginEntrypoint>> {
        // language=SQLite
        let query = "SELECT * FROM plugin_entrypoint WHERE plugin_id = :id";

        let result = connection
            .prepare(query)?
            .query_and_then(
                named_params! {
                    ":id": plugin_id
                },
                DbReadPluginEntrypoint::from_row,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(result)
    }

    pub fn get_entrypoint_by_id(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<DbReadPluginEntrypoint> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        self.get_entrypoint_by_id_with_executor(plugin_id, entrypoint_id, &connection)
    }

    fn get_entrypoint_by_id_with_executor(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        connection: &Connection,
    ) -> anyhow::Result<DbReadPluginEntrypoint> {
        // language=SQLite
        let query = "SELECT * FROM plugin_entrypoint WHERE id = :id AND plugin_id = :plugin_id";

        // todo change into query_one when updating to rusqlite 0.36
        let result = connection.query_row(
            query,
            named_params! {
                ":id": entrypoint_id,
                ":plugin_id": plugin_id,
            },
            DbReadPluginEntrypoint::from_row,
        )?;

        Ok(result)
    }

    pub fn get_entrypoint_by_id_option(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
    ) -> anyhow::Result<Option<DbReadPluginEntrypoint>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        self.get_entrypoint_by_id_option_with_executor(plugin_id, entrypoint_id, &connection)
    }

    fn get_entrypoint_by_id_option_with_executor(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        connection: &Connection,
    ) -> anyhow::Result<Option<DbReadPluginEntrypoint>> {
        // language=SQLite
        let query = "SELECT * FROM plugin_entrypoint WHERE id = :id AND plugin_id = :plugin_id";

        let result = connection
            .query_row(
                query,
                named_params! {
                    ":id": entrypoint_id,
                    ":plugin_id": plugin_id,
                },
                DbReadPluginEntrypoint::from_row,
            )
            .optional()?;

        Ok(result)
    }

    pub fn get_inline_view_entrypoint_id_for_plugin(&self, plugin_id: &str) -> anyhow::Result<Option<String>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = "SELECT id FROM plugin_entrypoint WHERE plugin_id = :plugin_id AND type = 'inline-view'";

        #[derive(RusqliteFromRow)]
        struct DbReadInlineViewEntrypoint {
            pub id: String,
        }

        let result = connection
            .query_row(
                query,
                named_params! {
                    ":plugin_id": plugin_id
                },
                DbReadInlineViewEntrypoint::from_row,
            )
            .optional()?
            .map(|row| row.id);

        Ok(result)
    }

    pub fn action_shortcuts(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
    ) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        self.action_shortcuts_with_executor(plugin_id, entrypoint_id, &connection)
    }

    fn action_shortcuts_with_executor(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        connection: &Connection,
    ) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        let DbReadPluginEntrypoint {
            actions,
            actions_user_data,
            ..
        } = self.get_entrypoint_by_id_with_executor(plugin_id, entrypoint_id, connection)?;

        let actions_user_data: HashMap<_, _> = actions_user_data
            .into_iter()
            .map(|data| {
                (
                    data.id,
                    (
                        data.key,
                        data.modifier_shift,
                        data.modifier_control,
                        data.modifier_alt,
                        data.modifier_meta,
                    ),
                )
            })
            .collect();

        let action_shortcuts = actions
            .into_iter()
            .filter_map(|action| {
                let id = action.id;

                let shortcut = match actions_user_data.get(&id) {
                    None => {
                        let (Some(key), Some(kind)) = (action.key, action.kind) else {
                            return None;
                        };

                        let (physical_key, modifier_shift) = match ActionShortcutKey::from_value(&key) {
                            Some(key) => key.to_physical_key(),
                            None => return Some(Err(anyhow!("unknown key: {}", &key))),
                        };

                        let (modifier_control, modifier_alt, modifier_meta) = match kind {
                            DbPluginActionShortcutKind::Main => {
                                if cfg!(target_os = "macos") {
                                    (false, false, true)
                                } else {
                                    (true, false, false)
                                }
                            }
                            DbPluginActionShortcutKind::Alternative => (false, true, false),
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

                Some(Ok((id, shortcut)))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(action_shortcuts)
    }

    pub fn get_action_id_for_shortcut(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> anyhow::Result<Option<String>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = r#"
            SELECT json_each.value ->> 'id' AS id
            FROM plugin_entrypoint e, json_each(actions_user_data)
            WHERE e.plugin_id = :plugin_id
                AND e.id = :entrypoint_id
                AND json_each.value ->> 'key' = :key
                AND json_each.value ->> 'modifier_shift' = :modifier_shift
                AND json_each.value ->> 'modifier_control' = :modifier_control
                AND json_each.value ->> 'modifier_alt' = :modifier_alt
                AND json_each.value ->> 'modifier_meta' = :modifier_meta
        "#;

        #[derive(RusqliteFromRow)]
        struct DbReadActionId {
            pub id: String,
        }

        let action_id = connection
            .query_row(
                query,
                named_params! {
                    ":plugin_id": plugin_id,
                    ":entrypoint_id": entrypoint_id,
                    ":key": key.to_value(),
                    ":modifier_shift": modifier_shift,
                    ":modifier_control": modifier_control,
                    ":modifier_alt": modifier_alt,
                    ":modifier_meta": modifier_meta,
                },
                DbReadActionId::from_row,
            )
            .optional()?
            .map(|row| row.id);

        match action_id {
            Some(action_id) => Ok(Some(action_id)),
            None => {
                let kind = if cfg!(target_os = "macos") {
                    match (modifier_control, modifier_alt, modifier_meta) {
                        (false, false, true) => DbPluginActionShortcutKind::Main,
                        (false, true, false) => DbPluginActionShortcutKind::Alternative,
                        _ => return Ok(None),
                    }
                } else {
                    match (modifier_control, modifier_alt, modifier_meta) {
                        (true, false, false) => DbPluginActionShortcutKind::Main,
                        (false, true, false) => DbPluginActionShortcutKind::Alternative,
                        _ => return Ok(None),
                    }
                };

                let kind = match kind {
                    DbPluginActionShortcutKind::Main => "main".to_owned(),
                    DbPluginActionShortcutKind::Alternative => "alternative".to_owned(),
                };

                // language=SQLite
                let query = r#"
                    SELECT json_each.value ->> 'id' AS id
                    FROM plugin_entrypoint e, json_each(actions)
                    WHERE e.plugin_id = :plugin_id
                        AND e.id = :entrypoint_id
                        AND json_each.value ->> 'key' = :key
                        AND json_each.value ->> 'kind' = :kind
                "#;

                let Some(logical_key) = ActionShortcutKey::from_physical_key(key, modifier_shift) else {
                    return Ok(None);
                };

                let action_id = connection
                    .query_row(
                        query,
                        named_params! {
                            ":plugin_id": plugin_id,
                            ":entrypoint_id": entrypoint_id,
                            ":key": logical_key.to_value(),
                            ":kind": kind,
                        },
                        DbReadActionId::from_row,
                    )
                    .optional()?
                    .map(|row| row.id);

                Ok(action_id)
            }
        }
    }

    pub fn is_plugin_enabled(&self, plugin_id: &str) -> anyhow::Result<bool> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        #[derive(RusqliteFromRow)]
        struct DbReadPluginEnabled {
            pub enabled: bool,
        }

        // language=SQLite
        let query = "SELECT enabled FROM plugin WHERE id = :id";

        // todo change into query_one when updating to rusqlite 0.36
        let result = connection.query_row(
            query,
            named_params! {
                ":id": plugin_id
            },
            DbReadPluginEnabled::from_row,
        )?;

        Ok(result.enabled)
    }

    pub fn get_asset_data(&self, plugin_id: &str, path: &str) -> anyhow::Result<Vec<u8>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        #[derive(RusqliteFromRow)]
        struct DbReadPluginAssetData {
            pub data: Vec<u8>,
        }

        // language=SQLite
        let query = "SELECT data FROM plugin_asset_data WHERE plugin_id = :plugin_id and path = :path";

        // todo change into query_one when updating to rusqlite 0.36
        let result = connection.query_row(
            query,
            named_params! {
                ":plugin_id": plugin_id,
                ":path": path,
            },
            DbReadPluginAssetData::from_row,
        )?;

        Ok(result.data)
    }

    fn get_all_asset_data_paths(&self, plugin_id: &str, connection: &Connection) -> anyhow::Result<HashSet<String>> {
        #[derive(RusqliteFromRow)]
        struct DbReadPluginAssetPaths {
            pub path: String,
        }

        // language=SQLite
        let query = "SELECT path FROM plugin_asset_data WHERE plugin_id = :plugin_id";

        let result = connection
            .prepare(query)?
            .query_and_then(
                named_params! {
                    ":plugin_id": plugin_id
                },
                DbReadPluginAssetPaths::from_row,
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|row| row.path)
            .collect();

        Ok(result)
    }

    pub fn inline_view_shortcuts(&self) -> anyhow::Result<HashMap<String, HashMap<String, PhysicalShortcut>>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        #[derive(RusqliteFromRow)]
        struct DbReadPluginInlineViewShortcuts {
            pub id: String,
            pub plugin_id: String,
        }

        // language=SQLite
        let query = "SELECT id, plugin_id FROM plugin_entrypoint WHERE type = 'inline-view'";

        let result = connection
            .prepare(query)?
            .query_and_then([], DbReadPluginInlineViewShortcuts::from_row)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|row| {
                let shortcuts = self.action_shortcuts_with_executor(&row.plugin_id, &row.id, &connection)?;

                Ok::<_, anyhow::Error>((row.plugin_id, shortcuts))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(result)
    }

    pub fn mark_entrypoint_frecency(&self, plugin_id: &str, entrypoint_id: &str) -> anyhow::Result<()> {
        let mut connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        let tx = connection.transaction()?;

        // TODO reset time after 5 half lives
        //  https://github.com/camdencheek/fre/blob/6574ee7045061957de24855567e0abf05f2778d9/src/main.rs#L23
        //  why? dunno

        #[derive(RusqliteFromRow)]
        struct DbFrecencyMetaParams {
            pub reference_time: f64,
            pub half_life: f64,
        }

        // language=SQLite
        let query = "SELECT reference_time, half_life FROM plugin_entrypoint_frecency_stats";

        let meta_params = tx.query_row(query, [], DbFrecencyMetaParams::from_row).optional()?;

        let meta_params = match meta_params {
            None => FrecencyMetaParams::default(),
            Some(meta_params) => {
                FrecencyMetaParams {
                    reference_time: meta_params.reference_time,
                    half_life: meta_params.half_life,
                }
            }
        };

        // language=SQLite
        let query = r#"
            SELECT plugin_id, entrypoint_id, reference_time, half_life, last_accessed, frecency, num_accesses
            FROM plugin_entrypoint_frecency_stats
            WHERE plugin_id = :plugin_id
                and entrypoint_id = :entrypoint_id
        "#;

        let stats = tx
            .query_row(
                query,
                named_params! {
                    ":plugin_id": plugin_id,
                    ":entrypoint_id": entrypoint_id,
                },
                DbPluginEntrypointFrecencyStats::from_row,
            )
            .optional()?;

        let mut new_stats = match stats {
            None => FrecencyItemStats::new(meta_params.reference_time, meta_params.half_life),
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
        let query = r#"
            INSERT OR REPLACE INTO plugin_entrypoint_frecency_stats (plugin_id, entrypoint_id, reference_time, half_life, last_accessed, frecency, num_accesses)
                VALUES(
                    :plugin_id,
                    :entrypoint_id,
                    :reference_time,
                    :half_life,
                    :last_accessed,
                    :frecency,
                    :num_accesses
                )
        "#;

        tx.execute(
            query,
            named_params! {
                ":plugin_id": plugin_id,
                ":entrypoint_id": entrypoint_id,
                ":reference_time": new_stats.reference_time,
                ":half_life": new_stats.half_life,
                ":last_accessed": new_stats.last_accessed,
                ":frecency": new_stats.frecency,
                ":num_accesses": new_stats.num_accesses
            },
        )?;

        tx.commit()?;

        Ok(())
    }

    pub fn get_frecency_for_plugin(&self, plugin_id: &str) -> anyhow::Result<HashMap<String, f64>> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        #[derive(RusqliteFromRow)]
        struct DbFrecencyStats {
            pub entrypoint_id: String,
            pub frecency: f64,
        }

        // language=SQLite
        let query = "SELECT entrypoint_id, frecency FROM plugin_entrypoint_frecency_stats WHERE plugin_id = :plugin_id";

        let result = connection
            .prepare(query)?
            .query_and_then(
                named_params! {
                    ":plugin_id": plugin_id
                },
                DbFrecencyStats::from_row,
            )?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|row| (row.entrypoint_id, row.frecency))
            .collect::<HashMap<_, _>>();

        Ok(result)
    }

    pub fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> anyhow::Result<()> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = "UPDATE plugin SET enabled = :enabled WHERE id = :id";

        connection.execute(
            query,
            named_params! {
                ":id": plugin_id,
                ":enabled": enabled,
            },
        )?;

        Ok(())
    }

    pub fn set_plugin_entrypoint_enabled(
        &self,
        plugin_id: &str,
        entrypoint_id: &str,
        enabled: bool,
    ) -> anyhow::Result<()> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = "UPDATE plugin_entrypoint SET enabled = :enabled WHERE id = :id AND plugin_id = :plugin_id";

        connection.execute(
            query,
            named_params! {
                ":id": entrypoint_id,
                ":plugin_id": plugin_id,
                ":enabled": enabled
            },
        )?;

        Ok(())
    }

    pub fn get_settings(&self) -> anyhow::Result<DbSettings> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = "SELECT settings FROM settings_data";

        let settings = connection
            .query_row(query, [], DbSettingsDataContainer::from_row)
            .optional()?;

        let theme = settings.map(|data| data.settings).flatten().unwrap_or_default();

        Ok(theme)
    }

    pub fn set_settings(&self, value: DbSettings) -> anyhow::Result<()> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = r#"
            INSERT INTO settings_data (id, settings)
                VALUES(:id, :settings)
                    ON CONFLICT (id)
                        DO UPDATE SET settings = :settings
        "#;

        connection.execute(
            query,
            named_params! {
                ":id": SETTINGS_DATA_ID,
                ":settings": serde_json::to_value(value)?,
            },
        )?;

        Ok(())
    }

    pub fn set_preference_value(
        &self,
        plugin_id: String,
        entrypoint_id: Option<String>,
        preference_id: String,
        value: DbPluginPreferenceUserData,
    ) -> anyhow::Result<()> {
        let mut connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        let mut tx = connection.transaction()?;

        match entrypoint_id {
            None => {
                let mut user_data = self
                    .get_plugin_by_id_with_executor(&plugin_id, &mut tx)?
                    .preferences_user_data;

                user_data.insert(preference_id, value);

                // language=SQLite
                let query = "UPDATE plugin SET preferences_user_data = :data WHERE id = :id";

                tx.execute(
                    query,
                    named_params! {
                        ":data": serde_json::to_value(user_data)?,
                        ":id": plugin_id
                    },
                )?;
            }
            Some(entrypoint_id) => {
                let mut user_data = self
                    .get_entrypoint_by_id_with_executor(&plugin_id, &entrypoint_id, &mut tx)?
                    .preferences_user_data;

                user_data.insert(preference_id, value);

                // language=SQLite
                let query = "UPDATE plugin_entrypoint SET preferences_user_data = :data WHERE id = :id AND plugin_id = :plugin_id";

                tx.execute(
                    query,
                    named_params! {
                        ":data": serde_json::to_value(user_data)?,
                        ":id": entrypoint_id,
                        ":plugin_id": plugin_id,
                    },
                )?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    pub fn remove_plugin(&self, plugin_id: &str) -> anyhow::Result<()> {
        let connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        // language=SQLite
        let query = "DELETE FROM plugin WHERE id = :id";

        connection.execute(
            query,
            named_params! {
                ":id": plugin_id
            },
        )?;

        Ok(())
    }

    pub fn save_plugin(&self, new_plugin: DbWritePlugin) -> anyhow::Result<()> {
        let mut connection = self.connection.lock().map_err(|_| anyhow!("lock is poisoned"))?;
        let mut tx = connection.transaction()?;

        let (uuid, enabled, preferences_user_data) = self
            .get_plugin_by_id_option_with_executor(&new_plugin.id, &mut tx)?
            .map(|plugin| (plugin.uuid, plugin.enabled, plugin.preferences_user_data))
            .unwrap_or((Uuid::new_v4().to_string(), new_plugin.enabled, HashMap::new()));

        // language=SQLite
        let query = r#"
            INSERT INTO plugin (id, name, enabled, code, permissions, preferences, preferences_user_data, description, type, uuid)
                VALUES(:id, :name, :enabled, :code, :permissions, :preferences, :preferences_user_data, :description, :type, :uuid)
                    ON CONFLICT (id)
                        DO UPDATE SET
                            name = :name,
                            enabled = :enabled,
                            code = :code,
                            permissions = :permissions,
                            preferences = :preferences,
                            preferences_user_data = :preferences_user_data ,
                            description = :description ,
                            type = :type,
                            uuid = :uuid
        "#;

        tx.execute(
            query,
            named_params! {
                ":id": new_plugin.id,
                ":name": new_plugin.name,
                ":enabled": enabled,
                ":code": serde_json::to_value(&new_plugin.code)?,
                ":permissions": serde_json::to_value(&new_plugin.permissions)?,
                ":preferences": serde_json::to_value(&new_plugin.preferences)?,
                ":preferences_user_data": serde_json::to_value(&preferences_user_data)?,
                ":description": new_plugin.description,
                ":type": new_plugin.plugin_type,
                ":uuid": uuid
            },
        )?;

        let mut old_entrypoint_ids = self
            .get_entrypoints_by_plugin_id_with_executor(&new_plugin.id, &mut tx)?
            .into_iter()
            .map(|entrypoint| entrypoint.id)
            .collect::<HashSet<_>>();

        for new_entrypoint in new_plugin.entrypoints {
            old_entrypoint_ids.remove(&new_entrypoint.id);

            let (uuid, preferences_user_data, actions_user_data, enabled) = self
                .get_entrypoint_by_id_option_with_executor(&new_plugin.id, &new_entrypoint.id, &mut tx)?
                .map(|entrypoint| {
                    (
                        entrypoint.uuid,
                        entrypoint.preferences_user_data,
                        entrypoint.actions_user_data,
                        entrypoint.enabled,
                    )
                })
                .unwrap_or((Uuid::new_v4().to_string(), HashMap::new(), vec![], true));

            // language=SQLite
            let query = r#"
                INSERT OR REPLACE INTO plugin_entrypoint (id, plugin_id, name, enabled, type, preferences, preferences_user_data, description, actions, actions_user_data, icon_path, uuid)
                    VALUES(
                        :id,
                        :plugin_id,
                        :name,
                        :enabled,
                        :type,
                        :preferences,
                        :preferences_user_data,
                        :description,
                        :actions,
                        :actions_user_data,
                        :icon_path,
                        :uuid
                    )
            "#;

            tx.execute(
                query,
                named_params! {
                    ":id": new_entrypoint.id,
                    ":plugin_id": &new_plugin.id,
                    ":name": new_entrypoint.name,
                    ":enabled": enabled,
                    ":type": new_entrypoint.entrypoint_type,
                    ":preferences": serde_json::to_value(new_entrypoint.preferences)?,
                    ":preferences_user_data": serde_json::to_value(preferences_user_data)?,
                    ":description": new_entrypoint.description,
                    ":actions": serde_json::to_value(new_entrypoint.actions)?,
                    ":actions_user_data": serde_json::to_value(actions_user_data)?,
                    ":icon_path": new_entrypoint.icon_path,
                    ":uuid": uuid,
                },
            )?;
        }

        for old_entrypoint_id in old_entrypoint_ids {
            // language=SQLite
            let query = "DELETE FROM plugin_entrypoint WHERE id = :id";

            tx.execute(
                query,
                named_params! {
                    ":id": old_entrypoint_id
                },
            )?;
        }

        let mut old_asset_data_paths = self.get_all_asset_data_paths(&new_plugin.id, &mut tx)?;

        for data in new_plugin.asset_data {
            old_asset_data_paths.remove(&data.path);

            // language=SQLite
            let query = r#"
                INSERT OR REPLACE INTO plugin_asset_data (plugin_id, path, data)
                    VALUES(:plugin_id, :path, :data)
            "#;

            tx.execute(
                query,
                named_params! {
                    ":plugin_id": new_plugin.id,
                    ":path": data.path,
                    ":data": data.data,
                },
            )?;
        }

        for old_asset_data_path in old_asset_data_paths {
            // language=SQLite
            let query = "DELETE FROM plugin_asset_data WHERE plugin_id = :plugin_id AND path = :path";

            tx.execute(
                query,
                named_params! {
                    ":plugin_id": new_plugin.id,
                    ":path": old_asset_data_path
                },
            )?;
        }

        tx.commit()?;

        Ok(())
    }
}

pub fn db_entrypoint_to_str(value: DbPluginEntrypointType) -> &'static str {
    match value {
        DbPluginEntrypointType::Command => "command",
        DbPluginEntrypointType::View => "view",
        DbPluginEntrypointType::InlineView => "inline-view",
        DbPluginEntrypointType::EntrypointGenerator => "command-generator", // command-generator in db for backwards compatibility
    }
}

pub fn db_entrypoint_from_str(value: &str) -> DbPluginEntrypointType {
    match value {
        "command" => DbPluginEntrypointType::Command,
        "view" => DbPluginEntrypointType::View,
        "inline-view" => DbPluginEntrypointType::InlineView,
        "command-generator" => DbPluginEntrypointType::EntrypointGenerator,
        _ => panic!("illegal entrypoint_type: {}", value),
    }
}

pub fn db_plugin_type_to_str(value: DbPluginType) -> &'static str {
    match value {
        DbPluginType::Normal => "normal",
        DbPluginType::Config => "config",
        DbPluginType::Bundled => "bundled",
    }
}

#[allow(unused)]
pub fn db_plugin_type_from_str(value: &str) -> DbPluginType {
    match value {
        "normal" => DbPluginType::Normal,
        "config" => DbPluginType::Config,
        "bundled" => DbPluginType::Bundled,
        _ => panic!("illegal plugin_type: {}", value),
    }
}

pub trait RusqliteFromRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self>
    where
        Self: Sized;
}
