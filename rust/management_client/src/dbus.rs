use common::dbus::DBusPlugin;

#[zbus::dbus_proxy(
    default_service = "dev.projectgauntlet.Gauntlet",
    default_path = "/dev/projectgauntlet/Server",
    interface = "dev.projectgauntlet.Server.Management",
)]
trait DbusManagementServerProxy {
    #[dbus_proxy(signal)]
    fn remote_plugin_download_finished_signal(&self, plugin_id: &str) -> zbus::Result<()>;

    async fn download_and_save_plugin(&self, plugin_id: &str) -> zbus::Result<()>;
    async fn plugins(&self) -> zbus::Result<Vec<DBusPlugin>>;
    async fn set_plugin_state(&self, plugin_id: &str, enabled: bool) -> zbus::Result<()>;
    async fn set_entrypoint_state(&self, plugin_id: &str, entrypoint_id: &str, enabled: bool) -> zbus::Result<()>;
}

