use crate::plugins::js::BackendForPluginRuntimeApiImpl;
use common_plugin_runtime::backend_for_plugin_runtime_api::BackendForPluginRuntimeApi;
use common_plugin_runtime::model::ClipboardData;
use deno_core::{op2, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
struct JSClipboardData {
    text_data: Option<String>,
    png_data: Option<Vec<u8>>
}

#[op2(async)]
#[serde]
pub async fn clipboard_read(state: Rc<RefCell<OpState>>) -> anyhow::Result<JSClipboardData> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    let result = api.clipboard_read().await?;

    Ok(JSClipboardData {
        text_data: result.text_data,
        png_data: result.png_data,
    })
}


#[op2(async)]
#[string]
pub async fn clipboard_read_text(state: Rc<RefCell<OpState>>) -> anyhow::Result<Option<String>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    api.clipboard_read_text().await
}

#[op2(async)]
pub async fn clipboard_write(state: Rc<RefCell<OpState>>, #[serde] data: JSClipboardData) -> anyhow::Result<()> { // TODO deserialization broken, fix when migrating to deno's op2
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    let clipboard_data = ClipboardData {
        text_data: data.text_data,
        png_data: data.png_data,
    };

    api.clipboard_write(clipboard_data).await
}

#[op2(async)]
pub async fn clipboard_write_text(state: Rc<RefCell<OpState>>, #[string] data: String) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    api.clipboard_write_text(data).await
}

#[op2(async)]
pub async fn clipboard_clear(state: Rc<RefCell<OpState>>) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    api.clipboard_clear().await
}
