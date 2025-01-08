use crate::plugins::applications::linux::x11;
use anyhow::anyhow;
use deno_core::{op2, OpState};
use encoding::{DecoderTrap, Encoding};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Infallible;
use std::rc::Rc;
use std::str::FromStr;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{Receiver, Sender};
use x11rb::connection::Connection;
use x11rb::errors::ConnectionError;
use x11rb::properties::{WmClass, WmHints};
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::{Atom, AtomEnum, ClientMessageEvent, ConfigureWindowAux, InputFocus, MapState, StackMode, Window};
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, EventMask};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use crate::plugins::applications::{linux, ApplicationContext, DesktopEnvironment};

pub struct X11DesktopEnvironment {
    receiver: Rc<RefCell<Receiver<JsX11ApplicationEvent>>>,
}

impl X11DesktopEnvironment {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        let handle = Handle::current();

        std::thread::spawn(move || {
            if let Err(e) = listen_on_x11_events(handle, sender.clone()) {
                tracing::error!("Error while listening on x11 events: {}", e);
            }
        });

        Self {
            receiver: Rc::new(RefCell::new(receiver)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum JsX11ApplicationEvent {
    Init {
        id: String,
        parent_id: String,
        override_redirect: bool,
        mapped: bool
    },
    CreateNotify {
        id: String,
        parent_id: String,
        override_redirect: bool
    },
    DestroyNotify {
        id: String,
    },
    MapNotify {
        id: String,
    },
    UnmapNotify {
        id: String,
    },
    ReparentNotify {
        id: String,
    },
    TitlePropertyNotify {
        id: String,
        title: String
    },
    ClassPropertyNotify {
        id: String,
        class: String,
        instance: String
    },
    HintsPropertyNotify {
        id: String,
        window_group: Option<String>,
    },
    ProtocolsPropertyNotify {
        id: String,
        protocols: Vec<JSX11WindowProtocol>,
    },
    TransientForPropertyNotify {
        id: String,
        transient_for: Option<String>,
    },
    WindowTypePropertyNotify {
        id: String,
        window_types: Vec<JSX11WindowType>
    },
    DesktopFileNamePropertyNotify {
        id: String,
        desktop_file_name: String
    },
}

#[derive(Debug, Deserialize, Serialize)]
enum JSX11WindowProtocol {
    TakeFocus,
    DeleteWindow,
}

#[derive(Debug, Deserialize, Serialize)]
enum JSX11WindowType {
    DropdownMenu,
    Dialog,
    Menu,
    Notification,
    Normal,
    PopupMenu,
    Splash,
    Toolbar,
    Tooltip,
    Utility,
}


#[op2(async)]
#[serde]
pub async fn application_x11_pending_event(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsX11ApplicationEvent> {
    let receiver = {
        let state = state.borrow();

        let context = state
            .borrow::<ApplicationContext>();

        match &context.desktop {
            DesktopEnvironment::Linux(linux::LinuxDesktopEnvironment::X11(X11DesktopEnvironment { receiver })) => receiver.clone(),
            _ => Err(anyhow!("Calling application_x11_pending_event on non-x11 platform"))?
        }
    };

    let mut receiver = receiver.borrow_mut();
    let event = receiver.recv()
        .await
        .ok_or_else(|| anyhow!("plugin event stream was suddenly closed"))?;

    tracing::trace!("Received application event {:?}", event);

    Ok(event)
}


#[op2(fast)]
pub fn linux_x11_focus_window(#[string] x11_window_id: String) -> anyhow::Result<()> {

    focus_window(x11_window_id)?;

    Ok(())
}

fn focus_window(window_id: String) -> anyhow::Result<()> {
    // https://github.com/freedesktop-unofficial-mirror/xcb__util-wm/blob/24eb17df2e1245885e72c9d4bbb0a0f69f0700f2/ewmh/xcb_ewmh.h.m4#L1268
    // this basically reimplementation of xcb_ewmh_request_change_active_window

    let window_to_activate = Window::from_str(&window_id)?;

    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let atoms = atoms::Atoms::new(&conn)?.reply()?;

    // TODO change desktop?
    // https://github.com/davatorium/rofi/blob/f9491690fdfbffc5fe13438b26323c05c67acd7b/source/modes/window.c#L820

    let active_window_property = conn.get_property(false, screen.root, atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, 2048)?
        .reply()?;

    let current_active_window = active_window_property
        .value32()
        .and_then(|mut iter| iter.next())
        .unwrap_or(x11rb::NONE); // no focused window

    // these values are same as rofi
    let source_indication: u32 = 2; // XCB_EWMH_CLIENT_SOURCE_TYPE_OTHER
    let timestamp: u32 = x11rb::CURRENT_TIME;

    let event = ClientMessageEvent::new(
        32,
        window_to_activate,
        atoms._NET_ACTIVE_WINDOW,
        [
            source_indication,
            timestamp,
            current_active_window,
            0,
            0
        ],
    );

    conn.send_event(
        false,
        screen.root,
        EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
        event
    )?;

    conn.flush()?;

    Ok(())
}

fn send_event(tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>, app_event: JsX11ApplicationEvent) {
    let sender = sender.clone();
    tokio_handle.spawn(async move {
        if let Err(e) = sender.send(app_event).await {
            tracing::error!("Error while sending x11 application event: {:?}", e);
        }
    });
}

pub fn listen_on_x11_events(
    tokio_handle: Handle,
    sender: Sender<JsX11ApplicationEvent>,
) -> anyhow::Result<Infallible> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let atoms = atoms::Atoms::new(&conn)?.reply()?;

    let aux = ChangeWindowAttributesAux::new()
        .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::PROPERTY_CHANGE);

    conn.change_window_attributes(screen.root, &aux)?.check()?;

    let mut init_window_data = Vec::<Window>::new();

    let _ = fetch_existing_windows(screen.root, &conn, &tokio_handle, &sender, atoms, &mut init_window_data);

    for window_id in init_window_data {
        update_properties(
            window_id,
            &conn,
            &tokio_handle,
            &sender,
            atoms,
        );
    }

    loop {
        match conn.wait_for_event()? {
            Event::CreateNotify(event) => {
                tracing::trace!("CreateNotify: {:?}", event);

                let aux = ChangeWindowAttributesAux::new()
                    .event_mask(EventMask::PROPERTY_CHANGE);

                conn.change_window_attributes(event.window, &aux)?;
                conn.flush()?;

                send_event(&tokio_handle, &sender, JsX11ApplicationEvent::CreateNotify {
                    id: format!("{}", event.window),
                    parent_id: format!("{}", event.parent),
                    override_redirect: event.override_redirect
                });

                update_properties(event.window, &conn, &tokio_handle, &sender, atoms);
            }
            Event::DestroyNotify(event) => {
                tracing::trace!("DestroyNotify: {:?}", event);

                send_event(&tokio_handle, &sender, JsX11ApplicationEvent::DestroyNotify {
                    id: format!("{}", event.window),
                })
            }
            Event::MapNotify(event) => {
                tracing::trace!("MapNotify: {:?}", event);

                send_event(&tokio_handle, &sender, JsX11ApplicationEvent::MapNotify {
                    id: format!("{}", event.window),
                });
            }
            Event::UnmapNotify(event) => {
                tracing::trace!("UnmapNotify: {:?}", event);

                send_event(&tokio_handle, &sender, JsX11ApplicationEvent::UnmapNotify {
                    id: format!("{}", event.window),
                });
            }
            Event::ReparentNotify(event) => {
                tracing::trace!("ReparentNotify: {:?}", event);

                send_event(&tokio_handle, &sender, JsX11ApplicationEvent::ReparentNotify {
                    id: format!("{}", event.window),
                });
            }
            Event::PropertyNotify(event) => {
                tracing::trace!("PropertyNotify: {:?}", event);

                match event.atom {
                    atom if atom == atoms._NET_WM_NAME || atom == Atom::from(AtomEnum::WM_NAME) => {
                        let _ = update_title(event.window, &conn, &tokio_handle, &sender, atoms);
                    }
                    atom if atom == Atom::from(AtomEnum::WM_CLASS) => {
                        let _ = update_class(event.window, &conn, &tokio_handle, &sender);
                    },
                    atom if atom == atoms.WM_PROTOCOLS => {
                        let _ = update_protocols(event.window, &conn, &tokio_handle, &sender, atoms);
                    },
                    atom if atom == atoms.WM_HINTS => {
                        let _ = update_hints(event.window, &conn, &tokio_handle, &sender);
                    },
                    atom if atom == Atom::from(AtomEnum::WM_TRANSIENT_FOR) => {
                        let _ = update_transient_for(event.window, &conn, &tokio_handle, &sender);
                    },
                    atom if atom == atoms._NET_WM_WINDOW_TYPE => {
                        let _ = update_net_window_type(event.window, &conn, &tokio_handle, &sender, atoms);
                    },
                    atom if atom == atoms._KDE_NET_WM_DESKTOP_FILE || atom == atoms._GTK_APPLICATION_ID => {
                        let _ = update_desktop_file_name(event.window, &conn, &tokio_handle, &sender, atoms);
                    },
                    _ => {},
                }
            }
            _ => {}
        }
    }
}

fn fetch_existing_windows(
    window_id: Window,
    conn: &RustConnection,
    tokio_handle: &Handle,
    sender: &Sender<JsX11ApplicationEvent>,
    atoms: atoms::Atoms,
    init_window_data: &mut Vec<Window>
) -> anyhow::Result<()> {
    let query_tree = conn.query_tree(window_id)?.reply()?;

    let attributes = conn.get_window_attributes(window_id)?.reply()?;

    init_window_data.push(window_id);

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::Init {
        id: format!("{}", window_id),
        parent_id: format!("{}", query_tree.parent),
        override_redirect: attributes.override_redirect,
        mapped: match attributes.map_state {
            MapState::UNMAPPED => false,
            MapState::UNVIEWABLE => true,
            MapState::VIEWABLE => true,
            unknown @ _ => Err(anyhow::anyhow!("Unknown map state: {:?}", unknown))?
        },
    });

    for window in query_tree.children {
        let _ = fetch_existing_windows(window, conn, tokio_handle, sender, atoms, init_window_data);
    }

    Ok(())
}


fn update_properties(
    window_id: Window,
    conn: &RustConnection,
    tokio_handle: &Handle,
    sender: &Sender<JsX11ApplicationEvent>,
    atoms: atoms::Atoms
) {
    let _ = update_title(window_id, conn, tokio_handle, sender, atoms);
    let _ = update_class(window_id, conn, tokio_handle, sender);
    let _ = update_hints(window_id, conn, tokio_handle, sender);
    let _ = update_protocols(window_id, conn, tokio_handle, sender, atoms);
    let _ = update_transient_for(window_id, conn, tokio_handle, sender);
    let _ = update_net_window_type(window_id, conn, tokio_handle, sender, atoms);
    let _ = update_desktop_file_name(window_id, &conn, &tokio_handle, &sender, atoms);
}

fn update_title(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>, atoms: atoms::Atoms) -> anyhow::Result<()> {
    let net_wm_name = read_window_property_string(window_id, conn, atoms, atoms._NET_WM_NAME)?;
    let wm_name = read_window_property_string(window_id, conn, atoms, AtomEnum::WM_NAME)?;

    // tracing::trace!("title - _NET_WM_NAME: {:?}", net_wm_name);
    // tracing::trace!("title - WM_NAME: {:?}", wm_name);

    let title = net_wm_name.or(wm_name).unwrap_or_default();

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::TitlePropertyNotify {
        id: format!("{}", window_id),
        title
    });

