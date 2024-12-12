use common_plugin_runtime::backend_for_plugin_runtime_api::BackendForPluginRuntimeApi;
use common_plugin_runtime::model::AdditionalSearchItem;
use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::rc::Rc;
use crate::plugins::js::BackendForPluginRuntimeApiImpl;

#[op2(async)]
pub async fn reload_search_index(state: Rc<RefCell<OpState>>, #[serde] generated_commands: Vec<AdditionalSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    api.reload_search_index(generated_commands, refresh_search_list).await?;

    Ok(())
}
