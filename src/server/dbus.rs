use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;
use crate::server::plugins::js::{UiPropertyValue, UiWidget, UiWidgetId};

use crate::server::search::SearchIndex;

pub struct DbusServer {
    pub plugins: Vec<String>,
    pub search_index: SearchIndex,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName")]
impl DbusServer {
    fn plugins(&mut self) -> Vec<String> {
        self.plugins.clone()
    }

    fn search(&self, text: &str) -> Vec<DBusSearchResult> {
        self.search_index.create_handle()
            .search(text)
            .unwrap()
            .into_iter()
            .map(|item| {
                DBusSearchResult {
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_id: item.entrypoint_id,
                    plugin_name: item.plugin_name,
                    plugin_uuid: item.plugin_id,
                }
            })
            .collect()
    }
}

#[zbus::dbus_proxy(
    default_service = "org.placeholdername.PlaceHolderName",
    default_path = "/org/placeholdername/PlaceHolderName",
    interface = "org.placeholdername.PlaceHolderName",
)]
trait DbusServerProxy {
    async fn plugins(&self) -> zbus::Result<Vec<String>>;
    async fn search(&self, text: &str) -> zbus::Result<Vec<DBusSearchResult>>;
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[zvariant(signature = "(ssss)")]
pub struct DBusSearchResult {
    pub plugin_uuid: String,
    pub plugin_name: String,
    pub entrypoint_id: String,
    pub entrypoint_name: String,
}


#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DBusUiWidget {
    pub widget_id: UiWidgetId,
}

impl From<UiWidget> for DBusUiWidget {
    fn from(value: UiWidget) -> Self {
        Self {
            widget_id: value.widget_id
        }
    }
}

impl From<DBusUiWidget> for UiWidget {
    fn from(value: DBusUiWidget) -> Self {
        Self {
            widget_id: value.widget_id
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
// #[zvariant(signature = "({s(u)}{s(uv)})")] // TODO create issue, for better error reporting
pub struct DBusUiPropertyContainer {
    pub zero: HashMap<String, DBusUiPropertyZeroValue>,
    pub one: HashMap<String, DBusUiPropertyOneValue>,
}

impl From<HashMap<String, UiPropertyValue>> for DBusUiPropertyContainer {
    fn from(value: HashMap<String, UiPropertyValue>) -> Self {
        let properties_one: HashMap<_, _> = value.iter()
            .filter_map(|(key, value)| {
                match value {
                    UiPropertyValue::Function => None,
                    UiPropertyValue::String(value) => Some((key.to_owned(), DBusUiPropertyOneValue::String(value.to_owned()))),
                    UiPropertyValue::Number(value) => Some((key.to_owned(), DBusUiPropertyOneValue::Number(value.to_owned()))),
                    UiPropertyValue::Bool(value) => Some((key.to_owned(), DBusUiPropertyOneValue::Bool(value.to_owned()))),
                }
            })
            .collect();

        let properties_zero: HashMap<_, _> = value.iter()
            .filter_map(|(key, value)| {
                match value {
                    UiPropertyValue::Function => Some((key.to_owned(), DBusUiPropertyZeroValue::Function)),
                    UiPropertyValue::String(_) => None,
                    UiPropertyValue::Number(_) => None,
                    UiPropertyValue::Bool(_) => None,
                }
            })
            .collect();


        DBusUiPropertyContainer { one: properties_one, zero: properties_zero }
    }
}

impl From<DBusUiPropertyContainer> for HashMap<String, UiPropertyValue> {
    fn from(value: DBusUiPropertyContainer) -> Self {
        let properties_one: HashMap<_, _> = value.one
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    DBusUiPropertyOneValue::String(value) => UiPropertyValue::String(value),
                    DBusUiPropertyOneValue::Number(value) => UiPropertyValue::Number(value),
                    DBusUiPropertyOneValue::Bool(value) => UiPropertyValue::Bool(value),
                };

                (key, value)
            })
            .collect();

        let mut properties: HashMap<_, _> = value.zero
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    DBusUiPropertyZeroValue::Function => UiPropertyValue::Function,
                };

                (key, value)
            })
            .collect();

        properties.extend(properties_one);

        properties
    }
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