use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginUuid(Arc<str>);

impl PluginUuid {
    pub fn new(plugin_uuid: impl ToString) -> Self {
        PluginUuid(plugin_uuid.to_string().into())
    }
}

impl ToString for PluginUuid {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntrypointUuid(Arc<str>);

impl EntrypointUuid {
    pub fn new(entrypoint_uuid: impl ToString) -> Self {
        EntrypointUuid(entrypoint_uuid.to_string().into())
    }
}

impl ToString for EntrypointUuid {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
