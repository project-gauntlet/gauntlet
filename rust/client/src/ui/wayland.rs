use smithay_client_toolkit::reexports::client::Connection;
use smithay_client_toolkit::reexports::client::Dispatch;
use smithay_client_toolkit::reexports::client::QueueHandle;
use smithay_client_toolkit::reexports::client::globals::GlobalListContents;
use smithay_client_toolkit::reexports::client::globals::registry_queue_init;
use smithay_client_toolkit::reexports::client::protocol::wl_registry;

struct WaylandState;

pub fn layer_shell_supported() -> bool {
    let Ok(conn) = Connection::connect_to_env() else {
        return false;
    };

    let Ok((globals, _)) = registry_queue_init::<WaylandState>(&conn) else {
        return false;
    };

    globals
        .contents()
        .clone_list()
        .iter()
        .find(|global| global.interface == "zwlr_layer_shell_v1")
        .is_some()
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut WaylandState,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
    }
}
