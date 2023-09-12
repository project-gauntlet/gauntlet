use crate::server::dbus::DbusServer;
use crate::server::plugins::PluginManager;
use crate::server::search::{SearchIndex, SearchItem};

pub mod dbus;
pub(in crate::server) mod search;
pub(in crate::server) mod plugins;
pub(in crate::server) mod model;

pub async fn start_server() -> anyhow::Result<()> {
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
