use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{anyhow, Context};

use crate::server::plugins::data_db_repository::{Code, DataDbRepository, SavePluginEntrypoint, SavePlugin};
use crate::server::plugins::PackageJson;

pub struct PluginDownloader {
    db_repository: DataDbRepository,
}

impl PluginDownloader {
    pub fn new(db_repository: DataDbRepository) -> Self {
        Self {
            db_repository
        }
    }

    pub async fn download_plugin(&self, plugin_id: String) -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;

        let url = gix::url::parse(gix::path::os_str_into_bstr(plugin_id.as_ref())?)?;
        let mut prepare_fetch = gix::clone::PrepareFetch::new(url, &temp_dir, gix::create::Kind::WithWorktree, Default::default(), Default::default())?
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

        let git_repo_dir = temp_dir.path();

        let plugins_path = git_repo_dir.join("plugins");

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

        self.save_plugin(version_path, plugin_id).await?;

        Ok(())
    }

    async fn save_plugin(&self, plugin_dir: PathBuf, plugin_id: String) -> anyhow::Result<()> {
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

        self.db_repository.save_plugin(SavePlugin {
            id: plugin_id,
            name: plugin_name,
            code: Code {
                js
            },
            entrypoints,
        }).await?;

        Ok(())
    }
}