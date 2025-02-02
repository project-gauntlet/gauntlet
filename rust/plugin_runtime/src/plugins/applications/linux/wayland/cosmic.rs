use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1;
use cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_info_v1;
use cosmic_protocols::toplevel_management::v1::client::zcosmic_toplevel_manager_v1;
use smithay_client_toolkit::reexports::calloop::channel::Sender;
use smithay_client_toolkit::seat::SeatState;
use tokio::runtime::Handle;
use wayland_client::backend::ObjectId;
use wayland_client::event_created_child;
use wayland_client::globals::GlobalList;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::Proxy;
use wayland_client::QueueHandle;

use crate::plugins::applications::linux::wayland::send_event;
use crate::plugins::applications::linux::wayland::JsWaylandApplicationEvent;
use crate::plugins::applications::linux::wayland::WaylandState;
use crate::plugins::applications::linux::wayland::WaylandStateInner;

pub struct CosmicWaylandState {
    uuid_to_obj_id: HashMap<String, ObjectId>,
    obj_id_to_uuid: HashMap<ObjectId, String>,
    toplevels: HashMap<ObjectId, zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1>,
    management: zcosmic_toplevel_manager_v1::ZcosmicToplevelManagerV1,
}

impl CosmicWaylandState {
    pub fn new(globals: &GlobalList, queue_handle: &QueueHandle<WaylandState>) -> anyhow::Result<Self> {
        let management =
            globals.bind::<zcosmic_toplevel_manager_v1::ZcosmicToplevelManagerV1, _, _>(&queue_handle, 3..=3, ())?;

        Ok(Self {
            management,
            uuid_to_obj_id: HashMap::new(),
            obj_id_to_uuid: HashMap::new(),
            toplevels: HashMap::new(),
        })
    }

    pub fn focus_window(&self, window_uuid: String, seat_state: &SeatState) -> anyhow::Result<()> {
        let obj_id = self
            .uuid_to_obj_id
            .get(&window_uuid)
            .ok_or(anyhow!("Unable to find object id for window uuid: {}", window_uuid))?;

        let toplevel = self
            .toplevels
            .get(&obj_id)
            .ok_or(anyhow!("Unable to find object id for window uuid: {}", window_uuid))?;

        match seat_state.seats().next() {
            Some(seat) => self.management.activate(&toplevel, &seat),
            None => Err(anyhow!("no wayland seats found"))?,
        };

        Ok(())
    }
}

impl Dispatch<zcosmic_toplevel_manager_v1::ZcosmicToplevelManagerV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &zcosmic_toplevel_manager_v1::ZcosmicToplevelManagerV1,
        _event: <zcosmic_toplevel_manager_v1::ZcosmicToplevelManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zcosmic_toplevel_info_v1::ZcosmicToplevelInfoV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &zcosmic_toplevel_info_v1::ZcosmicToplevelInfoV1,
        event: <zcosmic_toplevel_info_v1::ZcosmicToplevelInfoV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zcosmic_toplevel_info_v1::Event::Toplevel { toplevel } => {
                match &mut state.inner {
                    WaylandStateInner::Cosmic(inner) => {
                        let window_id = uuid::Uuid::new_v4().to_string();

                        inner.uuid_to_obj_id.insert(window_id.clone(), toplevel.id());
                        inner.obj_id_to_uuid.insert(toplevel.id(), window_id.clone());
                        inner.toplevels.insert(toplevel.id(), toplevel);

                        send_event(
                            &state.tokio_handle,
                            &state.sender,
                            JsWaylandApplicationEvent::WindowOpened { window_id },
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    event_created_child!(WaylandState, zcosmic_toplevel_info_v1::ZcosmicToplevelInfoV1, [
        zcosmic_toplevel_info_v1::EVT_TOPLEVEL_OPCODE => (zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1, ()),
    ]);
}

impl Dispatch<zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        proxy: &zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1,
        event: <zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zcosmic_toplevel_handle_v1::Event::Title { title } => {
                match &state.inner {
                    WaylandStateInner::Cosmic(inner) => {
                        match inner.obj_id_to_uuid.get(&proxy.id()) {
                            Some(window_id) => {
                                send_event(
                                    &state.tokio_handle,
                                    &state.sender,
                                    JsWaylandApplicationEvent::WindowTitleChanged {
                                        window_id: window_id.clone(),
                                        title,
                                    },
                                );
                            }
                            None => {
                                tracing::warn!(
                                    "Received event for cosmic wayland toplevel that doesn't exist in state"
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            zcosmic_toplevel_handle_v1::Event::AppId { app_id } => {
                match &state.inner {
                    WaylandStateInner::Cosmic(inner) => {
                        match inner.obj_id_to_uuid.get(&proxy.id()) {
                            Some(window_id) => {
                                send_event(
                                    &state.tokio_handle,
                                    &state.sender,
                                    JsWaylandApplicationEvent::WindowAppIdChanged {
                                        window_id: window_id.clone(),
                                        app_id,
                                    },
                                );
                            }
                            None => {
                                tracing::warn!(
                                    "Received event for cosmic wayland toplevel that doesn't exist in state"
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            zcosmic_toplevel_handle_v1::Event::Closed => {
                match &mut state.inner {
                    WaylandStateInner::Cosmic(inner) => {
                        inner.toplevels.remove(&proxy.id());
                        match inner.obj_id_to_uuid.remove(&proxy.id()) {
                            Some(window_id) => {
                                inner.uuid_to_obj_id.remove(&window_id);

                                send_event(
                                    &state.tokio_handle,
                                    &state.sender,
                                    JsWaylandApplicationEvent::WindowClosed {
                                        window_id: window_id.clone(),
                                    },
                                );
                            }
                            None => {
                                tracing::warn!(
                                    "Received event for cosmic wayland toplevel that doesn't exist in state"
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