    Ok(())
}

fn update_class(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>) -> anyhow::Result<()> {
    let (class, instance) = match WmClass::get(conn, window_id)?.reply() {
        Ok(Some(wm_class)) => {
            let class = encoding::all::ISO_8859_1
                .decode(wm_class.class(), DecoderTrap::Replace)
                .ok()
                .unwrap_or_default();

            let instance = encoding::all::ISO_8859_1
                .decode(wm_class.instance(), DecoderTrap::Replace)
                .ok()
                .unwrap_or_default();

            (class, instance)
        },
        Ok(None) => (Default::default(), Default::default()),
        Err(err) => Err(err)?,
    };

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::ClassPropertyNotify {
        id: format!("{}", window_id),
        class,
        instance
    });

    Ok(())
}

fn update_hints(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>) -> anyhow::Result<()> {
    let hints = match WmHints::get(conn, window_id)?.reply() {
        Ok(hints) => hints,
        Err(err) => Err(err)?,
    };

    let window_group = hints
        .and_then(|hints| hints.window_group)
        .map(|window| format!("{}", window));

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::HintsPropertyNotify {
        id: format!("{}", window_id),
        window_group
    });
    
    Ok(())
}

fn update_protocols(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>, atoms: atoms::Atoms) -> anyhow::Result<()> {
    let reply = conn
        .get_property(false, window_id, atoms.WM_PROTOCOLS, AtomEnum::ATOM, 0, 2048)?
        .reply()?;

    let protocols = reply.value32()
        .map(|vals| vals.collect::<Vec<_>>());

    let Some(protocols) = protocols else {
        return Ok(())
    };

    let protocols = protocols
        .into_iter()
        .filter_map(|atom| match atom {
            x if x == atoms.WM_TAKE_FOCUS => Some(JSX11WindowProtocol::TakeFocus),
            x if x == atoms.WM_DELETE_WINDOW => Some(JSX11WindowProtocol::DeleteWindow),
            _ => None,
        })
        .collect::<Vec<_>>();

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::ProtocolsPropertyNotify {
        id: format!("{}", window_id),
        protocols
    });

    Ok(())
}

