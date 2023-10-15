use crate::server::dbus::{DbusManagementServer, DbusServer};
use crate::server::plugins::PluginManager;
use crate::server::search::{SearchIndex, SearchItem};

pub mod dbus;
pub(in crate::server) mod search;
pub(in crate::server) mod plugins;
pub(in crate::server) mod model;

pub fn start_server() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        run_server().await
    }).unwrap();
}

async fn run_server() -> anyhow::Result<()> {
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
                        entrypoint_uuid: entrypoint.id().to_owned(),
                        plugin_name: plugin.name().to_owned(),
                        plugin_id: plugin.id().to_owned(),
                    }
                })
        })
        .collect();

    search_index.add_entries(search_items).unwrap();

    plugin_manager.start_all_contexts();

    let interface = DbusServer { search_index };
    let management_interface = DbusManagementServer { plugin_manager };

    let _conn = zbus::ConnectionBuilder::session()?
        .name("org.placeholdername.PlaceHolderName")?
        .serve_at("/org/placeholdername/PlaceHolderName", interface)?
        .serve_at("/org/placeholdername/PlaceHolderName/Management", management_interface)?
        .build()
        .await?;

    std::future::pending::<()>().await;

    Ok(())
}
