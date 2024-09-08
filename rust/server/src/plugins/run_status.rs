use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};

use common::model::PluginId;

pub struct RunStatusHolder {
    running_plugins: Arc<Mutex<HashMap<PluginId, CancellationToken>>>
}

impl RunStatusHolder {
    pub fn new() -> Self {
        Self {
            running_plugins: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn start_block(&self, plugin_id: PluginId) -> RunStatusGuard {
        let mut running_plugins = self.running_plugins.lock().expect("lock is poisoned");
        running_plugins.insert(plugin_id.clone(), CancellationToken::new());
        RunStatusGuard {
            running_plugins: self.running_plugins.clone(),
            id: plugin_id,
        }
    }

    pub fn is_plugin_running(&self, plugin_id: &PluginId) -> bool {
        let running_plugins = self.running_plugins.lock().expect("lock is poisoned");
        running_plugins.contains_key(plugin_id)
    }

    pub fn stop_plugin(&self, plugin_id: &PluginId) {
        let mut running_plugins = self.running_plugins.lock().expect("lock is poisoned");

        running_plugins
            .remove(plugin_id)
            .expect("value should always exist for specified id")
            .cancel()
    }
}

pub struct RunStatusGuard {
    id: PluginId,
    running_plugins: Arc<Mutex<HashMap<PluginId, CancellationToken>>>,
}

impl RunStatusGuard {
    pub fn stopped(&self) -> WaitForCancellationFutureOwned {
        let mut running_plugins = self.running_plugins.lock().expect("lock is poisoned");

        running_plugins
            .get(&self.id)
            .expect("value should always exist for specified id")
            .clone()
            .cancelled_owned()
    }
}
