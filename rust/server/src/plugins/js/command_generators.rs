use crate::plugins::data_db_repository::{db_entrypoint_from_str, DataDbRepository, DbPluginEntrypointType};
use crate::plugins::js::PluginData;
use deno_core::{op, OpState};
use std::cell::RefCell;
use std::rc::Rc;


#[op]
async fn get_command_generator_entrypoint_ids(state: Rc<RefCell<OpState>>) -> anyhow::Result<Vec<String>> {
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

    let result = repository.get_entrypoints_by_plugin_id(&plugin_id.to_string()).await?
        .into_iter()
        .filter(|entrypoint| entrypoint.enabled)
        .filter(|entrypoint| matches!(db_entrypoint_from_str(&entrypoint.entrypoint_type), DbPluginEntrypointType::CommandGenerator))
        .map(|entrypoint| entrypoint.id)
        .collect::<Vec<_>>();

    Ok(result)
}
