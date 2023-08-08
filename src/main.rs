use std::thread;

use relm4::RelmApp;
use tokio::task::LocalSet;

use crate::gtk::gtk_side::start_request_receiver_loop;
use crate::gtk::gui::{AppInput, AppModel};
use crate::plugins::PluginManager;
use crate::react_side::{PluginReactContext, run_react};
use crate::search::{SearchItem, SearchIndex};

mod react_side;
mod plugins;
mod gtk;
mod search;


fn main() {
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

    search_index.add_entries(search_items.clone()).unwrap();

    let (react_contexts, ui_contexts) = plugin_manager.create_all_contexts();

    // TODO what is proper letter case here?
    let app = RelmApp::new("org.placeholdername.placeholdername");

    spawn_react_thread(react_contexts);

    start_request_receiver_loop(ui_contexts.clone());

    app.run::<AppModel>(AppInput {
        search: search_index.create_handle(),
        plugin_manager: plugin_manager.clone(),
        search_items
    });
}

fn spawn_react_thread(react_contexts: Vec<PluginReactContext>) {
    let handle = move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let local_set = LocalSet::new();

        local_set.block_on(&runtime, async {
            let mut join_set = tokio::task::JoinSet::new();
            for react_context in react_contexts {
                join_set.spawn_local(async {
                    run_react(react_context).await
                });
            }
            while let Some(_) = join_set.join_next().await {
            }
        })
    };

    thread::Builder::new()
        .name("react-thread".into())
        .spawn(handle)
        .expect("failed to spawn thread");
}

