use crate::common::dbus::DBusPlugin;

#[zbus::dbus_proxy(
    default_service = "org.placeholdername.PlaceHolderName",
    default_path = "/org/placeholdername/PlaceHolderName/Management",
    interface = "org.placeholdername.PlaceHolderName.Management",
)]
trait DbusManagementServerProxy {
    async fn plugins(&self) -> zbus::Result<Vec<DBusPlugin>>;
    async fn set_plugin_state(&self, plugin_id: &str, enabled: bool) -> zbus::Result<()>;
    async fn set_entrypoint_state(&self, plugin_id: &str, entrypoint_id: &str, enabled: bool) -> zbus::Result<()>;
}

