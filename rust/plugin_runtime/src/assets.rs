use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use deno_core::futures::executor::block_on;
use deno_core::op2;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;

use crate::deno::GauntletJsError;

#[op2(async)]
#[buffer]
pub async fn asset_data(state: Rc<RefCell<OpState>>, #[string] path: String) -> Result<Vec<u8>, GauntletJsError> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data {:?}", path);

    api.get_asset_data(path).await.map_err(Into::into)
}

#[op2]
#[buffer]
pub fn asset_data_blocking(state: Rc<RefCell<OpState>>, #[string] path: String) -> Result<Vec<u8>, GauntletJsError> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data blocking {:?}", path);

    block_on(async {
        let data = api.get_asset_data(path).await?;

        Ok(data.into())
    })
}
