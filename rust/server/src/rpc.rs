use std::collections::HashMap;

use tonic::{Request, Response, Status};

use common::model::{DownloadStatus, EntrypointId, PluginId};
use common::rpc::{RpcDownloadPluginRequest, RpcDownloadPluginResponse, RpcDownloadStatus, RpcDownloadStatusRequest, RpcDownloadStatusResponse, RpcDownloadStatusValue, RpcEntrypointTypeSearchResult, RpcEventRenderView, RpcEventRunCommand, RpcEventViewEvent, RpcPlugin, RpcPluginsRequest, RpcPluginsResponse, RpcRequestRunCommandRequest, RpcRequestRunCommandResponse, RpcRequestRunGeneratedCommandRequest, RpcRequestRunGeneratedCommandResponse, RpcRequestViewRenderRequest, RpcRequestViewRenderResponse, RpcSaveLocalPluginRequest, RpcSaveLocalPluginResponse, RpcSearchRequest, RpcSearchResponse, RpcSearchResult, RpcSendKeyboardEventRequest, RpcSendKeyboardEventResponse, RpcSendViewEventRequest, RpcSendViewEventResponse, RpcSetEntrypointStateRequest, RpcSetEntrypointStateResponse, RpcSetPluginStateRequest, RpcSetPluginStateResponse, RpcSetPreferenceValueRequest, RpcSetPreferenceValueResponse};
use common::rpc::rpc_backend_server::{RpcBackend, RpcBackendServer};

use crate::model::from_rpc_to_intermediate_value;
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
        let frontend = event.frontend;

        self.application_manager.handle_render_view(PluginId::from_string(plugin_id), frontend, entrypoint_id);
        Ok(Response::new(RpcRequestViewRenderResponse::default()))
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
            .collect::<Option<Vec<_>>>()
            .ok_or(Status::invalid_argument("event_arguments"))?;

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
