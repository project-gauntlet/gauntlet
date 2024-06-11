use deno_core::op;

#[op]
fn op_log_trace(target: String, message: String) {
    tracing::trace!(target = target, message)
}

#[op]
fn op_log_debug(target: String, message: String) {
    tracing::debug!(target = target, message)
}

#[op]
fn op_log_info(target: String, message: String) {
    tracing::info!(target = target, message)
}

#[op]
fn op_log_warn(target: String, message: String) {
    tracing::warn!(target = target, message)
}

#[op]
fn op_log_error(target: String, message: String) {
    tracing::error!(target = target, message)
}