use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::model::{RootWidget, UiWidgetId};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ScenarioUiRenderLocation {
    InlineView,
    View
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScenarioFrontendEvent {
    ReplaceView {
        entrypoint_id: String,
        render_location: ScenarioUiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
        #[serde(with="base64")]
        images: HashMap<UiWidgetId, Vec<u8>>,
    },
    ShowPreferenceRequiredView {
        entrypoint_id: String,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    ShowPluginErrorView {
        entrypoint_id: String,
        render_location: ScenarioUiRenderLocation,
    },
}

mod base64 {
    use std::collections::HashMap;
    use std::str::FromStr;
    use serde::{Serialize, Deserialize};
    use serde::{Deserializer, Serializer};
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use crate::model::UiWidgetId;

    pub fn serialize<S: Serializer>(v: &HashMap<UiWidgetId, Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
        let map = v.iter()
            .map(|(key, value)| (key.to_string(), STANDARD.encode(value)))
            .collect();

        HashMap::<String, String>::serialize(&map, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<HashMap<UiWidgetId, Vec<u8>>, D::Error> {
        HashMap::<String, String>::deserialize(d)?
            .into_iter()
            .map(|(key, value)| {
                STANDARD.decode(value.as_bytes())
                    .map_err(|e| serde::de::Error::custom(e))
                    .map(|vec| (UiWidgetId::from_str(&key).expect("should not fail"), vec))
            })
            .collect()

    }
}