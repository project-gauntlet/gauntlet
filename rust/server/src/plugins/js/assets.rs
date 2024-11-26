use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::js::PluginData;
use deno_core::futures::executor::block_on;
use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::rc::Rc;

#[op2(async)]
#[buffer]
pub async fn asset_data(state: Rc<RefCell<OpState>>, #[string] path: String) -> anyhow::Result<Vec<u8>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data {:?}", path);

    repository.get_asset_data(&plugin_id.to_string(), &path).await
}

#[op2]
#[buffer]
pub fn asset_data_blocking(state: Rc<RefCell<OpState>>, #[string] path: String) -> anyhow::Result<Vec<u8>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data blocking {:?}", path);

    block_on(async {
        let data = repository.get_asset_data(&plugin_id.to_string(), &path).await?;

        Ok(data)
    })
}