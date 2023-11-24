use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use common::model::PluginId;

pub struct RunStatusHolder {
    running_plugins: Arc<Mutex<HashSet<PluginId>>>
}

impl RunStatusHolder {
    pub fn new() -> Self {
        Self {
            running_plugins: Arc::new(Mutex::new(HashSet::new()))
        }
    }

    pub fn start_block(&mut self, plugin_id: PluginId) -> RunStatusGuard {
        let mut running_plugins = self.running_plugins.lock().expect("lock is poisoned");
        running_plugins.insert(plugin_id.clone());
        RunStatusGuard {
            running_plugins: self.running_plugins.clone(),
            id: plugin_id,
        }
    }

    pub fn is_plugin_running(&self, plugin_id: &PluginId) -> bool {
        let running_plugins = self.running_plugins.lock().expect("lock is poisoned");
        running_plugins.contains(plugin_id)
    }
}

pub struct RunStatusGuard {
    id: PluginId,
    running_plugins: Arc<Mutex<HashSet<PluginId>>>,
}

impl Drop for RunStatusGuard {
    fn drop(&mut self) {
        let mut running_plugins = self.running_plugins.lock().expect("lock is poisoned");
        running_plugins.remove(&self.id);
    }
}