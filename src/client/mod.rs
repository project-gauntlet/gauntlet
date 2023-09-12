use std::collections::HashMap;

use gtk::glib::MainContext;
use relm4::RelmApp;
use tokio::runtime::Runtime;

use crate::client::context::{ClientContext, GtkContext, NativeUiEvent, PluginContainerContainer, PluginEventSenderContainer};
use crate::client::dbus::DbusClient;
use crate::client::dbus::DbusServerProxyProxy;
use crate::client::model::{NativeUiRequestData, NativeUiResponseData, NativeUiSearchRequest, NativeUiSearchResult};
use crate::client::native_ui::{AppInput, AppModel};
use crate::client::search::SearchClient;
use crate::dbus::{DbusEventViewCreated, DbusEventViewEvent};
use crate::utils::channel::channel;

pub(in crate::client) mod dbus;
pub(in crate::client) mod search;
pub(in crate::client) mod context;
pub(in crate::client) mod native_ui;
pub(in crate::client) mod model;

pub fn start_client(runtime: &Runtime) -> anyhow::Result<()> {
    let (request_tx, mut request_rx) = channel::<(String, NativeUiRequestData), NativeUiResponseData>();
    let (event_tx, mut event_rx) = channel::<(String, NativeUiEvent), ()>();
    let (search_tx, mut search_rx) = channel::<NativeUiSearchRequest, Vec<NativeUiSearchResult>>();

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
                NativeUiEvent::ViewCreated { view_name } => {
                    DbusClient::view_created_signal(&signal_context, &plugin_uuid, DbusEventViewCreated { view_name })
                        .await
                        .unwrap();
                }
                NativeUiEvent::ViewDestroyed => {}
                NativeUiEvent::ViewEvent { event_name, widget_id } => {
                    DbusClient::view_event_signal(&signal_context, &plugin_uuid, DbusEventViewEvent { event_name, widget_id })
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
                NativeUiRequestData::GetContainer => {
                    let response = client_context.get_container(&plugin_uuid);
                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::CreateInstance { widget_type } => {
                    let response = client_context.create_instance(&plugin_uuid, &widget_type);
                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::CreateTextInstance { text } => {
                    let response = client_context.create_text_instance(&plugin_uuid, &text);
                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::AppendChild { parent, child } => {
                    client_context.append_child(&plugin_uuid, parent, child);
                }
                NativeUiRequestData::RemoveChild { parent, child } => {
                    client_context.remove_child(&plugin_uuid, parent, child);
                }
                NativeUiRequestData::InsertBefore { parent, child, before_child } => {
                    client_context.insert_before(&plugin_uuid, parent, child, before_child);
                }
                NativeUiRequestData::SetProperties { widget, properties } => {
                    client_context.set_properties(container.clone(), &plugin_uuid, widget, properties).await;
                }
                NativeUiRequestData::SetText { widget, text } => {
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
                    NativeUiSearchResult {
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

