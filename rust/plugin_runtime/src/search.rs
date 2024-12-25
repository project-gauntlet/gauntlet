use deno_core::{op2, OpState};
use std::cell::RefCell;
use std::rc::Rc;
use crate::api::{BackendForPluginRuntimeApi, BackendForPluginRuntimeApiProxy};
use crate::model::JsGeneratedSearchItem;

#[op2(async)]
pub async fn reload_search_index(state: Rc<RefCell<OpState>>, #[serde] generated_commands: Vec<JsGeneratedSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiProxy>()
            .clone();

        api
    };

    api.reload_search_index(generated_commands, refresh_search_list).await?;

    Ok(())
}
