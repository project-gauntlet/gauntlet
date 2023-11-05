use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;

use anyhow::{anyhow, Context};

use crate::common::model::PluginId;
use crate::server::dbus::DbusManagementServer;
use crate::server::plugins::data_db_repository::{Code, DataDbRepository, SavePlugin, SavePluginEntrypoint};
use crate::server::plugins::PackageJson;

pub struct PluginLoader {
    db_repository: DataDbRepository,
}

impl PluginLoader {
    pub fn new(db_repository: DataDbRepository) -> Self {
        Self {
            db_repository
        }
    }

    pub async fn add_remote_plugin(
        &self,
        signal_context: zbus::SignalContext<'_>,
        plugin_id: PluginId
    ) -> anyhow::Result<()> {
        let data_db_repository = self.db_repository.clone();
        let signal_context = signal_context.to_owned();
        thread::spawn(move || {
            let temp_dir = tempfile::tempdir()
                .unwrap();

            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async move {
                let plugin_dir = PluginLoader::download(temp_dir.path(), plugin_id.clone())
                    .unwrap();

                let plugin_data = PluginLoader::read_plugin_dir(plugin_dir, plugin_id.clone())
                    .await
                    .unwrap();

                DbusManagementServer::remote_plugin_download_finished_signal(&signal_context, &plugin_id.to_string())
                    .await
                    .unwrap();

                data_db_repository.save_plugin(SavePlugin {
                    id: plugin_data.id,
                    name: plugin_data.name,
                    code: plugin_data.code,
                    entrypoints: plugin_data.entrypoints,
                    from_config: false,
                }).await.unwrap();
            });
        });

        Ok(())
    }

    pub async fn add_local_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        let plugin_dir = plugin_id.try_to_path()?;

        let plugin_data = PluginLoader::read_plugin_dir(plugin_dir, plugin_id.clone())
            .await
            .unwrap();

        self.db_repository.save_plugin(SavePlugin {
            id: plugin_data.id,
            name: plugin_data.name,
            code: plugin_data.code,
            entrypoints: plugin_data.entrypoints,
            from_config: false,
        }).await?;

        Ok(())
    }

    fn download(target_dir: &Path, plugin_id: PluginId) -> anyhow::Result<PathBuf> {
        let url = plugin_id.try_to_git_url()?;

        let mut prepare_fetch = gix::clone::PrepareFetch::new(url, &target_dir, gix::create::Kind::WithWorktree, Default::default(), Default::default())?
            .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(1.try_into().unwrap()))
            .configure_remote(|mut remote| {
                remote.replace_refspecs(
                    Some("+refs/heads/placeholdername/releases:refs/remotes/origin/placeholdername/releases"),
                    gix::remote::Direction::Fetch,
                )?;

                Ok(remote)
            });

        let (mut prepare_checkout, _) = prepare_fetch.fetch_then_checkout(
            gix::progress::Discard,
            &gix::interrupt::IS_INTERRUPTED,
        )?;

        let (_repo, _) = prepare_checkout.main_worktree(
            gix::progress::Discard,
            &gix::interrupt::IS_INTERRUPTED,
        )?;

        let plugins_path = target_dir.join("plugins");

        let mut latest_version = None;

        for entry in std::fs::read_dir(plugins_path.clone())? {
            let entry = entry?;
            let version = entry.file_name()
                .into_string()
                .map_err(|os_str| anyhow!("\"{:?}\" is not a valid utf-8", os_str))?
                .replace("v", "")
                .parse::<u32>()?;

            latest_version = latest_version.max(Some(version));
        }

        let latest_version = latest_version.ok_or_else(|| anyhow!("Repository contains no versions"))?;

        let version_path = plugins_path.join(format!("v{}", latest_version));

        Ok(version_path)
    }

    async fn read_plugin_dir(plugin_dir: PathBuf, plugin_id: PluginId) -> anyhow::Result<PluginDirData> {
        let js_dir = plugin_dir.join("js");

        let js_files = std::fs::read_dir(js_dir)?;

        let js: HashMap<_, _> = js_files.into_iter()
            .map(|dist_path| dist_path.unwrap().path())
            .filter(|dist_path| dist_path.extension() == Some(OsStr::new("js")))
            .map(|dist_path| {
                let js_content = std::fs::read_to_string(&dist_path).unwrap();
                let id = dist_path.file_stem().unwrap().to_str().unwrap().to_owned();

                (id, js_content)
            })
            .collect();

        let package_path = plugin_dir.join("package.json");
        let package_path_context = package_path.display().to_string();
        let package_content = std::fs::read_to_string(package_path).context(package_path_context)?;
        let package_json: PackageJson = serde_json::from_str(&package_content)?;

        let plugin_name = package_json.plugin.metadata.name;

        let entrypoints: Vec<_> = package_json.plugin
            .entrypoints
            .into_iter()
            .map(|entrypoint| SavePluginEntrypoint {
                id: entrypoint.id,
                name: entrypoint.name,
            })
            .collect();

        Ok(PluginDirData {
            id: plugin_id.to_string(),
            name: plugin_name,
            code: Code {
                js
            },
            entrypoints,
        })
    }
}

struct PluginDirData {
    pub id: String,
    pub name: String,
    pub code: Code,
    pub entrypoints: Vec<SavePluginEntrypoint>,
}