use crate::common::dbus::DBusPlugin;

#[zbus::dbus_proxy(
    default_service = "org.placeholdername.PlaceHolderName",
    default_path = "/org/placeholdername/PlaceHolderName/Management",
    interface = "org.placeholdername.PlaceHolderName.Management",
)]
trait DbusManagementServerProxy {
    async fn plugins(&self) -> zbus::Result<Vec<DBusPlugin>>;
}

