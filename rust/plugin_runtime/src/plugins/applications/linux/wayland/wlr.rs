use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use smithay_client_toolkit::reexports::calloop::channel::Sender;
use smithay_client_toolkit::seat::SeatState;
use tokio::runtime::Handle;
use crate::plugins::applications::linux::wayland::{send_event, JsWaylandApplicationEvent, WaylandState, WaylandStateInner};
use wayland_client::globals::GlobalList;
use wayland_client::{event_created_child, Connection, Dispatch, Proxy, QueueHandle};
use wayland_client::backend::ObjectId;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_protocols_wlr::foreign_toplevel::v1::client::{zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1};

pub struct WlrWaylandState {
    uuid_to_obj_id: HashMap<String, ObjectId>,
    obj_id_to_uuid: HashMap<ObjectId, String>,
    toplevels: HashMap<ObjectId, zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1>,
}

impl WlrWaylandState {
    pub fn new(globals: &GlobalList, queue_handle: &QueueHandle<WaylandState>) -> anyhow::Result<Self> {
        let _management = globals
            .bind::<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _, _>(
                &queue_handle,
                3..=3,
                (),
            )?;

        Ok(Self {
            uuid_to_obj_id: HashMap::new(),
            obj_id_to_uuid: HashMap::new(),
            toplevels: HashMap::new(),
        })
    }

    pub fn focus_window(&self, window_uuid: String, seat_state: &SeatState) -> anyhow::Result<()> {
        let obj_id = self.uuid_to_obj_id
            .get(&window_uuid)
            .ok_or(anyhow!("Unable to find object id for window uuid: {}", window_uuid))?;

        let toplevel = self.toplevels
            .get(&obj_id)
            .ok_or(anyhow!("Unable to find object id for window uuid: {}", window_uuid))?;

        match seat_state.seats().next() {
            Some(seat) => toplevel.activate(&seat),
            None => Err(anyhow!("no wayland seats found"))?
        };

        Ok(())
    }
}

impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        event: <zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                match &mut state.inner {
                    WaylandStateInner::Wlr(inner) => {
                        let window_id = uuid::Uuid::new_v4().to_string();

                        inner.uuid_to_obj_id.insert(window_id.clone(), toplevel.id());
                        inner.obj_id_to_uuid.insert(toplevel.id(), window_id.clone());
                        inner.toplevels.insert(toplevel.id(), toplevel);

                        send_event(&state.tokio_handle, &state.sender, JsWaylandApplicationEvent::WindowOpened {
                            window_id,
                        });
                    }
                    WaylandStateInner::Cosmic => {
                        todo!()
                    }
                    WaylandStateInner::None => {}
                }
            }
            _ => {}
        }
    }

    event_created_child!(WaylandState, zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, [
        zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        proxy: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        event: <zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                match &state.inner {
                    WaylandStateInner::Wlr(inner) => {
                        match inner.obj_id_to_uuid.get(&proxy.id()) {
                            Some(window_id) => {
                                send_event(&state.tokio_handle, &state.sender, JsWaylandApplicationEvent::WindowTitleChanged {
                                    window_id: window_id.clone(),
                                    title,
                                });
                            }
                            None => {
                                tracing::warn!("Received event for wlr wayland toplevel that doesn't exist in state");
                            }
                        }
                    }
                    WaylandStateInner::Cosmic => {
                        todo!()
                    }
                    WaylandStateInner::None => {}
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                match &state.inner {
                    WaylandStateInner::Wlr(inner) => {
                        match inner.obj_id_to_uuid.get(&proxy.id()) {
                            Some(window_id) => {
                                send_event(&state.tokio_handle, &state.sender, JsWaylandApplicationEvent::WindowAppIdChanged {
                                    window_id: window_id.clone(),
                                    app_id,
                                });
                            }
                            None => {
                                tracing::warn!("Received event for wlr wayland toplevel that doesn't exist in state");
                            }
                        }
                    }
                    WaylandStateInner::Cosmic => {
                        todo!()
                    }
                    WaylandStateInner::None => {}
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                match &mut state.inner {
                    WaylandStateInner::Wlr(inner) => {

                        inner.toplevels.remove(&proxy.id());
                        match inner.obj_id_to_uuid.remove(&proxy.id()) {
                            Some(window_id) => {
                                inner.uuid_to_obj_id.remove(&window_id);

                                send_event(&state.tokio_handle, &state.sender, JsWaylandApplicationEvent::WindowClosed {
                                    window_id: window_id.clone(),
                                });
                            }
                            None => {
                                tracing::warn!("Received event for wlr wayland toplevel that doesn't exist in state");
                            }
                        }
                    }
                    WaylandStateInner::Cosmic => {
                        todo!()
                    }
                    WaylandStateInner::None => {}
                }
            }
            _ => {}
        }
    }
}