fn update_transient_for(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>) -> anyhow::Result<()> {
    let reply = conn.get_property(false, window_id, AtomEnum::WM_TRANSIENT_FOR, AtomEnum::WINDOW, 0, 2048)?
        .reply()?;

    let transient_for = reply
        .value32()
        .and_then(|mut iter| iter.next())
        .filter(|w| *w != 0);

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::TransientForPropertyNotify {
        id: format!("{}", window_id),
        transient_for: transient_for.map(|window_id| format!("{}", window_id))
    });

    Ok(())
}

fn update_net_window_type(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>, atoms: atoms::Atoms) -> anyhow::Result<()> {
    let reply = conn
        .get_property(false, window_id, atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM, 0, 1024)?
        .reply()?;

    let window_types = reply.value32()
        .map(|iter| iter.collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|atom| {
            match atom {
                atom if atom == atoms._NET_WM_WINDOW_TYPE_DROPDOWN_MENU => Some(JSX11WindowType::DropdownMenu),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_DIALOG => Some(JSX11WindowType::Dialog),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_MENU => Some(JSX11WindowType::Menu),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_NOTIFICATION => Some(JSX11WindowType::Notification),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_NORMAL => Some(JSX11WindowType::Normal),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_POPUP_MENU => Some(JSX11WindowType::PopupMenu),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_SPLASH => Some(JSX11WindowType::Splash),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_TOOLBAR => Some(JSX11WindowType::Toolbar),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_TOOLTIP => Some(JSX11WindowType::Tooltip),
                atom if atom == atoms._NET_WM_WINDOW_TYPE_UTILITY => Some(JSX11WindowType::Utility),
                _ => None
            }
        })
        .collect();

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::WindowTypePropertyNotify {
        id: format!("{}", window_id),
        window_types
    });

    Ok(())
}

