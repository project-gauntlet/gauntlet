use std::fmt::Debug;
use zbus::DBusError;

use common::dbus::{DbusEventViewCreated, DbusEventViewEvent, DBusPlugin, DBusSearchResult, DBusUiPropertyContainer, DBusUiWidget};
use common::model::{EntrypointId, PluginId};
use crate::plugins::ApplicationManager;
use crate::search::SearchIndex;

pub struct DbusServer {
    pub search_index: SearchIndex,
}

#[zbus::dbus_interface(name = "dev.projectgauntlet.Server")]
impl DbusServer {
    fn search(&self, text: &str) -> Result<Vec<DBusSearchResult>> {
        let result = self.search_index.create_handle()
            .search(text)?
            .into_iter()
            .map(|item| {
                DBusSearchResult {
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_id: item.entrypoint_id,
                    plugin_name: item.plugin_name,
                    plugin_id: item.plugin_id,
                }
            })
            .collect();

        Ok(result)
    }
}


pub struct DbusManagementServer {
    pub application_manager: ApplicationManager,
}

#[zbus::dbus_interface(name = "dev.projectgauntlet.Server.Management")]
impl DbusManagementServer {

    #[dbus_interface(signal)]
    pub async fn remote_plugin_download_finished_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str) -> zbus::Result<()>;

    async fn new_remote_plugin(
        &mut self,
        #[zbus(signal_context)]
        signal_context: zbus::SignalContext<'_>,
        plugin_id: &str
    ) -> Result<()> {
        self.application_manager.new_remote_plugin(signal_context, PluginId::from_string(plugin_id))
            .await
            .map_err(|err| err.into())
    }

    async fn plugins(&self) -> Result<Vec<DBusPlugin>> {
        self.application_manager.plugins()
            .await
            .map_err(|err| err.into())
    }

    async fn set_plugin_state(&mut self, plugin_id: &str, enabled: bool) -> Result<()> {
        self.application_manager.set_plugin_state(PluginId::from_string(plugin_id), enabled)
            .await
            .map_err(|err| err.into())
    }

    async fn set_entrypoint_state(&mut self, plugin_id: &str, entrypoint_id: &str, enabled: bool) -> Result<()> {
        self.application_manager.set_entrypoint_state(PluginId::from_string(plugin_id), EntrypointId::new(entrypoint_id), enabled)
            .await
            .map_err(|err| err.into())
    }
}

type Result<T> = core::result::Result<T, ServerError>;

#[derive(DBusError, Debug)]
#[dbus_error(prefix = "dev.projectgauntlet.Server.Error")]
enum ServerError {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
    ServerError(String),
}

impl From<anyhow::Error> for ServerError {
    fn from(result: anyhow::Error) -> Self {
        ServerError::ServerError(result.to_string())
    }
}


#[zbus::dbus_proxy(
    default_service = "dev.projectgauntlet.Gauntlet.Client",
    default_path = "/dev/projectgauntlet/Client",
    interface = "dev.projectgauntlet.Client",
)]
trait DbusClientProxy {
    #[dbus_proxy(signal)]
    fn view_created_signal(&self, plugin_id: &str, event: DbusEventViewCreated) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn view_event_signal(&self, plugin_id: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    fn get_root(&self, plugin_id: &str) -> zbus::Result<DBusUiWidget>;

    fn create_instance(&self, plugin_id: &str, widget_type: &str, properties: DBusUiPropertyContainer) -> zbus::Result<DBusUiWidget>;

    fn create_text_instance(&self, plugin_id: &str, text: &str) -> zbus::Result<DBusUiWidget>;

    fn append_child(&self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn remove_child(&self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn insert_before(&self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) -> zbus::Result<()>;

    fn set_properties(&self, plugin_id: &str, widget: DBusUiWidget, properties: DBusUiPropertyContainer) -> zbus::Result<()>;

    fn set_text(&self, plugin_id: &str, widget: DBusUiWidget, text: &str) -> zbus::Result<()>;

    fn clone_instance(&self, plugin_id: &str, widget: DBusUiWidget, update_payload: Vec<String>, widget_type: &str, old_props: DBusUiPropertyContainer, new_props: DBusUiPropertyContainer, keep_children: bool) -> zbus::Result<DBusUiWidget>;

    fn replace_container_children(&self, plugin_id: &str, container: DBusUiWidget, new_children: Vec<DBusUiWidget>) -> zbus::Result<()>;
}
