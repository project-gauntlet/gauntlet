use zbus::{blocking, dbus_proxy, Result};

#[dbus_proxy(
    interface = "org.placeholdername.PlaceHolderName",
    default_service = "org.placeholdername.PlaceHolderName",
    default_path = "/org/placeholdername/PlaceHolderName"
)]
trait DbusInterface {
    async fn open_window(&self) -> Result<()>;
}

pub fn run_agent() {
    let connection = blocking::Connection::session().unwrap();

    let proxy = DbusInterfaceProxyBlocking::new(&connection).unwrap();

    proxy.open_window().unwrap();
}
