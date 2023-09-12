use crate::server::dbus::{DBusUiPropertyContainer, DBusUiWidget};
use crate::server::plugins::js::{UiEventViewCreated, UiEventViewEvent, UiRequestData, UiResponseData};
use crate::utils::channel::RequestSender;

pub struct DbusClient {
    pub(crate) channel: RequestSender<(String, UiRequestData), UiResponseData>,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName.Client")]
impl DbusClient {
    #[dbus_interface(signal)]
    pub async fn view_created_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_uuid: &str, event: UiEventViewCreated) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn view_event_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_uuid: &str, event: UiEventViewEvent) -> zbus::Result<()>;

    async fn get_container(&mut self, plugin_uuid: &str) -> DBusUiWidget {
        let input = (plugin_uuid.to_owned(), UiRequestData::GetContainer);

        match self.channel.send_receive(input).await.unwrap() {
            UiResponseData::GetContainer { container } => container.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_instance(&mut self, plugin_uuid: &str, widget_type: &str) -> DBusUiWidget {
        let data = UiRequestData::CreateInstance { widget_type: widget_type.to_owned() };
        let input = (plugin_uuid.to_owned(), data);

        match self.channel.send_receive(input).await.unwrap() {
            UiResponseData::CreateInstance { widget } => widget.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_text_instance(&mut self, plugin_uuid: &str, text: &str) -> DBusUiWidget {
        let data = UiRequestData::CreateTextInstance { text: text.to_owned() };
        let input = (plugin_uuid.to_owned(), data);

        match self.channel.send_receive(input).await.unwrap() {
            UiResponseData::CreateTextInstance { widget } => widget.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn append_child(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) {
        let data = UiRequestData::AppendChild { parent: parent.into(), child: child.into() };
        let input = (plugin_uuid.to_owned(), data);
        self.channel.send(input).unwrap();
    }

    async fn insert_before(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) {
        let data = UiRequestData::InsertBefore { parent: parent.into(), child: child.into(), before_child: before_child.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    async fn remove_child(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) {
        let data = UiRequestData::RemoveChild { parent: parent.into(), child: child.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    async fn set_properties(
        &mut self,
        plugin_uuid: &str,
        widget: DBusUiWidget,
        properties: DBusUiPropertyContainer,
    ) {
        let data = UiRequestData::SetProperties { widget: widget.into(), properties: properties.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    fn set_text(&mut self, plugin_uuid: &str, widget: DBusUiWidget, text: &str) {
        let data = UiRequestData::SetText { widget: widget.into(), text: text.to_owned() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }
}

#[zbus::dbus_proxy(
    default_service = "org.placeholdername.PlaceHolderName.Client",
    default_path = "/org/placeholdername/PlaceHolderName",
    interface = "org.placeholdername.PlaceHolderName.Client",
)]
trait DbusClientProxy {
    #[dbus_proxy(signal)]
    fn view_created_signal(&self, plugin_uuid: &str, event: UiEventViewCreated) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn view_event_signal(&self, plugin_uuid: &str, event: UiEventViewEvent) -> zbus::Result<()>;

    fn get_container(&self, plugin_uuid: &str) -> zbus::Result<DBusUiWidget>;

    fn create_instance(&self, plugin_uuid: &str, widget_type: &str) -> zbus::Result<DBusUiWidget>;

    fn create_text_instance(&self, plugin_uuid: &str, text: &str) -> zbus::Result<DBusUiWidget>;

    fn append_child(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn remove_child(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn insert_before(&self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) -> zbus::Result<()>;

    fn set_properties(&self, plugin_uuid: &str, widget: DBusUiWidget, properties: DBusUiPropertyContainer) -> zbus::Result<()>;

    fn set_text(&self, plugin_uuid: &str, widget: DBusUiWidget, text: &str) -> zbus::Result<()>;
}