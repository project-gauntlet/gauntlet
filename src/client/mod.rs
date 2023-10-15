pub(in crate::client) mod dbus;
pub(in crate::client) mod ui;
pub(in crate::client) mod model;

pub fn start_client() {
    ui::run();
}

