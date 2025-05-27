use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use deno_core::op2;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;

use crate::deno::GauntletJsError;

#[op2(async)]
#[serde]
pub async fn get_entrypoint_generator_entrypoint_ids(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<String>, GauntletJsError> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.get_entrypoint_generator_entrypoint_ids().await.map_err(Into::into)
}
