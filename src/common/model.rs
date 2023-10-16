use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(Arc<str>);

impl PluginId {
    pub fn new(plugin_id: impl ToString) -> Self {
        PluginId(plugin_id.to_string().into())
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
    pub fn new(entrypoint_id: impl ToString) -> Self {
        EntrypointId(entrypoint_id.to_string().into())
    }
}

impl ToString for EntrypointId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
