use std::cell::RefCell;
use std::rc::Rc;
use deno_core::{op, OpState};
use crate::plugins::js::PluginData;

#[op]
fn op_log_trace(state: Rc<RefCell<OpState>>, target: String, message: String) -> anyhow::Result<()> {
    let plugin_id = state.borrow()
        .borrow::<PluginData>()
        .plugin_id()
        .to_string();

    tracing::trace!(target = target, plugin_id = plugin_id, message);

    Ok(())
}

#[op]
fn op_log_debug(state: Rc<RefCell<OpState>>, target: String, message: String) -> anyhow::Result<()> {
    let plugin_id = state.borrow()
        .borrow::<PluginData>()
        .plugin_id()
        .to_string();

    tracing::debug!(target = target, plugin_id = plugin_id, message);

    Ok(())
}

#[op]
fn op_log_info(state: Rc<RefCell<OpState>>, target: String, message: String) -> anyhow::Result<()> {
    let plugin_id = state.borrow()
        .borrow::<PluginData>()
        .plugin_id()
        .to_string();

    tracing::info!(target = target, plugin_id = plugin_id, message);

    Ok(())
}

#[op]
fn op_log_warn(state: Rc<RefCell<OpState>>, target: String, message: String) -> anyhow::Result<()> {
    let plugin_id = state.borrow()
        .borrow::<PluginData>()
        .plugin_id()
        .to_string();

    tracing::warn!(target = target, plugin_id = plugin_id, message);

    Ok(())
}

#[op]
fn op_log_error(state: Rc<RefCell<OpState>>, target: String, message: String) -> anyhow::Result<()> {
    let plugin_id = state.borrow()
        .borrow::<PluginData>()
        .plugin_id()
        .to_string();

    tracing::error!(target = target, plugin_id = plugin_id, message);

    Ok(())
}