fn update_desktop_file_name(window_id: Window, conn: &RustConnection, tokio_handle: &Handle, sender: &Sender<JsX11ApplicationEvent>, atoms: atoms::Atoms) -> anyhow::Result<()> {
    let kde_net_wm_desktop_file = read_window_property_string(window_id, conn, atoms, atoms._KDE_NET_WM_DESKTOP_FILE)?;
    let gtk_application_id = read_window_property_string(window_id, conn, atoms, atoms._GTK_APPLICATION_ID)?;

    let desktop_file_name = kde_net_wm_desktop_file.or(gtk_application_id).unwrap_or_default();

    send_event(&tokio_handle, &sender, JsX11ApplicationEvent::DesktopFileNamePropertyNotify {
        id: format!("{}", window_id),
        desktop_file_name
    });

    Ok(())
}

fn read_window_property_string(window_id: Window, conn: &RustConnection, atoms: atoms::Atoms, atom: impl Into<Atom>) -> anyhow::Result<Option<String>> {
    let reply = conn
        .get_property(false, window_id, atom, AtomEnum::ANY, 0, 2048)?
        .reply()?;

    let Some(bytes) = reply.value8() else {
        return Ok(None)
    };

    let bytes = bytes.collect::<Vec<u8>>();

    match reply.type_ {
        x if x == Atom::from(AtomEnum::STRING) => {
            let decoded = encoding::all::ISO_8859_1
                .decode(&bytes, DecoderTrap::Replace)
                .ok();

            Ok(decoded)
        },
        x if x == atoms.UTF8_STRING => {
            Ok(String::from_utf8(bytes).ok())
        },
        _ => Ok(None),
    }
}
mod atoms {
    x11rb::atom_manager! {
        pub Atoms:
        AtomsCookie {
            // data formats
            UTF8_STRING,

            // client -> server
            WM_HINTS,
            WM_PROTOCOLS,
            WM_TAKE_FOCUS,
            WM_DELETE_WINDOW,
            _NET_WM_NAME,
            _NET_WM_PID,
            _NET_WM_WINDOW_TYPE,
            _NET_WM_WINDOW_TYPE_DROPDOWN_MENU,
            _NET_WM_WINDOW_TYPE_DIALOG,
            _NET_WM_WINDOW_TYPE_MENU,
            _NET_WM_WINDOW_TYPE_NOTIFICATION,
            _NET_WM_WINDOW_TYPE_NORMAL,
            _NET_WM_WINDOW_TYPE_POPUP_MENU,
            _NET_WM_WINDOW_TYPE_SPLASH,
            _NET_WM_WINDOW_TYPE_TOOLBAR,
            _NET_WM_WINDOW_TYPE_TOOLTIP,
            _NET_WM_WINDOW_TYPE_UTILITY,
            _NET_WM_STATE_MODAL,
            _NET_ACTIVE_WINDOW,

            // non-standard
            _KDE_NET_WM_DESKTOP_FILE,
            _GTK_APPLICATION_ID,
        }
    }
}


