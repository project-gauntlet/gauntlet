use crate::dbus::{DbusManagementServer, DbusServer};
use crate::plugins::ApplicationManager;
use crate::search::SearchIndex;

pub mod dbus;
pub(in crate) mod search;
pub(in crate) mod plugins;
pub(in crate) mod model;
mod dirs;

pub fn start_server() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            run_server().await
        })
        .unwrap();
}

async fn run_server() -> anyhow::Result<()> {
    let search_index = SearchIndex::create_index()?;
    let mut application_manager = ApplicationManager::create(search_index.clone()).await?;

    application_manager.reload_all_plugins().await?; // TODO do not return here ?

    let interface = DbusServer { search_index };
    let management_interface = DbusManagementServer { application_manager };

    let _conn = zbus::ConnectionBuilder::session()?
        .name("dev.projectgauntlet.Gauntlet")?
        .serve_at("/dev/projectgauntlet/Server", interface)?
        .serve_at("/dev/projectgauntlet/Server", management_interface)?
        .build()
        .await?;

    std::future::pending::<()>().await;

    Ok(())
}
