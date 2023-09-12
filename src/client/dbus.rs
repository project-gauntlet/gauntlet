use crate::client::model::{NativeUiRequestData, NativeUiResponseData};
use crate::dbus::{DbusEventViewCreated, DbusEventViewEvent, DBusSearchResult, DBusUiPropertyContainer, DBusUiWidget};
use crate::utils::channel::RequestSender;

pub struct DbusClient {
    pub(crate) channel: RequestSender<(String, NativeUiRequestData), NativeUiResponseData>,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName.Client")]
impl DbusClient {
    #[dbus_interface(signal)]
    pub async fn view_created_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_uuid: &str, event: DbusEventViewCreated) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn view_event_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_uuid: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    async fn get_container(&mut self, plugin_uuid: &str) -> DBusUiWidget {
        let input = (plugin_uuid.to_owned(), NativeUiRequestData::GetContainer);

        match self.channel.send_receive(input).await.unwrap() {
            NativeUiResponseData::GetContainer { container } => DBusUiWidget { widget_id: container.widget_id },
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_instance(&mut self, plugin_uuid: &str, widget_type: &str) -> DBusUiWidget {
        let data = NativeUiRequestData::CreateInstance { widget_type: widget_type.to_owned() };
        let input = (plugin_uuid.to_owned(), data);

        match self.channel.send_receive(input).await.unwrap() {
            NativeUiResponseData::CreateInstance { widget } => DBusUiWidget { widget_id: widget.widget_id },
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_text_instance(&mut self, plugin_uuid: &str, text: &str) -> DBusUiWidget {
        let data = NativeUiRequestData::CreateTextInstance { text: text.to_owned() };
        let input = (plugin_uuid.to_owned(), data);

        match self.channel.send_receive(input).await.unwrap() {
            NativeUiResponseData::CreateTextInstance { widget } => DBusUiWidget { widget_id: widget.widget_id },
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn append_child(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) {
        let data = NativeUiRequestData::AppendChild { parent: parent.into(), child: child.into() };
        let input = (plugin_uuid.to_owned(), data);
        self.channel.send(input).unwrap();
    }

    async fn insert_before(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) {
        let data = NativeUiRequestData::InsertBefore { parent: parent.into(), child: child.into(), before_child: before_child.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    async fn remove_child(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) {
        let data = NativeUiRequestData::RemoveChild { parent: parent.into(), child: child.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    async fn set_properties(
        &mut self,
        plugin_uuid: &str,
        widget: DBusUiWidget,
        properties: DBusUiPropertyContainer,
    ) {
        let data = NativeUiRequestData::SetProperties { widget: widget.into(), properties: properties.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    fn set_text(&mut self, plugin_uuid: &str, widget: DBusUiWidget, text: &str) {
        let data = NativeUiRequestData::SetText { widget: widget.into(), text: text.to_owned() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
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

