use std::thread;

use relm4::RelmApp;
use tokio::task::LocalSet;

use crate::gtk::gtk_side::start_request_receiver_loop;
use crate::gtk::gui::AppModel;
use crate::plugins::PluginManager;
use crate::react_side::{PluginReactContext, run_react};

mod react_side;
mod plugins;
mod gtk;


fn main() {
    let mut plugin_manager = PluginManager::create();

    let (react_contexts, ui_contexts) = plugin_manager.create_all_contexts();

    // TODO what is proper letter case here?
    let app = RelmApp::new("org.placeholdername.placeholdername");

    spawn_react_thread(react_contexts);

    start_request_receiver_loop(ui_contexts.clone());

    app.run::<AppModel>(plugin_manager.clone());
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

