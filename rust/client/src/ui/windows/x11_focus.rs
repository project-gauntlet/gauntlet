use std::convert::Infallible;

use anyhow::anyhow;
use iced::Subscription;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::stream;
use tokio::runtime::Handle;
use x11rb::connection::Connection;
use x11rb::properties::WmClass;
use x11rb::protocol::xproto::AtomEnum;
use x11rb::protocol::xproto::ChangeWindowAttributesAux;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::EventMask;
use x11rb::protocol::xproto::Window;
use x11rb::rust_connection::RustConnection;

use crate::ui::AppMsg;
use crate::ui::windows::WindowActionMsg;

pub fn x11_linux_focus_change_subscription() -> Subscription<AppMsg> {
    Subscription::run(|| {
        stream::channel(100, async move |sender| {
            let handle = Handle::current();

            let err = tokio::task::spawn_blocking(|| listen_on_x11_active_window_change(sender, handle)).await;

            if let Err(err) = err {
                tracing::error!("error occurred when listening on x11 events: {:?}", err);
            }
        })
    })
}

fn listen_on_x11_active_window_change(sender: Sender<AppMsg>, handle: Handle) -> anyhow::Result<Infallible> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let atoms = atoms::Atoms::new(&conn)?.reply()?;

    let aux = ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE);

    conn.change_window_attributes(screen.root, &aux)?.check()?;

    loop {
        if let x11rb::protocol::Event::PropertyNotify(event) = conn.wait_for_event()? {
            if event.atom == atoms._NET_ACTIVE_WINDOW {
                let Ok(window) = fetch_window_id(&conn, screen.root, &atoms) else {
                    continue;
                };

                let wm_name = fetch_app_wm_name(&conn, window).ok();

                let mut sender = sender.clone();
                handle.spawn(async move {
                    sender
                        .send(AppMsg::WindowAction(WindowActionMsg::X11ActiveWindowChanged {
                            window,
                            wm_name,
                        }))
                        .await
                });
            }
        }
    }
}

fn fetch_window_id(conn: &impl Connection, root: Window, atoms: &atoms::Atoms) -> anyhow::Result<Window> {
    let window = conn
        .get_property(false, root, atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, 1)?
        .reply()?
        .value32()
        .ok_or(anyhow!("_NET_ACTIVE_WINDOW has incorrect format"))?
        .next()
        .ok_or(anyhow!("_NET_ACTIVE_WINDOW is empty"))?;

    Ok(window)
}

fn fetch_app_wm_name(conn: &impl Connection, window_id: Window) -> anyhow::Result<String> {
    let wm_class = WmClass::get(conn, window_id)?;
    let wm_class = wm_class.reply()?.ok_or(anyhow!("no WM_CLASS prop on the window"))?;
    let class = std::str::from_utf8(wm_class.class())?;

    Ok(class.to_string())
}

mod atoms {
    x11rb::atom_manager! {
        pub Atoms:
        AtomsCookie {
            _NET_ACTIVE_WINDOW,
            _NET_WM_NAME,
            UTF8_STRING,
        }
    }
}
