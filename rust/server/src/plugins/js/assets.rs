use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::js::{BackendForPluginRuntimeApiImpl, PluginData};
use deno_core::futures::executor::block_on;
use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::rc::Rc;
use common_plugin_runtime::backend_for_plugin_runtime_api::BackendForPluginRuntimeApi;

#[op2(async)]
#[buffer]
pub async fn asset_data(state: Rc<RefCell<OpState>>, #[string] path: String) -> anyhow::Result<Vec<u8>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data {:?}", path);

    api.get_asset_data(&path).await
}

#[op2]
#[buffer]
pub fn asset_data_blocking(state: Rc<RefCell<OpState>>, #[string] path: String) -> anyhow::Result<Vec<u8>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data blocking {:?}", path);

    block_on(async {
        let data = api.get_asset_data(&path).await?;

        Ok(data)
    })
}