use std::fmt::Debug;

use crate::common::dbus::{DBusEntrypoint, DbusEventViewCreated, DbusEventViewEvent, DBusPlugin, DBusSearchResult, DBusUiPropertyContainer, DBusUiWidget};
use crate::server::plugins::PluginManager;
use crate::server::search::SearchIndex;

pub struct DbusServer {
    pub search_index: SearchIndex,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName")]
impl DbusServer {
    fn search(&self, text: &str) -> Vec<DBusSearchResult> {
        self.search_index.create_handle()
            .search(text)
            .unwrap()
            .into_iter()
            .map(|item| {
                DBusSearchResult {
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_uuid: item.entrypoint_uuid,
                    plugin_name: item.plugin_name,
                    plugin_uuid: item.plugin_id,
                }
            })
            .collect()
    }
}


pub struct DbusManagementServer {
    pub plugin_manager: PluginManager,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName.Management")]
impl DbusManagementServer {
    fn plugins(&mut self) -> Vec<DBusPlugin> {
        self.plugin_manager.plugins()
            .iter()
            .map(|plugin| DBusPlugin {
                plugin_uuid: plugin.id().to_owned(),
                plugin_name: plugin.name().to_owned(),
                entrypoints: plugin.entrypoints()
                    .into_iter()
                    .map(|entrypoint| DBusEntrypoint {
                        entrypoint_uuid: entrypoint.id().to_owned(),
                        entrypoint_name: entrypoint.name().to_owned()
                    })
                    .collect()
            })
            .collect()
    }
}


#[zbus::dbus_proxy(
default_service = "org.placeholdername.PlaceHolderName.Client",
default_path = "/org/placeholdername/PlaceHolderName",
interface = "org.placeholdername.PlaceHolderName.Client",
)]
trait DbusClientProxy {
    #[dbus_proxy(signal)]
    fn view_created_signal(&self, plugin_uuid: &str, event: DbusEventViewCreated) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn view_event_signal(&self, plugin_uuid: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    fn get_container(&self, plugin_uuid: &str) -> zbus::Result<DBusUiWidget>;

    fn create_instance(&self, plugin_uuid: &str, widget_type: &str, properties: DBusUiPropertyContainer) -> zbus::Result<DBusUiWidget>;

    fn create_text_instance(&self, plugin_uuid: &str, text: &str) -> zbus::Result<DBusUiWidget>;

    fn append_child(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn remove_child(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn insert_before(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) -> zbus::Result<()>;

    fn set_properties(&self, plugin_uuid: &str, widget: DBusUiWidget, properties: DBusUiPropertyContainer) -> zbus::Result<()>;

    fn set_text(&self, plugin_uuid: &str, widget: DBusUiWidget, text: &str) -> zbus::Result<()>;

    fn clone_instance(&self, plugin_uuid: &str, widget_type: &str, properties: DBusUiPropertyContainer) -> zbus::Result<DBusUiWidget>;

    fn replace_container_children(&self, plugin_uuid: &str, container: DBusUiWidget, new_children: Vec<DBusUiWidget>) -> zbus::Result<()>;
}
