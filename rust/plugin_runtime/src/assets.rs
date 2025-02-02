use std::cell::RefCell;
use std::rc::Rc;

use deno_core::futures::executor::block_on;
use deno_core::op2;
use deno_core::OpState;
use tokio::runtime::Handle;

use crate::api::BackendForPluginRuntimeApi;
use crate::api::BackendForPluginRuntimeApiProxy;

#[op2(async)]
#[buffer]
pub async fn asset_data(state: Rc<RefCell<OpState>>, #[string] path: String) -> anyhow::Result<Vec<u8>> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data {:?}", path);

    api.get_asset_data(&path).await
}

#[op2]
#[buffer]
pub fn asset_data_blocking(state: Rc<RefCell<OpState>>, #[string] path: String) -> anyhow::Result<Vec<u8>> {
    let (api, outer_handle) = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        let outer_handle = state.borrow::<Handle>().clone();

        (api, outer_handle)
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data blocking {:?}", path);

    outer_handle.block_on(async {
        let data = api.get_asset_data(&path).await?;

        Ok(data)
    })
}
