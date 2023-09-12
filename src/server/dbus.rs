use std::fmt::Debug;

use crate::dbus::{DbusEventViewCreated, DbusEventViewEvent, DBusSearchResult, DBusUiPropertyContainer, DBusUiWidget};
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

    fn create_instance(&self, plugin_uuid: &str, widget_type: &str) -> zbus::Result<DBusUiWidget>;

    fn create_text_instance(&self, plugin_uuid: &str, text: &str) -> zbus::Result<DBusUiWidget>;

    fn append_child(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn remove_child(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn insert_before(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) -> zbus::Result<()>;

    fn set_properties(&self, plugin_uuid: &str, widget: DBusUiWidget, properties: DBusUiPropertyContainer) -> zbus::Result<()>;

    fn set_text(&self, plugin_uuid: &str, widget: DBusUiWidget, text: &str) -> zbus::Result<()>;
}
