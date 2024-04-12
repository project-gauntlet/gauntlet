use std::path::PathBuf;
use std::sync::Arc;
use anyhow::anyhow;
use gix::Url;
use gix::url::Scheme;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(Arc<str>);

impl PluginId {
    pub fn from_string(plugin_id: impl ToString) -> Self {
        PluginId(plugin_id.to_string().into())
    }

    pub fn try_to_git_url(&self) -> anyhow::Result<Url> {
        let url = gix::url::parse(gix::path::os_str_into_bstr(self.to_string().as_ref())?)?;
        Ok(url)
    }

    pub fn try_to_path(&self) -> anyhow::Result<PathBuf> {
        let url = self.try_to_git_url()?;

        if url.scheme != Scheme::File {
            return Err(anyhow!("plugin id is expected to point to local file"))
        }

        let plugin_dir: String = url.path.try_into()?;
        let plugin_dir = PathBuf::from(plugin_dir);
        Ok(plugin_dir)
    }
}

impl ToString for PluginId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntrypointId(Arc<str>);

impl EntrypointId {
    pub fn from_string(entrypoint_id: impl ToString) -> Self {
        EntrypointId(entrypoint_id.to_string().into())
    }
}

impl ToString for EntrypointId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    InProgress,
    Done,
    Failed {
        message: String
    },
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Json(String),
    Undefined,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RenderLocation {
    InlineView,
    View
}
