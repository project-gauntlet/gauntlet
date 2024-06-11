use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use deno_core::{op, OpState};
use deno_core::futures::executor::block_on;
use crate::model::PreferenceUserData;
use crate::plugins::data_db_repository::{DataDbRepository, DbPluginPreference, DbPluginPreferenceUserData, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::js::PluginData;


#[op]
fn get_plugin_preferences(state: Rc<RefCell<OpState>>) -> anyhow::Result<HashMap<String, PreferenceUserData>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    block_on(async {
        let DbReadPlugin { preferences, preferences_user_data, .. } = repository
            .get_plugin_by_id(&plugin_id.to_string())
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    })
}

#[op]
fn get_entrypoint_preferences(state: Rc<RefCell<OpState>>, entrypoint_id: &str) -> anyhow::Result<HashMap<String, PreferenceUserData>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    block_on(async {
        let DbReadPluginEntrypoint { preferences, preferences_user_data, .. } = repository
            .get_entrypoint_by_id(&plugin_id.to_string(), entrypoint_id)
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    })
}


#[op]
async fn plugin_preferences_required(state: Rc<RefCell<OpState>>) -> anyhow::Result<bool> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let DbReadPlugin { preferences, preferences_user_data, .. } = repository
        .get_plugin_by_id(&plugin_id.to_string()).await?;

    Ok(all_preferences_required(preferences, preferences_user_data))
}

#[op]
async fn entrypoint_preferences_required(state: Rc<RefCell<OpState>>, entrypoint_id: String) -> anyhow::Result<bool> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let DbReadPluginEntrypoint { preferences, preferences_user_data, .. } = repository
        .get_entrypoint_by_id(&plugin_id.to_string(), &entrypoint_id).await?;

    Ok(all_preferences_required(preferences, preferences_user_data))
}


fn all_preferences_required(preferences: HashMap<String, DbPluginPreference>, preferences_user_data: HashMap<String, DbPluginPreferenceUserData>) -> bool {
    for (name, preference) in preferences {
        match preferences_user_data.get(&name) {
            None => {
                let no_default = match preference {
                    DbPluginPreference::Number { default, .. } => default.is_none(),
                    DbPluginPreference::String { default, .. } => default.is_none(),
                    DbPluginPreference::Enum { default, .. } => default.is_none(),
                    DbPluginPreference::Bool { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfStrings { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfNumbers { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfEnums { default, .. } => default.is_none(),
                };

                if no_default {
                    return true
                }
            }
            Some(preference) => {
                let no_value = match preference {
                    DbPluginPreferenceUserData::Number { value } => value.is_none(),
                    DbPluginPreferenceUserData::String { value } => value.is_none(),
                    DbPluginPreferenceUserData::Enum { value } => value.is_none(),
                    DbPluginPreferenceUserData::Bool { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfStrings { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfNumbers { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfEnums { value } => value.is_none(),
                };

                if no_value {
                    return true
                }
            }
        }
    }

    false
}


fn preferences_to_js(
    preferences: HashMap<String, DbPluginPreference>,
    mut preferences_user_data: HashMap<String, DbPluginPreferenceUserData>
) -> HashMap<String, PreferenceUserData> {
    preferences.into_iter()
        .map(|(name, preference)| {
            let user_data = match preferences_user_data.remove(&name) {
                None => match preference {
                    DbPluginPreference::Number { default, .. } => PreferenceUserData::Number(default.expect("at this point preference should always have value")),
                    DbPluginPreference::String { default, .. } => PreferenceUserData::String(default.expect("at this point preference should always have value")),
                    DbPluginPreference::Enum { default, .. } => PreferenceUserData::String(default.expect("at this point preference should always have value")),
                    DbPluginPreference::Bool { default, .. } => PreferenceUserData::Bool(default.expect("at this point preference should always have value")),
                    DbPluginPreference::ListOfStrings { default, .. } => PreferenceUserData::ListOfStrings(default.expect("at this point preference should always have value")),
                    DbPluginPreference::ListOfNumbers { default, .. } => PreferenceUserData::ListOfNumbers(default.expect("at this point preference should always have value")),
                    DbPluginPreference::ListOfEnums { default, .. } => PreferenceUserData::ListOfStrings(default.expect("at this point preference should always have value")),
                }
                Some(user_data) => match user_data {
                    DbPluginPreferenceUserData::Number { value } => PreferenceUserData::Number(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::String { value } => PreferenceUserData::String(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::Enum { value } => PreferenceUserData::String(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::Bool { value } => PreferenceUserData::Bool(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::ListOfStrings { value } => PreferenceUserData::ListOfStrings(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::ListOfNumbers { value } => PreferenceUserData::ListOfNumbers(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::ListOfEnums { value } => PreferenceUserData::ListOfStrings(value.expect("at this point preference should always have value")),
                }
            };

            (name, user_data)
        })
        .collect()
}
