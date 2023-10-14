pub(in crate::client) mod dbus;
pub(in crate::client) mod native_ui;
pub(in crate::client) mod model;

pub fn start_client() {
    native_ui::run();
}

