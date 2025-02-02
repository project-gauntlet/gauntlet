use std::cell::RefCell;
use std::rc::Rc;

use deno_core::op2;
use deno_core::OpState;

use crate::api::BackendForPluginRuntimeApi;
use crate::api::BackendForPluginRuntimeApiProxy;
use crate::model::JsGeneratedSearchItem;

#[op2(async)]
pub async fn reload_search_index(
    state: Rc<RefCell<OpState>>,
    #[serde] generated_entrypoints: Vec<JsGeneratedSearchItem>,
    refresh_search_list: bool,
) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.reload_search_index(generated_entrypoints, refresh_search_list)
        .await?;

    Ok(())
}
