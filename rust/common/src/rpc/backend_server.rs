use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tonic::{Request, Response, Status};
use tonic::transport::Server;

use crate::model::{ActionShortcut, DownloadStatus, EntrypointId, PhysicalKey, PluginId, PluginPreferenceUserData, SearchResult, SettingsEntrypointType, SettingsPlugin, UiPropertyValue, UiWidgetId};
use crate::rpc::grpc::{RpcDownloadPluginRequest, RpcDownloadPluginResponse, RpcDownloadStatus, RpcDownloadStatusRequest, RpcDownloadStatusResponse, RpcDownloadStatusValue, RpcEntrypoint, RpcEntrypointTypeSettings, RpcOpenSettingsWindowPreferencesRequest, RpcOpenSettingsWindowPreferencesResponse, RpcOpenSettingsWindowRequest, RpcOpenSettingsWindowResponse, RpcPingRequest, RpcPingResponse, RpcPlugin, RpcPluginsRequest, RpcPluginsResponse, RpcRemovePluginRequest, RpcRemovePluginResponse, RpcRequestRunCommandRequest, RpcRequestRunCommandResponse, RpcRequestRunGeneratedCommandRequest, RpcRequestRunGeneratedCommandResponse, RpcRequestViewCloseRequest, RpcRequestViewCloseResponse, RpcRequestViewRenderRequest, RpcRequestViewRenderResponse, RpcRequestViewRenderResponseAction, RpcSaveLocalPluginRequest, RpcSaveLocalPluginResponse, RpcSearchRequest, RpcSearchResponse, RpcSendKeyboardEventRequest, RpcSendKeyboardEventResponse, RpcSendOpenEventRequest, RpcSendOpenEventResponse, RpcSendViewEventRequest, RpcSendViewEventResponse, RpcSetEntrypointStateRequest, RpcSetEntrypointStateResponse, RpcSetPluginStateRequest, RpcSetPluginStateResponse, RpcSetPreferenceValueRequest, RpcSetPreferenceValueResponse};
use crate::rpc::grpc::rpc_backend_server::{RpcBackend, RpcBackendServer};
use crate::rpc::grpc_convert::{physical_key_from_rpc, plugin_preference_to_rpc, plugin_preference_user_data_from_rpc, plugin_preference_user_data_to_rpc, ui_property_value_from_rpc, ui_search_result_to_rpc};

