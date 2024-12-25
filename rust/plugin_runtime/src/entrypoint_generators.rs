use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::rc::Rc;
use crate::api::{BackendForPluginRuntimeApi, BackendForPluginRuntimeApiProxy};

#[op2(async)]
#[serde]
pub async fn get_entrypoint_generator_entrypoint_ids(state: Rc<RefCell<OpState>>) -> anyhow::Result<Vec<String>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiProxy>()
            .clone();

        api
    };

    api.get_entrypoint_generator_entrypoint_ids().await
}
