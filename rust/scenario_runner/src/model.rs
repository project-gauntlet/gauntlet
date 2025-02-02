use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScenarioBackendEvent {
    Search { text: String },
    RequestViewRender,
}
