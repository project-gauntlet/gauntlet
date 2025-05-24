use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use deno_core::op2;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;
use gauntlet_common_plugin_runtime::model::JsGeneratedSearchItem;

use crate::model::DenoInGeneratedSearchItem;

#[op2(async)]
pub async fn reload_search_index(
    state: Rc<RefCell<OpState>>,
    #[serde] generated_entrypoints: Vec<DenoInGeneratedSearchItem>,
    refresh_search_list: bool,
) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    let generated_entrypoints = generated_entrypoints
        .into_iter()
        .map(|item| {
            JsGeneratedSearchItem {
                entrypoint_name: item.entrypoint_name,
                generator_entrypoint_id: item.generator_entrypoint_id,
                entrypoint_id: item.entrypoint_id,
                entrypoint_uuid: item.entrypoint_uuid,
                entrypoint_icon: item.entrypoint_icon.map(|buffer| buffer.to_vec()),
                entrypoint_actions: item.entrypoint_actions,
                entrypoint_accessories: item.entrypoint_accessories,
            }
        })
        .collect();

    api.reload_search_index(generated_entrypoints, refresh_search_list)
        .await?;

    Ok(())
}
