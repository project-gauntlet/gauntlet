use crate::model::UiRenderLocation;
use crate::scenario_model::ScenarioUiRenderLocation;

pub fn ui_render_location_to_scenario(render_location: UiRenderLocation) -> ScenarioUiRenderLocation {
    match render_location {
        UiRenderLocation::InlineView => ScenarioUiRenderLocation::InlineView,
        UiRenderLocation::View => ScenarioUiRenderLocation::View,
    }
}

pub fn ui_render_location_from_scenario(render_location: ScenarioUiRenderLocation) -> UiRenderLocation {
    match render_location {
        ScenarioUiRenderLocation::InlineView => UiRenderLocation::InlineView,
        ScenarioUiRenderLocation::View => UiRenderLocation::View,
    }
}
