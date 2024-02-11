use zbus::DBusError;

use common::dbus::{DbusEventRenderView, DbusEventRunCommand, DbusEventViewEvent, DBusSearchResult, DBusUiWidget};
use common::model::PluginId;
use utils::channel::RequestSender;

use crate::model::{NativeUiRequestData, NativeUiResponseData, NativeUiWidget};

pub struct DbusClient {
    pub(crate) context_tx: RequestSender<(PluginId, NativeUiRequestData), NativeUiResponseData>
}

#[zbus::dbus_interface(name = "dev.projectgauntlet.Client")]
impl DbusClient {
    #[dbus_interface(signal)]
    pub async fn render_view_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str, event: DbusEventRenderView) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn run_command_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str, event: DbusEventRunCommand) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn view_event_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    async fn replace_container_children(&self, plugin_id: &str, top_level_view: bool, container: DBusUiWidget) -> Result<()> {
        let container = container.try_into()
            .expect("unable to convert widget into native");

        let data = NativeUiRequestData::ReplaceContainerChildren { top_level_view, container };
        let data = (PluginId::from_string(plugin_id), data);

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::ReplaceContainerChildren => {},
        };

        Ok(())
    }
}

type Result<T> = core::result::Result<T, ClientError>;

#[derive(DBusError, Debug)]
#[dbus_error(prefix = "dev.projectgauntlet.Client.Error")]
enum ClientError {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
    ClientError(String),
}

impl From<anyhow::Error> for ClientError {
    fn from(result: anyhow::Error) -> Self {
        ClientError::ClientError(result.to_string())
    }
}

#[zbus::dbus_proxy(
    default_service = "dev.projectgauntlet.Gauntlet",
    default_path = "/dev/projectgauntlet/Server",
    interface = "dev.projectgauntlet.Server",
)]
trait DbusServerProxy {
    async fn search(&self, text: &str) -> zbus::Result<Vec<DBusSearchResult>>;
}

