use std::collections::HashMap;

use tonic::{Request, Response, Status};

use common::model::{DownloadStatus, EntrypointId, PluginId};
use common::rpc::{RpcDownloadPluginRequest, RpcDownloadPluginResponse, RpcDownloadStatus, RpcDownloadStatusRequest, RpcDownloadStatusResponse, RpcDownloadStatusValue, RpcEntrypointTypeSearchResult, RpcEventRenderView, RpcEventRunCommand, RpcEventViewEvent, RpcOpenSettingsWindowPreferencesRequest, RpcOpenSettingsWindowPreferencesResponse, RpcOpenSettingsWindowRequest, RpcOpenSettingsWindowResponse, RpcPlugin, RpcPluginsRequest, RpcPluginsResponse, RpcRemovePluginRequest, RpcRemovePluginResponse, RpcRequestRunCommandRequest, RpcRequestRunCommandResponse, RpcRequestRunGeneratedCommandRequest, RpcRequestRunGeneratedCommandResponse, RpcRequestViewRenderRequest, RpcRequestViewRenderResponse, RpcRequestViewRenderResponseAction, RpcRequestViewRenderResponseActionKind, RpcSaveLocalPluginRequest, RpcSaveLocalPluginResponse, RpcSearchRequest, RpcSearchResponse, RpcSearchResult, RpcSendKeyboardEventRequest, RpcSendKeyboardEventResponse, RpcSendOpenEventRequest, RpcSendOpenEventResponse, RpcSendViewEventRequest, RpcSendViewEventResponse, RpcSetEntrypointStateRequest, RpcSetEntrypointStateResponse, RpcSetPluginStateRequest, RpcSetPluginStateResponse, RpcSetPreferenceValueRequest, RpcSetPreferenceValueResponse, settings_env_data_to_string, SettingsEnvData};
use common::rpc::rpc_backend_server::{RpcBackend, RpcBackendServer};

use crate::{FRONTEND_ENV, SETTINGS_ENV};
use crate::model::{ActionShortcutKind, from_rpc_to_intermediate_value};
use crate::plugins::ApplicationManager;
use crate::search::{SearchIndex, SearchIndexPluginEntrypointType};

pub struct RpcBackendServerImpl {
    pub search_index: SearchIndex,
    pub application_manager: ApplicationManager,
}

#[tonic::async_trait]
impl RpcBackend for RpcBackendServerImpl {
    async fn search(&self, request: Request<RpcSearchRequest>) -> Result<Response<RpcSearchResponse>, Status> {
        let request = request.into_inner();
        let text = request.text;

        let results = self.search_index.create_handle()
            .search(&text)
            .map_err(|err| Status::internal(err.to_string()))?
            .into_iter()
            .map(|item| {
                let entrypoint_type = match item.entrypoint_type {
                    SearchIndexPluginEntrypointType::Command => RpcEntrypointTypeSearchResult::SrCommand,
                    SearchIndexPluginEntrypointType::View => RpcEntrypointTypeSearchResult::SrView,
                    SearchIndexPluginEntrypointType::GeneratedCommand => RpcEntrypointTypeSearchResult::SrGeneratedCommand,
                };

                RpcSearchResult {
                    entrypoint_type: entrypoint_type.into(),
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_id: item.entrypoint_id,
                    entrypoint_icon_path: item.entrypoint_icon_path.unwrap_or_default(),
                    plugin_name: item.plugin_name,
                    plugin_id: item.plugin_id,
                }
            })
            .collect();

        self.application_manager.handle_inline_view(&text);

        Ok(Response::new(RpcSearchResponse { results }))
    }

