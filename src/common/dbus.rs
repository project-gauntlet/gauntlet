use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusSearchResult {
    pub plugin_id: String,
    pub plugin_name: String,
    pub entrypoint_id: String,
    pub entrypoint_name: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusPlugin {
    pub plugin_id: String,
    pub plugin_name: String,
    pub enabled: bool,
    pub entrypoints: Vec<DBusEntrypoint>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusEntrypoint {
    pub entrypoint_id: String,
    pub entrypoint_name: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DBusUiWidget {
    pub widget_id: DbusUiWidgetId,
}


#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DbusEventViewCreated {
    pub reconciler_mode: String,
    pub view_name: String,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DbusEventViewEvent {
    pub event_name: DbusUiEventName,
    pub widget_id: DbusUiWidgetId,
}

pub type DbusUiWidgetId = u32;
pub type DbusUiEventName = String;


#[derive(Debug, Serialize, Deserialize, Type)]
// #[zvariant(signature = "({s(u)}{s(uv)})")] // TODO create issue, for better error reporting
pub struct DBusUiPropertyContainer {
    pub zero: HashMap<String, DBusUiPropertyZeroValue>,
    pub one: HashMap<String, DBusUiPropertyOneValue>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[zvariant(signature = "(uv)")]
pub enum DBusUiPropertyOneValue {
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[zvariant(signature = "u")]
pub enum DBusUiPropertyZeroValue {
    Function,
}