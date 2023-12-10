use zbus::DBusError;
use crate::model::{from_dbus, NativeUiRequestData, NativeUiResponseData};
use common::dbus::{DbusEventViewCreated, DbusEventViewEvent, DBusSearchResult, DBusUiPropertyContainer, DBusUiWidget};
use common::model::PluginId;
use utils::channel::RequestSender;

pub struct DbusClient {
    pub(crate) context_tx: RequestSender<(PluginId, NativeUiRequestData), NativeUiResponseData>
}

#[zbus::dbus_interface(name = "dev.projectgauntlet.Client")]
impl DbusClient {
    #[dbus_interface(signal)]
    pub async fn view_created_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str, event: DbusEventViewCreated) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn view_event_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_id: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    async fn get_root(&mut self, plugin_id: &str) -> Result<DBusUiWidget> {
        let input = (PluginId::from_string(plugin_id), NativeUiRequestData::GetRoot);

        match self.context_tx.send_receive(input).await {
            NativeUiResponseData::GetRoot { container } => Ok(container.into()),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_instance(&mut self, plugin_id: &str, widget_type: &str, properties: DBusUiPropertyContainer) -> Result<DBusUiWidget> {
        let data = NativeUiRequestData::CreateInstance { widget_type: widget_type.to_owned(), properties: from_dbus(properties)? };
        let input = (PluginId::from_string(plugin_id), data);

        let widget = match self.context_tx.send_receive(input).await {
            NativeUiResponseData::CreateInstance { widget } => widget?.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        };

        Ok(widget)
    }

    async fn create_text_instance(&mut self, plugin_id: &str, text: &str) -> Result<DBusUiWidget> {
        let data = NativeUiRequestData::CreateTextInstance { text: text.to_owned() };
        let input = (PluginId::from_string(plugin_id), data);

        let widget = match self.context_tx.send_receive(input).await {
            NativeUiResponseData::CreateTextInstance { widget } => widget?.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        };

        Ok(widget)
    }

    async fn append_child(&mut self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget) -> Result<()> {
        let data = NativeUiRequestData::AppendChild { parent: parent.into(), child: child.into() };
        let input = (PluginId::from_string(plugin_id), data);

        match self.context_tx.send_receive(input).await {
            NativeUiResponseData::AppendChild { result } => result?,
            value @ _ => panic!("unsupported response type {:?}", value),
        };

        Ok(())
    }

    async fn clone_instance(&self, plugin_id: &str, widget: DBusUiWidget, _update_payload: Vec<String>, widget_type: &str, _old_props: DBusUiPropertyContainer, new_props: DBusUiPropertyContainer, keep_children: bool) -> Result<DBusUiWidget> {
        let data = NativeUiRequestData::CloneInstance {
            widget: widget.into(),
            widget_type: widget_type.to_owned(),
            new_props: from_dbus(new_props)?,
            keep_children,
        };
        let input = (PluginId::from_string(plugin_id), data);

        let widget = match self.context_tx.send_receive(input).await {
            NativeUiResponseData::CloneInstance { widget } => widget?.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        };

        Ok(widget)
    }

    async fn replace_container_children(&self, plugin_id: &str, container: DBusUiWidget, new_children: Vec<DBusUiWidget>) -> Result<()> {
        let new_children = new_children.into_iter().map(|child| child.into()).collect();
        let data = NativeUiRequestData::ReplaceContainerChildren { container: container.into(), new_children };
        let data = (PluginId::from_string(plugin_id), data);

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::ReplaceContainerChildren { result } => result?,
            value @ _ => panic!("unsupported response type {:?}", value),
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

