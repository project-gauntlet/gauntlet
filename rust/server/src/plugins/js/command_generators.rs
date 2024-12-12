use crate::plugins::js::BackendForPluginRuntimeApiImpl;
use common_plugin_runtime::backend_for_plugin_runtime_api::BackendForPluginRuntimeApi;
use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::rc::Rc;

#[op2(async)]
#[serde]
pub async fn get_command_generator_entrypoint_ids(state: Rc<RefCell<OpState>>) -> anyhow::Result<Vec<String>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    api.get_command_generator_entrypoint_ids().await
}
