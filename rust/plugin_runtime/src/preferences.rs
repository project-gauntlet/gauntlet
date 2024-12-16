use common::model::EntrypointId;
use deno_core::futures::executor::block_on;
use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::api::{BackendForPluginRuntimeApi, BackendForPluginRuntimeApiProxy};
use crate::model::JsPreferenceUserData;

#[op2]
#[serde]
pub fn get_plugin_preferences(state: Rc<RefCell<OpState>>) -> anyhow::Result<HashMap<String, JsPreferenceUserData>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiProxy>()
            .clone();

        api
    };

    block_on(async {
        api.get_plugin_preferences().await
    })
}

#[op2]
#[serde]
pub fn get_entrypoint_preferences(state: Rc<RefCell<OpState>>, #[string] entrypoint_id: &str) -> anyhow::Result<HashMap<String, JsPreferenceUserData>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiProxy>()
            .clone();

        api
    };

    block_on(async {
        api.get_entrypoint_preferences(EntrypointId::from_string(entrypoint_id)).await
    })
}


#[op2(async)]
pub async fn plugin_preferences_required(state: Rc<RefCell<OpState>>) -> anyhow::Result<bool> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiProxy>()
            .clone();

        api
    };

    api.plugin_preferences_required().await
}

#[op2(async)]
pub async fn entrypoint_preferences_required(state: Rc<RefCell<OpState>>, #[string] entrypoint_id: String) -> anyhow::Result<bool> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiProxy>()
            .clone();

        api
    };

    api.entrypoint_preferences_required(EntrypointId::from_string(entrypoint_id)).await
}
