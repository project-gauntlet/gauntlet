use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use deno_core::op2;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;
use gauntlet_common_plugin_runtime::model::JsClipboardData;

use crate::model::DenoInClipboardData;
use crate::model::DenoOutClipboardData;

#[op2(async)]
#[serde]
pub async fn clipboard_read(state: Rc<RefCell<OpState>>) -> anyhow::Result<DenoOutClipboardData> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    let result = api.clipboard_read().await?;

    Ok(DenoOutClipboardData {
        text_data: result.text_data,
        png_data: result.png_data.map(|buffer| buffer.into()),
    })
}

#[op2(async)]
#[string]
pub async fn clipboard_read_text(state: Rc<RefCell<OpState>>) -> anyhow::Result<Option<String>> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.clipboard_read_text().await.map_err(Into::into)
}

#[op2(async)]
pub async fn clipboard_write(state: Rc<RefCell<OpState>>, #[serde] data: DenoInClipboardData) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    let clipboard_data = JsClipboardData {
        text_data: data.text_data,
        png_data: data.png_data.map(|buffer| buffer.to_vec()),
    };

    api.clipboard_write(clipboard_data).await.map_err(Into::into)
}

#[op2(async)]
pub async fn clipboard_write_text(state: Rc<RefCell<OpState>>, #[string] data: String) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.clipboard_write_text(data).await.map_err(Into::into)
}

#[op2(async)]
pub async fn clipboard_clear(state: Rc<RefCell<OpState>>) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.clipboard_clear().await.map_err(Into::into)
}
