pub(in crate) mod dbus;
pub(in crate) mod ui;
pub(in crate) mod model;

pub fn start_client() {
    ui::run();
}