    async fn request_view_render(&self, request: Request<RpcRequestViewRenderRequest>) -> Result<Response<RpcRequestViewRenderResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);
        self.application_manager.handle_render_view(plugin_id.clone(), entrypoint_id.clone());

        let action_shortcuts = self.application_manager.action_shortcuts(plugin_id, entrypoint_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let action_shortcuts = action_shortcuts.into_iter()
            .map(|(id, shortcut)| {
                let action = RpcRequestViewRenderResponseAction {
                    key: shortcut.key,
                    kind: match shortcut.kind {
                        ActionShortcutKind::Main => RpcRequestViewRenderResponseActionKind::Main.into(),
                        ActionShortcutKind::Alternative => RpcRequestViewRenderResponseActionKind::Alternative.into(),
                    },
                };

                (id, action)
            })
            .collect();

        Ok(Response::new(RpcRequestViewRenderResponse {
            action_shortcuts,
        }))
    }

    async fn request_run_command(&self, request: Request<RpcRequestRunCommandRequest>) -> Result<Response<RpcRequestRunCommandResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;

        self.application_manager.handle_run_command(PluginId::from_string(plugin_id), entrypoint_id);
        Ok(Response::new(RpcRequestRunCommandResponse::default()))
    }

    async fn request_run_generated_command(&self, request: Request<RpcRequestRunGeneratedCommandRequest>) -> Result<Response<RpcRequestRunGeneratedCommandResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;

        self.application_manager.handle_run_generated_command(PluginId::from_string(plugin_id), entrypoint_id);
        Ok(Response::new(RpcRequestRunGeneratedCommandResponse::default()))
    }


    async fn send_view_event(&self, request: Request<RpcSendViewEventRequest>) -> Result<Response<RpcSendViewEventResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let widget_id = event.widget_id.ok_or(Status::invalid_argument("widget_id"))?.value;
        let event_name = event.event_name;
        let event_arguments = event.event_arguments;

        let event_arguments = event_arguments.into_iter()
            .map(|arg| from_rpc_to_intermediate_value(arg))
            .collect::<Vec<_>>();

        self.application_manager.handle_view_event(PluginId::from_string(plugin_id), widget_id, event_name, event_arguments);
        Ok(Response::new(RpcSendViewEventResponse::default()))
    }

    async fn send_keyboard_event(&self, request: Request<RpcSendKeyboardEventRequest>) -> Result<Response<RpcSendKeyboardEventResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;
        let key = event.key;
        let modifier_shift = event.modifier_shift;
        let modifier_control = event.modifier_control;
        let modifier_alt = event.modifier_alt;
        let modifier_meta = event.modifier_meta;

        self.application_manager.handle_keyboard_event(
            PluginId::from_string(plugin_id),
            EntrypointId::from_string(entrypoint_id),
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        );

        Ok(Response::new(RpcSendKeyboardEventResponse::default()))
    }

    async fn send_open_event(&self, request: Request<RpcSendOpenEventRequest>) -> Result<Response<RpcSendOpenEventResponse>, Status> {
        let request = request.into_inner();
        let _plugin_id = request.plugin_id;
        let href = request.href;

        match open::that(&href) {
            Ok(()) => tracing::info!("Opened '{}' successfully.", href),
            Err(err) => tracing::error!("An error occurred when opening '{}': {}", href, err),
        }

        Ok(Response::new(RpcSendOpenEventResponse::default()))
    }


    async fn plugins(&self, _: Request<RpcPluginsRequest>) -> Result<Response<RpcPluginsResponse>, Status> {
        let result = self.application_manager.plugins()
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'plugins' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))
            .map(|plugins| Response::new(RpcPluginsResponse { plugins }))
    }

    async fn set_plugin_state(&self, request: Request<RpcSetPluginStateRequest>) -> Result<Response<RpcSetPluginStateResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let enabled = request.enabled;

        let result = self.application_manager.set_plugin_state(PluginId::from_string(plugin_id), enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_plugin_state' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSetPluginStateResponse::default()))
    }

    async fn set_entrypoint_state(&self, request: Request<RpcSetEntrypointStateRequest>) -> Result<Response<RpcSetEntrypointStateResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let enabled = request.enabled;

        let result = self.application_manager.set_entrypoint_state(PluginId::from_string(plugin_id), EntrypointId::from_string(entrypoint_id), enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_entrypoint_state' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSetEntrypointStateResponse::default()))
    }

    async fn set_preference_value(&self, request: Request<RpcSetPreferenceValueRequest>) -> Result<Response<RpcSetPreferenceValueResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let plugin_id = PluginId::from_string(plugin_id);

        let entrypoint_id = if request.entrypoint_id.is_empty() {
            None
        } else {
            Some(EntrypointId::from_string(request.entrypoint_id))
        };

        let preference_name = request.preference_name;
        let preference_value = request.preference_value.unwrap();

        let result = self.application_manager.set_preference_value(plugin_id, entrypoint_id, preference_name, preference_value)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_preference_value' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSetPreferenceValueResponse::default()))
    }

    async fn download_plugin(&self, request: Request<RpcDownloadPluginRequest>) -> Result<Response<RpcDownloadPluginResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let result = self.application_manager.download_plugin(PluginId::from_string(plugin_id))
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'download_plugin' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcDownloadPluginResponse::default()))
    }

    async fn download_status(&self, _: Request<RpcDownloadStatusRequest>) -> Result<Response<RpcDownloadStatusResponse>, Status> {
        let status_per_plugin = self.application_manager.download_status()
            .into_iter()
            .map(|(plugin_id, status)| {
                let (status, message) = match status {
                    DownloadStatus::InProgress => (RpcDownloadStatus::InProgress, "".to_owned()),
                    DownloadStatus::Done => (RpcDownloadStatus::Done, "".to_owned()),
                    DownloadStatus::Failed { message } => (RpcDownloadStatus::Failed, message),
                };

                (plugin_id, RpcDownloadStatusValue { status: status.into(), message })
            })
            .collect();

        let response = RpcDownloadStatusResponse {
            status_per_plugin,
        };

        Ok(Response::new(response))
    }

    async fn open_settings_window(&self, _: Request<RpcOpenSettingsWindowRequest>) -> Result<Response<RpcOpenSettingsWindowResponse>, Status> {
        std::process::Command::new(std::env::current_exe()?)
            .args(["management"])
            .spawn()
            .expect("failed to execute settings process");

        Ok(Response::new(RpcOpenSettingsWindowResponse::default()))
    }

    async fn open_settings_window_preferences(&self, request: Request<RpcOpenSettingsWindowPreferencesRequest>) -> Result<Response<RpcOpenSettingsWindowPreferencesResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;

        let data = if entrypoint_id.is_empty() {
            SettingsEnvData::OpenPluginPreferences { plugin_id }
        } else {
            SettingsEnvData::OpenEntrypointPreferences { plugin_id, entrypoint_id }
        };

        std::process::Command::new(std::env::current_exe()?)
            .args(["management"])
            .env(SETTINGS_ENV, settings_env_data_to_string(data))
            .spawn()
            .expect("failed to execute settings process"); // this can fail in dev if binary was replaced by frontend compilation

        Ok(Response::new(RpcOpenSettingsWindowPreferencesResponse::default()))
    }

    async fn remove_plugin(&self, request: Request<RpcRemovePluginRequest>) -> Result<Response<RpcRemovePluginResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let result = self.application_manager.remove_plugin(PluginId::from_string(plugin_id))
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'remove_plugin' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcRemovePluginResponse::default()))
    }

    async fn save_local_plugin(&self, request: Request<RpcSaveLocalPluginRequest>) -> Result<Response<RpcSaveLocalPluginResponse>, Status> {
        let request = request.into_inner();
        let path = request.path;

        let result = self.application_manager.save_local_plugin(&path)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'save_local_plugin' request {:?}", err)
        }

        result.map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSaveLocalPluginResponse::default()))
    }
}
