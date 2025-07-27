use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use deno_core::op2;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;

use crate::deno::GauntletJsError;

#[op2(async)]
pub async fn open_settings(state: Rc<RefCell<OpState>>) -> Result<(), GauntletJsError> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.ui_show_settings().await.map_err(Into::into)
}
