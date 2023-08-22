use std::cell::Cell;
use std::collections::HashMap;
use std::process::exit;
use std::thread;

use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, WidgetExt};
use relm4::{Component, ComponentController};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::LocalSet;

use crate::gtk::{PluginContainerContainer, PluginEventSenderContainer, PluginUiContext, PluginUiData};
use crate::gtk::gtk_side::{start_request_receiver_loop, start_server_event_receiver_loop};
use crate::gtk::gui::{AppInput, AppModel};
use crate::plugins::PluginManager;
use crate::react_side::{PluginReactData, run_react};
use crate::search::{SearchIndex, SearchItem};

pub enum ServerEvent {
    OpenWindow
}

struct DbusInterface {
    server_event_sender: UnboundedSender<ServerEvent>,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName")]
impl DbusInterface {
    fn open_window(&mut self) {
        self.server_event_sender.send(ServerEvent::OpenWindow).unwrap();
    }
}

pub fn run_server(dev: bool) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (server_event_sender, server_event_receiver) = tokio::sync::mpsc::unbounded_channel::<ServerEvent>();

    let mut plugin_manager = PluginManager::create();
    let mut search_index = SearchIndex::create_index().unwrap();

    let search_items: Vec<_> = plugin_manager.plugins()
        .iter()
        .flat_map(|plugin| {
            plugin.entrypoints()
                .iter()
                .map(|entrypoint| {
                    SearchItem {
                        entrypoint_name: entrypoint.name().to_owned(),
                        entrypoint_id: entrypoint.id().to_owned(),
                        plugin_name: plugin.name().to_owned(),
                        plugin_id: plugin.id().to_owned(),
                    }
                })
        })
        .collect();

    search_index.add_entries(search_items).unwrap();

    let (react_contexts, ui_contexts) = plugin_manager.create_all_contexts();

    let zbus_connection: Result<zbus::Connection, zbus::Error> = runtime.block_on(async {
        let interface = DbusInterface { server_event_sender };

        let conn = zbus::ConnectionBuilder::session()?
            .name("org.placeholdername.PlaceHolderName")?
            .serve_at("/org/placeholdername/PlaceHolderName", interface)?
            .build()
            .await?;

        Ok(conn)
    });

    let _zbus_connection = zbus_connection.unwrap_or_else(|error| {
        match error {
            zbus::Error::NameTaken => eprintln!("Another server already running"),
            _ => eprintln!("Unexpected error occurred when setting up dbus connection: {error}"),
        }
        exit(1)
    });

    spawn_gtk_thread(dev, ui_contexts, plugin_manager, search_index, server_event_receiver);

    run_react_loops(&runtime, react_contexts);
}

fn spawn_gtk_thread(
    dev: bool,
    ui_data: Vec<PluginUiData>,
    plugin_manager: PluginManager,
    search_index: SearchIndex,
    server_event_receiver: UnboundedReceiver<ServerEvent>,
) {
    let handle = move || {
        let (contexts, event_senders): (Vec<_>, Vec<_>) = ui_data.into_iter()
            .map(|ui_data| {
                let context = (ui_data.plugin.clone(), ui_data.request_receiver);
                let event_sender = (ui_data.plugin.id().to_owned(), (ui_data.event_sender, ui_data.event_waker));
                (context, event_sender)
            })
            .unzip();

        let ui_contexts = contexts.into_iter()
            .map(|(plugin, receiver)| PluginUiContext::new(plugin, receiver))
            .collect::<Vec<_>>();

        let event_senders = event_senders.into_iter()
            .collect::<HashMap<_, _>>();

        let container_container = PluginContainerContainer::new();
        let event_senders_container = PluginEventSenderContainer::new(event_senders);

        start_request_receiver_loop(
            ui_contexts,
            container_container.clone(),
            event_senders_container.clone(),
        );

        let input = AppInput {
            search: search_index.create_handle(),
            plugin_manager,
            container_container,
            event_senders_container,
        };

        let application = gtk::Application::builder()
            .build();

        let _ = application.hold();

        let payload = Cell::new(Some((input, server_event_receiver)));

        application.connect_activate(move |application| {
            if let Some((input, server_event_receiver)) = payload.take() {
                let mut controller = AppModel::builder()
                    .launch(input)
                    .detach();

                let window = controller.widget()
                    .clone()
                    .upcast::<gtk::Window>();

                controller.detach_runtime();

                start_server_event_receiver_loop(
                    window.clone(),
                    server_event_receiver,
                );

                application.add_window(&window);
                if dev {
                    window.set_visible(true);
                }
            }
        });

        application.run();
    };

    thread::Builder::new()
        .name("gtk-thread".into())
        .spawn(handle)
        .expect("failed to spawn thread");
}


fn run_react_loops(runtime: &Runtime, react_contexts: Vec<PluginReactData>) {
    let local_set = LocalSet::new();

    local_set.block_on(runtime, async {
        let mut join_set = tokio::task::JoinSet::new();
        for react_context in react_contexts {
            join_set.spawn_local(tokio::task::unconstrained(async {
                run_react(react_context).await
            }));
        }
        while let Some(_) = join_set.join_next().await {}
    });
}

