use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScenarioBackendEvent {
    Search {
        text: String
    },
    RequestViewRender,
}
