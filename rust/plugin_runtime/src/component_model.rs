use std::collections::HashMap;
use gauntlet_component_model::{create_component_model, Component};

pub struct ComponentModel {
    components: HashMap<String, Component>,
}

impl ComponentModel {
    pub fn new() -> Self {
        let components = create_component_model()
            .into_iter()
            .filter_map(|component| {
                match &component {
                    Component::Standard { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                    Component::Root { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                    Component::TextPart { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                }
            })
            .collect();

        Self {
            components
        }
    }

    pub fn components(&self) -> &HashMap<String, Component> {
        &self.components
    }
}