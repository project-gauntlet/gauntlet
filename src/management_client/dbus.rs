use crate::common::dbus::DBusPlugin;

#[zbus::dbus_proxy(
    default_service = "org.placeholdername.PlaceHolderName",
    default_path = "/org/placeholdername/PlaceHolderName/Management",
    interface = "org.placeholdername.PlaceHolderName.Management",
)]
trait DbusManagementServerProxy {
    #[dbus_proxy(signal)]
    fn remote_plugin_download_finished_signal(&self, plugin_id: &str) -> zbus::Result<()>;

    async fn new_remote_plugin(&self, plugin_id: &str) -> zbus::Result<()>;
    async fn plugins(&self) -> zbus::Result<Vec<DBusPlugin>>;
    async fn set_plugin_state(&self, plugin_id: &str, enabled: bool) -> zbus::Result<()>;
    async fn set_entrypoint_state(&self, plugin_id: &str, entrypoint_id: &str, enabled: bool) -> zbus::Result<()>;
}

