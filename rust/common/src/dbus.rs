use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{DeserializeDict, OwnedValue, SerializeDict, Type, Value};

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

#[derive(Debug, DeserializeDict, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct DBusUiWidget {
    pub widget_id: DbusUiWidgetId,
    pub widget_type: String,
    pub widget_properties: DBusUiPropertyContainer,
    pub widget_children: Vec<DBusUiWidget>,
}


#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DbusEventViewCreated {
    pub reconciler_mode: String,
    pub view_name: String,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct DbusEventViewEvent {
    pub widget_id: DbusUiWidgetId,
    pub event_name: DbusUiEventName,
    pub event_arguments: Vec<DBusUiPropertyValue>,
}

pub type DbusUiWidgetId = u32;
pub type DbusUiEventName = String;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusUiPropertyContainer(pub HashMap<String, DBusUiPropertyValue>);

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum DBusUiPropertyValueType {
    Undefined,
    String,
    Number,
    Bool,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DBusUiPropertyValue(pub DBusUiPropertyValueType, pub OwnedValue);

pub fn value_undefined_to_dbus() -> DBusUiPropertyValue {
    DBusUiPropertyValue(DBusUiPropertyValueType::Undefined, Value::U8(0).to_owned())
}

pub fn value_string_to_dbus(value: String) -> DBusUiPropertyValue {
    DBusUiPropertyValue(DBusUiPropertyValueType::String, Value::Str(value.into()).to_owned())
}

pub fn value_number_to_dbus(value: f64) -> DBusUiPropertyValue {
    DBusUiPropertyValue(DBusUiPropertyValueType::Number, Value::F64(value.into()).to_owned())
}

pub fn value_bool_to_dbus(value: bool) -> DBusUiPropertyValue {
    DBusUiPropertyValue(DBusUiPropertyValueType::Bool, Value::Bool(value.into()).to_owned())
}
