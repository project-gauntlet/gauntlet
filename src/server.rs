use std::collections::HashMap;

use deno_core::anyhow;
use gtk::glib::MainContext;
use gtk::prelude::WidgetExt;
use relm4::{Component, RelmApp};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use zbus::zvariant::Type;

use crate::channel::channel;
use crate::gtk::{PluginContainerContainer, PluginEventSenderContainer};
use crate::gtk::gtk_side::{ClientContext, DbusClient, GtkContext};
use crate::gtk::gui::{AppInput, AppModel};
use crate::plugins::PluginManager;
use crate::react_side::{UiEvent, UiEventViewCreated, UiEventViewEvent, UiRequestData, UiResponseData};
use crate::search::{SearchClient, SearchIndex, SearchItem, UiSearchRequest, UiSearchResult};

struct DbusServer {
    plugins: Vec<String>,
    search_index: SearchIndex,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName")]
impl DbusServer {
    fn plugins(&mut self) -> Vec<String> {
        self.plugins.clone()
    }

    fn search(&self, text: &str) -> Vec<DBusSearchResult> {
        self.search_index.create_handle()
            .search(text)
            .unwrap()
            .into_iter()
            .map(|item| {
                DBusSearchResult {
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_id: item.entrypoint_id,
                    plugin_name: item.plugin_name,
                    plugin_uuid: item.plugin_id,
                }
            })
            .collect()
    }
}

#[zbus::dbus_proxy(
    default_service = "org.placeholdername.PlaceHolderName",
    default_path = "/org/placeholdername/PlaceHolderName",
    interface = "org.placeholdername.PlaceHolderName",
)]
trait DbusServerProxy {
    async fn plugins(&self) -> zbus::Result<Vec<String>>;
    async fn search(&self, text: &str) -> zbus::Result<Vec<DBusSearchResult>>;
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[zvariant(signature = "(ssss)")]
pub struct DBusSearchResult {
    pub plugin_uuid: String,
    pub plugin_name: String,
    pub entrypoint_id: String,
    pub entrypoint_name: String,
}

pub async fn run_server() -> anyhow::Result<()> {
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

    let plugin_uuids: Vec<_> = plugin_manager.plugins()
        .iter()
        .map(|plugin| plugin.id().to_owned())
        .collect();

    search_index.add_entries(search_items).unwrap();

    plugin_manager.start_all_contexts();

    let interface = DbusServer { plugins: plugin_uuids, search_index };

    let _conn = zbus::ConnectionBuilder::session()?
        .name("org.placeholdername.PlaceHolderName")?
        .serve_at("/org/placeholdername/PlaceHolderName", interface)?
        .build()
        .await?;

    std::future::pending::<()>().await;

    Ok(())
}

pub fn run_client(runtime: &Runtime) -> anyhow::Result<()> {

    let (request_tx, mut request_rx) = channel::<(String, UiRequestData), UiResponseData>();
    let (event_tx, mut event_rx) = channel::<(String, UiEvent), ()>();
    let (search_tx, mut search_rx) = channel::<UiSearchRequest, Vec<UiSearchResult>>();

    let dbus_client = DbusClient {
        channel: request_tx
    };

    let path = "/org/placeholdername/PlaceHolderName";

    let (connection, server_proxy) = runtime.block_on(async {
        let connection = zbus::ConnectionBuilder::session()?
            .name("org.placeholdername.PlaceHolderName.Client")?
            .serve_at(path, dbus_client)?
            .build()
            .await?;
        let server_proxy = DbusServerProxyProxy::new(&connection).await?;

        Ok::<_, anyhow::Error>((connection, server_proxy))
    })?;

    let connection_clone = connection.clone();

    runtime.spawn(tokio::task::unconstrained(async move {
        println!("starting event handler loop");

        let interface_ref = connection_clone
            .object_server()
            .interface::<_, DbusClient>(path)
            .await
            .unwrap();

        let signal_context = interface_ref
            .signal_context();

        while let Ok(((plugin_uuid, event_data), _)) = event_rx.recv().await {
            match event_data {
                UiEvent::ViewCreated { view_name } => {
                    DbusClient::view_created_signal(&signal_context, &plugin_uuid, UiEventViewCreated { view_name })
                        .await
                        .unwrap();
                }
                UiEvent::ViewDestroyed => {}
                UiEvent::ViewEvent { event_name, widget_id } => {
                    DbusClient::view_event_signal(&signal_context, &plugin_uuid, UiEventViewEvent { event_name, widget_id })
                        .await
                        .unwrap();
                }
            }
        }
    }));

    let event_senders_container = PluginEventSenderContainer::new(event_tx);
    let container = event_senders_container.clone();

    let container_container = PluginContainerContainer::new();
    let containers = container_container.clone();

    let server_proxy_clone = server_proxy.clone();
    MainContext::default().spawn_local(async move {
        println!("starting request handler loop");

        let plugin_uuids = server_proxy_clone.plugins().await.unwrap();

        let contexts: HashMap<_, _> = plugin_uuids.iter()
            .map(|plugin_uuid| (plugin_uuid.to_owned(), GtkContext::new()))
            .collect();

        let mut client_context = ClientContext {
            contexts,
            containers,
        };

        while let Ok(((plugin_uuid, request_data), responder)) = request_rx.recv().await {
            match request_data {
                UiRequestData::GetContainer => {
                    let response = client_context.get_container(&plugin_uuid);
                    responder.respond(response).unwrap()
                }
                UiRequestData::CreateInstance { widget_type } => {
                    let response = client_context.create_instance(&plugin_uuid, &widget_type);
                    responder.respond(response).unwrap()
                }
                UiRequestData::CreateTextInstance { text } => {
                    let response = client_context.create_text_instance(&plugin_uuid, &text);
                    responder.respond(response).unwrap()
                }
                UiRequestData::AppendChild { parent, child } => {
                    client_context.append_child(&plugin_uuid, parent, child);
                }
                UiRequestData::RemoveChild { parent, child } => {
                    client_context.remove_child(&plugin_uuid, parent, child);
                }
                UiRequestData::InsertBefore { parent, child, before_child } => {
                    client_context.insert_before(&plugin_uuid, parent, child, before_child);
                }
                UiRequestData::SetProperties { widget, properties } => {
                    client_context.set_properties(container.clone(), &plugin_uuid, widget, properties).await;
                }
                UiRequestData::SetText { widget, text } => {
                    client_context.set_text(&plugin_uuid, widget, &text);
                }
            }
        }
    });

    let server_proxy_clone = server_proxy.clone();
    runtime.spawn(tokio::task::unconstrained(async move {
        println!("starting search handler loop");
        while let Ok((search_request, responder)) = search_rx.recv().await {
            let result: Vec<_> = server_proxy_clone.search(&search_request.prompt)
                .await
                .unwrap()
                .into_iter()
                .map(|item| {
                    UiSearchResult {
                        plugin_uuid: item.plugin_uuid,
                        plugin_name: item.plugin_name,
                        entrypoint_id: item.entrypoint_id,
                        entrypoint_name: item.entrypoint_name,
                    }
                })
                .collect();

            responder.respond(result).unwrap();
        }
    }));

    let search_client = SearchClient::new(search_tx);

    let input = AppInput {
        search_client,
        container_container,
        event_senders_container,
    };

    let app = RelmApp::from_app(gtk::Application::builder().build());
    app.run::<AppModel>(input);

    Ok(())
}