pub async fn wait_for_backend_server() {
    loop {
        let addr: SocketAddr = "127.0.0.1:42320".parse().unwrap();

        if TcpStream::connect(addr).await.is_ok() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn start_backend_server(server: Box<dyn BackendServer + Sync + Send>) {
    let addr = "127.0.0.1:42320".parse().unwrap();

    Server::builder()
        .add_service(RpcBackendServer::new(RpcBackendServerImpl::new(server)))
        .serve(addr)
        .await
        .expect("unable to start backend server");
}

struct RpcBackendServerImpl {
    server: Box<dyn BackendServer + Sync + Send>
}

impl RpcBackendServerImpl {
    pub fn new(server: Box<dyn BackendServer + Sync + Send>) -> Self {
        Self {
            server
        }
    }
}

#[tonic::async_trait]
pub trait BackendServer {
    async fn search(
        &self,
        text: String,
        render_inline_view: bool,
    ) -> anyhow::Result<Vec<SearchResult>>;

    async fn request_view_render(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    ) -> anyhow::Result<HashMap<String, ActionShortcut>>;

    async fn request_view_close(
        &self,
        plugin_id: PluginId,
    ) -> anyhow::Result<()>;

    async fn request_run_command(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    ) -> anyhow::Result<()>;

    async fn request_run_generated_command(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    ) -> anyhow::Result<()>;

    async fn send_view_event(
        &self,
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>
    ) -> anyhow::Result<()>;

    async fn send_keyboard_event(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<()>;

    async fn send_open_event(
        &self,
        plugin_id: PluginId,
        href: String
    ) -> anyhow::Result<()>;

    async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>>;

    async fn set_plugin_state(
        &self,
        plugin_id: PluginId,
        enabled: bool
    ) -> anyhow::Result<()>;

    async fn set_entrypoint_state(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        enabled: bool
    ) -> anyhow::Result<()>;

    async fn set_preference_value(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        preference_name: String,
        preference_value: PluginPreferenceUserData
    ) -> anyhow::Result<()>;

    async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()>;

    async fn download_status(&self) -> anyhow::Result<HashMap<PluginId, DownloadStatus>>;

    async fn open_settings_window(&self) -> anyhow::Result<()>;

    async fn open_settings_window_preferences(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>
    ) -> anyhow::Result<()>;

    async fn remove_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()>;

    async fn save_local_plugin(&self, path: String) -> anyhow::Result<()>;
}


#[tonic::async_trait]
impl RpcBackend for RpcBackendServerImpl {
    async fn ping(&self, _: Request<RpcPingRequest>) -> Result<Response<RpcPingResponse>, Status> {
        Ok(Response::new(RpcPingResponse::default()))
    }

    async fn search(&self, request: Request<RpcSearchRequest>) -> Result<Response<RpcSearchResponse>, Status> {
        let request = request.into_inner();
        let text = request.text;
        let render_inline_view = request.render_inline_view;

        let result = self.server.search(text, render_inline_view)
            .await;

        let results = result
            .map_err(|err| Status::internal(err.to_string()))?
            .into_iter()
            .map(|item| ui_search_result_to_rpc(item))
            .collect();

        Ok(Response::new(RpcSearchResponse { results }))
    }

    async fn request_view_render(&self, request: Request<RpcRequestViewRenderRequest>) -> Result<Response<RpcRequestViewRenderResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        let action_shortcuts = self.server.request_view_render(plugin_id, entrypoint_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let action_shortcuts = action_shortcuts.into_iter()
            .map(|(id, shortcut)| {
                let action = RpcRequestViewRenderResponseAction {
                    key: shortcut.key.to_value(),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                };

                (id, action)
            })
            .collect();

        Ok(Response::new(RpcRequestViewRenderResponse {
            action_shortcuts,
        }))
    }

    async fn request_view_close(&self, request: Request<RpcRequestViewCloseRequest>) -> Result<Response<RpcRequestViewCloseResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server.request_view_close(plugin_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcRequestViewCloseResponse::default()))
    }

    async fn request_run_command(&self, request: Request<RpcRequestRunCommandRequest>) -> Result<Response<RpcRequestRunCommandResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.request_run_command(plugin_id, entrypoint_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcRequestRunCommandResponse::default()))
    }

    async fn request_run_generated_command(&self, request: Request<RpcRequestRunGeneratedCommandRequest>) -> Result<Response<RpcRequestRunGeneratedCommandResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let event = request.event.ok_or(Status::invalid_argument("event"))?;
        let entrypoint_id = event.entrypoint_id;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.request_run_generated_command(plugin_id, entrypoint_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

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
            .map(|arg| ui_property_value_from_rpc(arg))
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|err| Status::internal(err.to_string()))?;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server.send_view_event(plugin_id, widget_id, event_name, event_arguments)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

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

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.send_keyboard_event(plugin_id, entrypoint_id, physical_key_from_rpc(key), modifier_shift, modifier_control, modifier_alt, modifier_meta, )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSendKeyboardEventResponse::default()))
    }

    async fn send_open_event(&self, request: Request<RpcSendOpenEventRequest>) -> Result<Response<RpcSendOpenEventResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let href = request.href;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server.send_open_event(plugin_id, href)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSendOpenEventResponse::default()))
    }

    async fn plugins(&self, _: Request<RpcPluginsRequest>) -> Result<Response<RpcPluginsResponse>, Status> {
        let plugins = self.server.plugins()
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .into_iter()
            .map(|plugin| {
                let entrypoints = plugin.entrypoints
                    .into_iter()
                    .map(|(_, entrypoint)| RpcEntrypoint {
                        enabled: entrypoint.enabled,
                        entrypoint_id: entrypoint.entrypoint_id.to_string(),
                        entrypoint_name: entrypoint.entrypoint_name,
                        entrypoint_description: entrypoint.entrypoint_description,
                        entrypoint_type: match entrypoint.entrypoint_type {
                            SettingsEntrypointType::Command => RpcEntrypointTypeSettings::SCommand,
                            SettingsEntrypointType::View => RpcEntrypointTypeSettings::SView,
                            SettingsEntrypointType::InlineView => RpcEntrypointTypeSettings::SInlineView,
                            SettingsEntrypointType::CommandGenerator => RpcEntrypointTypeSettings::SCommandGenerator,
                        }.into(),
                        preferences: entrypoint.preferences.into_iter()
                            .map(|(key, value)| (key, plugin_preference_to_rpc(value)))
                            .collect(),
                        preferences_user_data: entrypoint.preferences_user_data.into_iter()
                            .map(|(key, value)| (key, plugin_preference_user_data_to_rpc(value)))
                            .collect(),
                    })
                    .collect();

                RpcPlugin {
                    plugin_id: plugin.plugin_id.to_string(),
                    plugin_name: plugin.plugin_name,
                    plugin_description: plugin.plugin_description,
                    enabled: plugin.enabled,
                    entrypoints,
                    preferences: plugin.preferences.into_iter()
                        .map(|(key, value)| (key, plugin_preference_to_rpc(value)))
                        .collect(),
                    preferences_user_data: plugin.preferences_user_data.into_iter()
                        .map(|(key, value)| (key, plugin_preference_user_data_to_rpc(value)))
                        .collect(),
                }
            })
            .collect();

        Ok(Response::new(RpcPluginsResponse { plugins }))
    }

    async fn set_plugin_state(&self, request: Request<RpcSetPluginStateRequest>) -> Result<Response<RpcSetPluginStateResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let enabled = request.enabled;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server.set_plugin_state(plugin_id, enabled)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSetPluginStateResponse::default()))
    }

    async fn set_entrypoint_state(&self, request: Request<RpcSetEntrypointStateRequest>) -> Result<Response<RpcSetEntrypointStateResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let enabled = request.enabled;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.set_entrypoint_state(plugin_id, entrypoint_id, enabled)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

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

        self.server.set_preference_value(plugin_id, entrypoint_id, preference_name, plugin_preference_user_data_from_rpc(preference_value))
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSetPreferenceValueResponse::default()))
    }

    async fn download_plugin(&self, request: Request<RpcDownloadPluginRequest>) -> Result<Response<RpcDownloadPluginResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server.download_plugin(plugin_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcDownloadPluginResponse::default()))
    }

    async fn download_status(&self, _: Request<RpcDownloadStatusRequest>) -> Result<Response<RpcDownloadStatusResponse>, Status> {
        let status_per_plugin = self.server.download_status()
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .into_iter()
            .map(|(plugin_id, status)| {
                let (status, message) = match status {
                    DownloadStatus::InProgress => (RpcDownloadStatus::InProgress, "".to_owned()),
                    DownloadStatus::Done => (RpcDownloadStatus::Done, "".to_owned()),
                    DownloadStatus::Failed { message } => (RpcDownloadStatus::Failed, message),
                };

                (plugin_id.to_string(), RpcDownloadStatusValue { status: status.into(), message })
            })
            .collect();

        let response = RpcDownloadStatusResponse {
            status_per_plugin,
        };

        Ok(Response::new(response))
    }

    async fn open_settings_window(&self, _: Request<RpcOpenSettingsWindowRequest>) -> Result<Response<RpcOpenSettingsWindowResponse>, Status> {
        self.server.open_settings_window()
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcOpenSettingsWindowResponse::default()))
    }

    async fn open_settings_window_preferences(&self, request: Request<RpcOpenSettingsWindowPreferencesRequest>) -> Result<Response<RpcOpenSettingsWindowPreferencesResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;

        let plugin_id = PluginId::from_string(plugin_id);

        let entrypoint_id = if entrypoint_id.is_empty() {
            None
        } else {
            Some(EntrypointId::from_string(entrypoint_id))
        };

        self.server.open_settings_window_preferences(plugin_id, entrypoint_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcOpenSettingsWindowPreferencesResponse::default()))
    }

    async fn remove_plugin(&self, request: Request<RpcRemovePluginRequest>) -> Result<Response<RpcRemovePluginResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server.remove_plugin(plugin_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcRemovePluginResponse::default()))
    }

    async fn save_local_plugin(&self, request: Request<RpcSaveLocalPluginRequest>) -> Result<Response<RpcSaveLocalPluginResponse>, Status> {
        let request = request.into_inner();
        let path = request.path;

        self.server.save_local_plugin(path)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RpcSaveLocalPluginResponse::default()))
    }
}
