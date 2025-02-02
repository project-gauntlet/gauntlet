use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gauntlet_common::model::DownloadStatus;
use gauntlet_common::model::PluginId;

pub struct DownloadStatusHolder {
    running_downloads: Arc<Mutex<HashMap<PluginId, DownloadStatus>>>,
}

impl DownloadStatusHolder {
    pub fn new() -> Self {
        Self {
            running_downloads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn download_started(&self, plugin_id: PluginId) -> DownloadStatusGuard {
        let mut running_downloads = self.running_downloads.lock().expect("lock is poisoned");
        running_downloads.insert(plugin_id.clone(), DownloadStatus::InProgress);
        DownloadStatusGuard {
            running_downloads: self.running_downloads.clone(),
            id: plugin_id,
        }
    }

    pub fn download_status(&self) -> HashMap<PluginId, DownloadStatus> {
        let running_downloads = self.running_downloads.lock().expect("lock is poisoned");
        running_downloads
            .iter()
            .map(|(plugin_id, status)| (plugin_id.clone(), status.clone()))
            .collect()
    }
}

pub struct DownloadStatusGuard {
    id: PluginId,
    running_downloads: Arc<Mutex<HashMap<PluginId, DownloadStatus>>>,
}

impl DownloadStatusGuard {
    pub fn download_finished(&self) {
        let mut running_downloads = self.running_downloads.lock().expect("lock is poisoned");

        running_downloads.insert(self.id.clone(), DownloadStatus::Done);

        self.drop_eventually()
    }

    pub fn download_failed(&self, message: String) {
        let mut running_downloads = self.running_downloads.lock().expect("lock is poisoned");

        running_downloads.insert(self.id.clone(), DownloadStatus::Failed { message });

        self.drop_eventually()
    }

    fn drop_eventually(&self) {
        let running_downloads = self.running_downloads.clone();
        let plugin_id = self.id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let mut running_downloads = running_downloads.lock().expect("lock is poisoned");
            running_downloads.remove(&plugin_id);
        });
    }
}
