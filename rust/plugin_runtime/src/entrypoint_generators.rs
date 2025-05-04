use std::cell::RefCell;
use std::rc::Rc;

use deno_core::op2;
use deno_core::OpState;

use crate::api::BackendForPluginRuntimeApi;
use crate::api::BackendForPluginRuntimeApiProxy;

#[op2(async)]
#[serde]
pub async fn get_entrypoint_generator_entrypoint_ids(state: Rc<RefCell<OpState>>) -> anyhow::Result<Vec<String>> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.get_entrypoint_generator_entrypoint_ids().await.map_err(Into::into)
}
