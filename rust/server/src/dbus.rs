use std::fmt::Debug;

use zbus::DBusError;

use common::dbus::{DBusEntrypointType, DbusEventRenderView, DbusEventRunCommand, DbusEventViewEvent, DBusPlugin, DBusSearchResult, DBusUiWidget, RenderLocation};
use common::model::{EntrypointId, PluginId};

use crate::model::PluginEntrypointType;
use crate::plugins::ApplicationManager;
use crate::search::SearchIndex;

pub struct DbusServer {
    pub search_index: SearchIndex,
    pub application_manager: ApplicationManager,
}

#[zbus::dbus_interface(name = "dev.projectgauntlet.Server")]
impl DbusServer {
    fn search(&self, text: &str) -> Result<Vec<DBusSearchResult>> {
        let result = self.search_index.create_handle()
            .search(text)?
            .into_iter()
            .flat_map(|item| {
                let entrypoint_type = match item.entrypoint_type {
                    PluginEntrypointType::Command => DBusEntrypointType::Command,
                    PluginEntrypointType::View => DBusEntrypointType::View,
                    PluginEntrypointType::InlineView => {
                        return None
                    }
                };

                Some(DBusSearchResult {
                    entrypoint_type,
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_id: item.entrypoint_id,
                    plugin_name: item.plugin_name,
                    plugin_id: item.plugin_id,
                })
            })
            .collect();

        self.application_manager.handle_inline_view(text);

        Ok(result)
    }

    #[dbus_interface(signal)]
    pub async fn remote_plugin_download_finished_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str) -> zbus::Result<()>;

    async fn download_and_save_plugin(
        &mut self,
        #[zbus(signal_context)]
        signal_context: zbus::SignalContext<'_>,
        plugin_id: &str,
    ) -> Result<()> {
        let result = self.application_manager.download_and_save_plugin(signal_context, PluginId::from_string(plugin_id))
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "dbus", "error occurred when handling 'download_and_save_plugin' request {:?}", err)
        }

        result.map_err(|err| err.into())
    }

    async fn save_local_plugin(&mut self, path: &str) -> Result<()> {
        let result = self.application_manager.save_local_plugin(path)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "dbus", "error occurred when handling 'save_local_plugin' request {:?}", err)
        }

        result.map_err(|err| err.into())
    }

    async fn plugins(&self) -> Result<Vec<DBusPlugin>> {
        let result = self.application_manager.plugins()
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "dbus", "error occurred when handling 'plugins' request {:?}", err)
        }

        result.map_err(|err| err.into())
    }

    async fn set_plugin_state(&mut self, plugin_id: &str, enabled: bool) -> Result<()> {
        let result = self.application_manager.set_plugin_state(PluginId::from_string(plugin_id), enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "dbus", "error occurred when handling 'set_plugin_state' request {:?}", err)
        }

        result.map_err(|err| err.into())
    }

    async fn set_entrypoint_state(&mut self, plugin_id: &str, entrypoint_id: &str, enabled: bool) -> Result<()> {
        let result = self.application_manager.set_entrypoint_state(PluginId::from_string(plugin_id), EntrypointId::new(entrypoint_id), enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "dbus", "error occurred when handling 'set_entrypoint_state' request {:?}", err)
        }

        result.map_err(|err| err.into())
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
    fn render_view_signal(&self, plugin_id: &str, event: DbusEventRenderView) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn run_command_signal(&self, plugin_id: &str, event: DbusEventRunCommand) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn view_event_signal(&self, plugin_id: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    fn replace_view(&self, plugin_id: &str, render_location: RenderLocation, top_level_view: bool, container: DBusUiWidget) -> zbus::Result<()>;

    fn clear_inline_view(&self, plugin_id: &str) -> zbus::Result<()>;
}
