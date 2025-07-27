use gauntlet_utils::channel::RequestResult;
use gauntlet_utils_macros::boundary_gen;

use crate::model::EntrypointId;
use crate::model::LocalSaveData;
use crate::model::PluginId;

#[allow(async_fn_in_trait)]
#[boundary_gen(in_process)]
pub trait ServerGrpcApi {
    async fn show_window(&self) -> RequestResult<()>;

    async fn show_settings_window(&self) -> RequestResult<()>;

    async fn run_action(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_id: String,
    ) -> RequestResult<()>;

    async fn save_local_plugin(&self, path: String) -> RequestResult<LocalSaveData>;
}
