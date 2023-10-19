use crate::server::dbus::{DbusManagementServer, DbusServer};
use crate::server::plugins::PluginManager;
use crate::server::search::SearchIndex;

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
    let search_index = SearchIndex::create_index().unwrap();
    let mut plugin_manager = PluginManager::create(search_index.clone());

    plugin_manager.reload_all_plugins();

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
