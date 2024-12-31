use std::cell::RefCell;
use std::rc::Rc;
use anyhow::anyhow;
use deno_core::{op2, OpState};
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::reexports::calloop;
use smithay_client_toolkit::reexports::calloop::{EventLoop, InsertError, RegistrationToken};
use smithay_client_toolkit::reexports::calloop::channel::{Channel, Event};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use tokio::runtime::Handle;
use tokio::sync::mpsc::{Receiver, Sender};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_client::globals::{registry_queue_init, GlobalList, GlobalListContents};
use wayland_client::protocol::wl_registry;
use wayland_client::protocol::wl_seat::WlSeat;
use crate::plugins::applications::{linux, ApplicationContext, DesktopEnvironment};

pub mod wlr;

pub struct WaylandDesktopEnvironment {
    activate_sender: calloop::channel::Sender<String>,
    event_receiver: Rc<RefCell<Receiver<JsWaylandApplicationEvent>>>,
}

impl WaylandDesktopEnvironment {
    pub fn new() -> anyhow::Result<WaylandDesktopEnvironment> {
        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(100);
        let (activate_sender, activate_receiver) = calloop::channel::channel();

        let environment = WaylandDesktopEnvironment {
            activate_sender,
            event_receiver: Rc::new(RefCell::new(event_receiver)),
        };

        let handle = Handle::current();

        std::thread::spawn(|| {
            if let Err(e) = run_wayland_client(handle, event_sender, activate_receiver) {
                tracing::error!("Error while running wayland client: {:?}", e);
            }
        });

        Ok(environment)
    }

    pub fn focus_window(&self, window_uuid: String) -> anyhow::Result<()> {
        self.activate_sender.send(window_uuid)?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsWaylandApplicationEvent {
    WindowOpened {
        window_id: String
    },
    WindowClosed {
        window_id: String
    },
    WindowTitleChanged {
        window_id: String,
        title: String,
    },
    WindowAppIdChanged {
        window_id: String,
        app_id: String
    }
}

pub struct WaylandState {
    seat_state: SeatState,
    tokio_handle: Handle,
    sender: Sender<JsWaylandApplicationEvent>,
    inner: WaylandStateInner
}

impl WaylandState {
    fn new(
        tokio_handle: Handle,
        sender: Sender<JsWaylandApplicationEvent>,
        seat_state: SeatState,
        globals: &GlobalList,
        queue_handle: &QueueHandle<WaylandState>
    ) -> anyhow::Result<Self> {

        let inner = wlr::WlrWaylandState::new(globals, queue_handle)
            .map(|state| WaylandStateInner::Wlr(state))
            .or_else(|_| anyhow::Ok(WaylandStateInner::None))?;

        Ok(WaylandState {
            seat_state,
            tokio_handle,
            sender,
            inner,
        })
    }
}

pub enum WaylandStateInner {
    Wlr(wlr::WlrWaylandState),
    Cosmic,
    None
}

fn send_event(tokio_handle: &Handle, sender: &Sender<JsWaylandApplicationEvent>, app_event: JsWaylandApplicationEvent) {
    let sender = sender.clone();
    tokio_handle.spawn(async move {
        if let Err(e) = sender.send(app_event).await {
            tracing::error!("Error while sending wayland application event: {:?}", e);
        }
    });
}

fn run_wayland_client(
    tokio_handle: Handle,
    event_sender: Sender<JsWaylandApplicationEvent>,
    activate_receiver: Channel<String>,
) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init::<WaylandState>(&conn)?;

    let mut event_loop = EventLoop::<WaylandState>::try_new()?;
    let queue_handle = event_queue.handle();
    let wayland_source = WaylandSource::new(conn.clone(), event_queue);
    let seat_state = SeatState::new(&globals, &queue_handle);
    let loop_handle = event_loop.handle();

    if let Err(err) = loop_handle.insert_source(activate_receiver, activation_handler) {
        tracing::error!("Unable to insert activation source into event loop: {:?}", err);

        Err(anyhow!("Unable to insert activation source into event loop"))?
    };

    if let Err(err) = wayland_source.insert(loop_handle) {
        tracing::error!("Unable to insert wayland source into event loop: {:?}", err);

        Err(anyhow!("Unable to insert wayland source into event loop"))?
    };

    let mut state = WaylandState::new(tokio_handle, event_sender, seat_state, &globals, &queue_handle)?;

    loop {
        if let Err(err) = event_loop.dispatch(None, &mut state) {
            tracing::error!("Wayland event queue has failed: {:?}", err);
            break;
        }
    }

    Ok(())
}

#[op2(fast)]
pub fn linux_wayland_focus_window(state: Rc<RefCell<OpState>>, #[string] window_uuid: String) -> anyhow::Result<()> {
    {
        let state = state.borrow();

        let context = state
            .borrow::<ApplicationContext>();

        match &context.desktop {
            DesktopEnvironment::Linux(linux::LinuxDesktopEnvironment::Wayland(env)) => {
                env.focus_window(window_uuid)?;
            },
            _ => Err(anyhow!("Calling linux_wayland_focus_window on non-wayland platform"))?
        };
    };

    Ok(())
}

fn activation_handler(event: Event<String>, _metadata: &mut (), state: &mut WaylandState) {
    let window_uuid = match event {
        Event::Msg(window_uuid) => window_uuid,
        Event::Closed => panic!("activation source was closed")
    };

    match &state.inner {
        WaylandStateInner::Wlr(wlr) => {
            if let Err(err) = wlr.focus_window(window_uuid, &state.seat_state) {
                tracing::error!("Unable to focus wayland window: {:?}", err);
            };
        }
        WaylandStateInner::Cosmic => {}
        WaylandStateInner::None => {
            tracing::error!("Calling focus window when there is no supported wayland protocols available");
        }
    }
}


#[op2(async)]
#[serde]
pub async fn application_wayland_pending_event(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsWaylandApplicationEvent> {
    let receiver = {
        let state = state.borrow();

        let context = state
            .borrow::<ApplicationContext>();

        match &context.desktop {
            DesktopEnvironment::Linux(linux::LinuxDesktopEnvironment::Wayland(env)) => {
                env.event_receiver.clone()
            },
            _ => Err(anyhow!("Calling application_wayland_pending_event on non-wayland platform"))?
        }
    };

    let mut receiver = receiver.borrow_mut();
    let event = receiver.recv()
        .await
        .ok_or_else(|| anyhow!("plugin event stream was suddenly closed"))?;

    tracing::trace!("Received application event {:?}", event);

    Ok(event)
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

impl SeatHandler for WaylandState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {
    }

    fn new_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat, _capability: Capability) {
    }

    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat, _capability: Capability) {
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {
    }
}

smithay_client_toolkit::delegate_seat!(WaylandState);
