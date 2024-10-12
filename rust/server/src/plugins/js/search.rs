use crate::plugins::data_db_repository::{db_entrypoint_from_str, DataDbRepository, DbPluginEntrypointType, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::icon_cache::IconCache;
use crate::plugins::js::PluginData;
use crate::search::{SearchIndex, SearchIndexItem, SearchIndexItemAction};
use anyhow::Context;
use common::model::{EntrypointId, SearchResultEntrypointType};
use deno_core::{op, OpState};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[op]
async fn reload_search_index(state: Rc<RefCell<OpState>>, generated_commands: Vec<AdditionalSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> {
    let (plugin_id, plugin_uuid, repository, mut search_index, icon_cache) = {
        let state = state.borrow();

        let plugin_data = state.borrow::<PluginData>();

        let plugin_id = plugin_data
            .plugin_id()
            .clone();

        let plugin_uuid = plugin_data
            .plugin_uuid()
            .to_owned();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        let search_index = state
            .borrow::<SearchIndex>()
            .clone();

        let icon_cache = state
            .borrow::<IconCache>()
            .clone();

        (plugin_id, plugin_uuid, repository, search_index, icon_cache)
    };

    icon_cache.clear_plugin_icon_cache_dir(&plugin_uuid)
        .context("error when clearing up icon cache before recreating it")?;

    let DbReadPlugin { name, .. } = repository.get_plugin_by_id(&plugin_id.to_string())
        .await
        .context("error when getting plugin by id")?;

    let entrypoints = repository.get_entrypoints_by_plugin_id(&plugin_id.to_string())
        .await
        .context("error when getting entrypoints by plugin id")?;

    let frecency_map = repository.get_frecency_for_plugin(&plugin_id.to_string())
        .await
        .context("error when getting frecency for plugin")?;

    let mut shortcuts = HashMap::new();

    for DbReadPluginEntrypoint { id, .. } in &entrypoints {
        let entrypoint_shortcuts = repository.action_shortcuts(&plugin_id.to_string(), id).await?;
        shortcuts.insert(id.clone(), entrypoint_shortcuts);
    }

    let mut plugins_search_items = generated_commands.into_iter()
        .map(|item| {
            let entrypoint_icon_path = match item.entrypoint_icon {
                None => None,
                Some(data) => Some(icon_cache.save_entrypoint_icon_to_cache(&plugin_uuid, &item.entrypoint_uuid, &data)?),
            };

            let entrypoint_frecency = frecency_map.get(&item.entrypoint_id).cloned().unwrap_or(0.0);

            let shortcuts = shortcuts
                .get(&item.generator_entrypoint_id);

            let entrypoint_actions = item.entrypoint_actions.iter()
                .map(|action| {
                    let shortcut = match (shortcuts, &action.id) {
                        (Some(shortcuts), Some(id)) => {
                            shortcuts.get(id).cloned()
                        }
                        _ => None
                    };

                    SearchIndexItemAction {
                        label: action.label.clone(),
                        shortcut,
                    }
                })
                .collect();

            Ok(SearchIndexItem {
                entrypoint_type: SearchResultEntrypointType::GeneratedCommand,
                entrypoint_id: EntrypointId::from_string(item.entrypoint_id),
                entrypoint_name: item.entrypoint_name,
                entrypoint_icon_path,
                entrypoint_frecency,
                entrypoint_actions,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut icon_asset_data = HashMap::new();

    for entrypoint in &entrypoints {
        if let Some(path_to_asset) = &entrypoint.icon_path {
            let result = repository.get_asset_data(&plugin_id.to_string(), path_to_asset)
                .await;

            if let Ok(data) = result {
                icon_asset_data.insert((entrypoint.id.clone(), path_to_asset.clone()), data);
            }
        }
    }

    let mut builtin_search_items = entrypoints.into_iter()
        .filter(|entrypoint| entrypoint.enabled)
        .map(|entrypoint| {
            let entrypoint_type = db_entrypoint_from_str(&entrypoint.entrypoint_type);
            let entrypoint_id = entrypoint.id.to_string();

            let entrypoint_frecency = frecency_map.get(&entrypoint_id).cloned().unwrap_or(0.0);

            let entrypoint_icon_path = match entrypoint.icon_path {
                None => None,
                Some(path_to_asset) => {
                    match icon_asset_data.get(&(entrypoint.id, path_to_asset)) {
                        None => None,
                        Some(data) => Some(icon_cache.save_entrypoint_icon_to_cache(&plugin_uuid, &entrypoint.uuid, data)?)
                    }
                },
            };

            let entrypoint_id = EntrypointId::from_string(entrypoint_id);

            match &entrypoint_type {
                DbPluginEntrypointType::Command => {
                    Ok(Some(SearchIndexItem {
                        entrypoint_type: SearchResultEntrypointType::Command,
                        entrypoint_name: entrypoint.name,
                        entrypoint_id,
                        entrypoint_icon_path,
                        entrypoint_frecency,
                        entrypoint_actions: vec![],
                    }))
                },
                DbPluginEntrypointType::View => {
                    Ok(Some(SearchIndexItem {
                        entrypoint_type: SearchResultEntrypointType::View,
                        entrypoint_name: entrypoint.name,
                        entrypoint_id,
                        entrypoint_icon_path,
                        entrypoint_frecency,
                        entrypoint_actions: vec![],
                    }))
                },
                DbPluginEntrypointType::CommandGenerator | DbPluginEntrypointType::InlineView => {
                    Ok(None)
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|item| item)
        .collect::<Vec<_>>();

    plugins_search_items.append(&mut builtin_search_items);

    search_index.save_for_plugin(plugin_id, name, plugins_search_items, refresh_search_list)
        .context("error when updating search index")?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AdditionalSearchItem {
    entrypoint_name: String,
    generator_entrypoint_id: String,
    entrypoint_id: String,
    entrypoint_uuid: String,
    entrypoint_icon: Option<Vec<u8>>,
    entrypoint_actions: Vec<AdditionalSearchItemAction>,
}

#[derive(Debug, Deserialize)]
pub struct AdditionalSearchItemAction {
    id: Option<String>,
    label: String,
}