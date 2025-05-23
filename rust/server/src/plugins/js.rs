use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context;
use futures::AsyncBufReadExt;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::KeyboardEventOrigin;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::SearchResultAccessory;
use gauntlet_common::model::SearchResultEntrypointType;
use gauntlet_common::model::UiPropertyValue;
use gauntlet_common::model::UiRenderLocation;
use gauntlet_common::model::UiWidgetId;
use gauntlet_common::rpc::frontend_api::FrontendApi;
use gauntlet_common::rpc::frontend_api::FrontendApiProxy;
use gauntlet_common::settings_env_data_to_string;
use gauntlet_common_plugin_runtime::api::handle_proxy_message;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::model::JsClipboardData;
use gauntlet_common_plugin_runtime::model::JsEvent;
use gauntlet_common_plugin_runtime::model::JsGeneratedSearchItem;
use gauntlet_common_plugin_runtime::model::JsGeneratedSearchItemAccessory;
use gauntlet_common_plugin_runtime::model::JsGeneratedSearchItemActionType;
use gauntlet_common_plugin_runtime::model::JsInit;
use gauntlet_common_plugin_runtime::model::JsKeyboardEventOrigin;
use gauntlet_common_plugin_runtime::model::JsMessage;
use gauntlet_common_plugin_runtime::model::JsPluginCode;
use gauntlet_common_plugin_runtime::model::JsPluginPermissions;
use gauntlet_common_plugin_runtime::model::JsPluginPermissionsExec;
use gauntlet_common_plugin_runtime::model::JsPluginPermissionsFileSystem;
use gauntlet_common_plugin_runtime::model::JsPluginPermissionsMainSearchBar;
use gauntlet_common_plugin_runtime::model::JsPluginRuntimeMessage;
use gauntlet_common_plugin_runtime::model::JsPreferenceUserData;
use gauntlet_common_plugin_runtime::model::JsUiPropertyValue;
use gauntlet_common_plugin_runtime::model::JsUiRenderLocation;
use gauntlet_common_plugin_runtime::recv_message;
use gauntlet_common_plugin_runtime::send_message;
use gauntlet_common_plugin_runtime::JsMessageSide;
use gauntlet_utils::channel::RequestResult;
use interprocess::local_socket::tokio::RecvHalf;
use interprocess::local_socket::tokio::SendHalf;
use interprocess::local_socket::traits::tokio::Listener;
use interprocess::local_socket::traits::tokio::Stream;
use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::ToFsName;
use interprocess::local_socket::ToNsName;
use interprocess::TryClone;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;

use crate::model::IntermediateUiEvent;
use crate::plugins::binary_data_gatherer::BinaryDataGatherer;
use crate::plugins::clipboard::Clipboard;
use crate::plugins::data_db_repository::db_entrypoint_from_str;
use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::data_db_repository::DbPluginClipboardPermissions;
use crate::plugins::data_db_repository::DbPluginEntrypointType;
use crate::plugins::data_db_repository::DbPluginPreference;
use crate::plugins::data_db_repository::DbPluginPreferenceUserData;
use crate::plugins::data_db_repository::DbReadPlugin;
use crate::plugins::data_db_repository::DbReadPluginEntrypoint;
use crate::plugins::icon_cache::IconCache;
use crate::plugins::run_status::RunStatusGuard;
use crate::search::SearchIndex;
use crate::search::SearchIndexItem;
use crate::search::SearchIndexItemAction;
use crate::search::SearchIndexItemActionActionType;
use crate::PLUGIN_CONNECT_ENV;
use crate::PLUGIN_UUID_ENV;

pub struct PluginRuntimeData {
    pub id: PluginId,
    pub uuid: String,
    pub name: String,
    pub entrypoint_names: HashMap<EntrypointId, String>,
    pub code: JsPluginCode,
    pub inline_view_entrypoint_id: Option<String>,
    pub permissions: PluginPermissions,
    pub command_receiver: tokio::sync::broadcast::Receiver<PluginCommand>,
    pub db_repository: DataDbRepository,
    pub search_index: SearchIndex,
    pub icon_cache: IconCache,
    pub frontend_api: FrontendApiProxy,
    pub dirs: Dirs,
    pub clipboard: Clipboard,
}

pub struct PluginPermissions {
    pub environment: Vec<String>,
    pub network: Vec<String>,
    pub filesystem: JsPluginPermissionsFileSystem,
    pub exec: JsPluginPermissionsExec,
    pub system: Vec<String>,
    pub clipboard: Vec<PluginPermissionsClipboard>,
    pub main_search_bar: Vec<JsPluginPermissionsMainSearchBar>,
}

#[derive(Clone, Debug)]
pub struct PluginRuntimePermissions {
    pub clipboard: Vec<PluginPermissionsClipboard>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum PluginPermissionsClipboard {
    Read,
    Write,
    Clear,
}

#[derive(Clone, Debug)]
pub enum PluginCommand {
    One { id: PluginId, data: OnePluginCommandData },
    All { data: AllPluginCommandData },
}

#[derive(Clone, Debug)]
pub enum OnePluginCommandData {
    RenderView {
        entrypoint_id: EntrypointId,
    },
    CloseView,
    RunCommand {
        entrypoint_id: String,
    },
    RunGeneratedEntrypoint {
        entrypoint_id: String,
        action_index: usize,
    },
    HandleViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    HandleKeyboardEvent {
        entrypoint_id: EntrypointId,
        origin: KeyboardEventOrigin,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    },
    RefreshSearchIndex,
}

#[derive(Clone, Debug)]
pub enum AllPluginCommandData {
    OpenInlineView { text: String },
}

pub async fn start_plugin_runtime(data: PluginRuntimeData, run_status_guard: RunStatusGuard) -> anyhow::Result<()> {
    let runtime_permissions = PluginRuntimePermissions {
        clipboard: data.permissions.clipboard,
    };

    let api = BackendForPluginRuntimeApiImpl::new(
        data.icon_cache.clone(),
        data.db_repository,
        data.search_index,
        data.clipboard,
        data.frontend_api,
        data.uuid.clone(),
        data.id.clone(),
        data.name,
        runtime_permissions,
    );

    let mut command_receiver = data.command_receiver;
    let cache = data.icon_cache;
    let plugin_uuid = data.uuid.clone();
    let plugin_id = data.id.clone();

    let plugin_id_str = plugin_id.to_string();
    let dev_plugin = plugin_id_str.starts_with("file://");

    let (stdout_file, stderr_file) = if dev_plugin {
        let (stdout_file, stderr_file) = data.dirs.plugin_log_files(&plugin_uuid);

        std::fs::create_dir_all(stdout_file.parent().unwrap())?;
        File::create(&stdout_file)?;

        let stdout_file = stdout_file
            .to_str()
            .context("non-uft8 paths are not supported")?
            .to_string();

        std::fs::create_dir_all(stderr_file.parent().unwrap())?;
        File::create(&stderr_file)?;

        let stderr_file = stderr_file
            .to_str()
            .context("non-uft8 paths are not supported")?
            .to_string();

        (Some(stdout_file), Some(stderr_file))
    } else {
        (None, None)
    };

    let home_dir = data.dirs.home_dir();
    let local_storage_dir = data.dirs.plugin_local_storage(&plugin_uuid);
    let uds_socket_file = data.dirs.plugin_uds_socket(&plugin_uuid);
    let plugin_cache_dir = data.dirs.plugin_cache(&plugin_uuid)?;
    let plugin_data_dir = data.dirs.plugin_data(&plugin_uuid)?;

    #[cfg(target_os = "windows")]
    let name_str = format!("project-gauntlet-{}", plugin_uuid);

    #[cfg(unix)]
    let name_str = uds_socket_file.clone();

    // namespaced, removed when both client and server disconnect
    #[cfg(target_os = "windows")]
    let name = name_str
        .clone()
        .to_ns_name::<interprocess::local_socket::GenericNamespaced>()?;

    // not namespaced, needs to be cleaned up manually,
    // by using close-behind semantics and additionally removing it before creating a new runtime
    #[cfg(unix)]
    let name = {
        let uds_socket_file = uds_socket_file.clone();

        // manually remove in case of unexpected situation where removing after connection did not work properly
        let _ = std::fs::remove_file(&uds_socket_file);

        std::fs::create_dir_all(&uds_socket_file.parent().unwrap())?;

        uds_socket_file.to_fs_name::<interprocess::os::unix::local_socket::FilesystemUdSocket>()?
    };

    let listener = ListenerOptions::new().name(name).reclaim_name(false).create_tokio()?;

    let home_dir = home_dir
        .to_str()
        .context("non-uft8 paths are not supported")?
        .to_string();

    let local_storage_dir = local_storage_dir
        .to_str()
        .context("non-uft8 paths are not supported")?
        .to_string();

    let uds_socket_file = uds_socket_file
        .to_str()
        .context("non-uft8 paths are not supported")?
        .to_string();

    let plugin_cache_dir = plugin_cache_dir
        .to_str()
        .context("non-uft8 paths are not supported")?
        .to_string();

    let plugin_data_dir = plugin_data_dir
        .to_str()
        .context("non-uft8 paths are not supported")?
        .to_string();

    let permissions = JsPluginPermissions {
        environment: data.permissions.environment,
        network: data.permissions.network,
        filesystem: data.permissions.filesystem,
        exec: data.permissions.exec,
        system: data.permissions.system,
        main_search_bar: data.permissions.main_search_bar,
    };

    let init = JsInit {
        plugin_id: plugin_id.clone(),
        plugin_uuid: plugin_uuid.clone(),
        code: data.code,
        permissions,
        inline_view_entrypoint_id: data.inline_view_entrypoint_id,
        entrypoint_names: data.entrypoint_names,
        dev_plugin,
        home_dir,
        local_storage_dir,
        plugin_cache_dir,
        plugin_data_dir,
        stdout_file,
        stderr_file,
    };

    let current_exe = std::env::current_exe().context("unable to get current_exe")?;

    #[cfg(not(feature = "scenario_runner"))]
    let mut runtime_process = std::process::Command::new(current_exe)
        .env(PLUGIN_CONNECT_ENV, name_str)
        .env(PLUGIN_UUID_ENV, plugin_uuid.clone())
        .spawn()
        .context("start plugin runtime process")?;

    // use only for debugging and scenario_runner, only works if only one plugin is enabled
    #[cfg(feature = "scenario_runner")]
    std::thread::spawn(move || gauntlet_plugin_runtime::run_plugin_runtime(name_str.to_str().unwrap().to_string()));

    let conn = listener.accept().await?;

    #[cfg(unix)]
    let _ = std::fs::remove_file(&uds_socket_file);

    let (mut recver, mut sender) = conn.split();

    send_message(JsMessageSide::Backend, &mut sender, init).await?;

    let sender = Arc::new(Mutex::new(sender));

    tokio::task::spawn({
        let sender = sender.clone();
        async move {
            run_status_guard.stopped().await;

            tracing::info!("Requesting plugin runtime to stop...");

            let mut sender = sender.lock().await;
            if let Err(err) = send_message(JsMessageSide::Backend, &mut sender, JsMessage::Stop).await {
                tracing::error!("Error when sending stop request to plugin runtime {:?}", err);
            }
        }
    });

    tokio::select! {
        result = {
            let sender = sender.clone();
            let plugin_id = plugin_id.clone();
            tokio::task::unconstrained(async move {
                loop {
                    if let Err(err) = event_loop(&mut command_receiver, &sender, plugin_id.clone()).await {
                        tracing::error!("Event loop faced an error {:?}", err);
                        break;
                    }
                }
            })
        } => {
            tracing::error!("Event loop has been stopped {:?}", plugin_id)
        }
        result = {
             tokio::task::unconstrained(async {
                 let sender = sender.clone();
                 loop {
                     match request_loop(&mut recver, &sender, &api).await {
                         Ok(stop) => {
                             if stop {
                                 tracing::debug!("Stopping request loop as requested by plugin runtime");
                                 break;
                             }
                         }
                         Err(err) => {
                             tracing::error!("Request loop faced an error {:?}", err);
                             break;
                         }
                     }
                 }
             })
        } => {
            tracing::debug!("Request loop has been stopped {:?}", plugin_id)
        }
    }

    drop((recver, sender));

    #[cfg(not(feature = "scenario_runner"))]
    {
        let code = runtime_process
            .wait()
            .context("Error while waiting for JS runtime process to finish")?
            .code();

        match code {
            Some(code) => {
                if code == 0 {
                    tracing::info!("Plugin Runtime was stopped successfully")
                } else {
                    tracing::error!("Runtime process finished with status code: {code}")
                }
            }
            None => tracing::error!("Process terminated by signal"),
        }
    }

    Ok(())
}

async fn event_loop(
    command_receiver: &mut tokio::sync::broadcast::Receiver<PluginCommand>,
    send: &Mutex<SendHalf>,
    plugin_id: PluginId,
) -> anyhow::Result<()> {
    let command = command_receiver.recv().await?;

    let event = match command {
        PluginCommand::One { id, data } => {
            if id != plugin_id {
                None
            } else {
                match data {
                    OnePluginCommandData::RenderView { entrypoint_id } => {
                        Some(IntermediateUiEvent::OpenView { entrypoint_id })
                    }
                    OnePluginCommandData::CloseView => Some(IntermediateUiEvent::CloseView),
                    OnePluginCommandData::RunCommand { entrypoint_id } => {
                        Some(IntermediateUiEvent::RunCommand { entrypoint_id })
                    }
                    OnePluginCommandData::RunGeneratedEntrypoint {
                        entrypoint_id,
                        action_index,
                    } => {
                        Some(IntermediateUiEvent::RunGeneratedEntrypoint {
                            entrypoint_id,
                            action_index,
                        })
                    }
                    OnePluginCommandData::HandleViewEvent {
                        widget_id,
                        event_name,
                        event_arguments,
                    } => {
                        Some(IntermediateUiEvent::HandleViewEvent {
                            widget_id,
                            event_name,
                            event_arguments,
                        })
                    }
                    OnePluginCommandData::HandleKeyboardEvent {
                        entrypoint_id,
                        origin,
                        key,
                        modifier_shift,
                        modifier_control,
                        modifier_alt,
                        modifier_meta,
                    } => {
                        Some(IntermediateUiEvent::HandleKeyboardEvent {
                            entrypoint_id,
                            origin,
                            key,
                            modifier_shift,
                            modifier_control,
                            modifier_alt,
                            modifier_meta,
                        })
                    }
                    OnePluginCommandData::RefreshSearchIndex => Some(IntermediateUiEvent::RefreshSearchIndex),
                }
            }
        }
        PluginCommand::All { data } => {
            match data {
                AllPluginCommandData::OpenInlineView { text } => Some(IntermediateUiEvent::OpenInlineView { text }),
            }
        }
    };

    if let Some(event) = event {
        let mut send = send.lock().await;

        send_message(
            JsMessageSide::Backend,
            &mut send,
            JsMessage::Event(from_intermediate_to_js_event(event)),
        )
        .await?;
    }

    Ok(())
}

async fn request_loop(
    recv: &mut RecvHalf,
    send: &Mutex<SendHalf>,
    api: &BackendForPluginRuntimeApiImpl,
) -> anyhow::Result<bool> {
    match recv_message::<JsPluginRuntimeMessage>(JsMessageSide::Backend, recv).await {
        Err(e) => Err(anyhow!("Unable to handle message: {:?}", e)),
        Ok(message) => {
            tracing::trace!("Handling js runtime message: {:?}", message);

            match message {
                JsPluginRuntimeMessage::Stopped => Ok(true),
                JsPluginRuntimeMessage::Request(message) => {
                    match handle_proxy_message(message, api).await {
                        Ok(response) => {
                            let mut send = send.lock().await;

                            tracing::trace!("Sending request response: {:?}", response);

                            send_message(JsMessageSide::Backend, &mut send, JsMessage::Response(Ok(response))).await?;

                            Ok(false)
                        }
                        Err(err) => {
                            let mut send = send.lock().await;

                            let err = format!("{:?}", err);

                            send_message(JsMessageSide::Backend, &mut send, JsMessage::Response(Err(err))).await?;

                            Ok(false)
                        }
                    }
                }
            }
        }
    }
}

fn from_intermediate_to_js_event(event: IntermediateUiEvent) -> JsEvent {
    match event {
        IntermediateUiEvent::OpenView { entrypoint_id } => {
            JsEvent::OpenView {
                entrypoint_id: entrypoint_id.to_string(),
            }
        }
        IntermediateUiEvent::CloseView => JsEvent::CloseView,
        IntermediateUiEvent::RunCommand { entrypoint_id } => JsEvent::RunCommand { entrypoint_id },
        IntermediateUiEvent::RunGeneratedEntrypoint {
            entrypoint_id,
            action_index,
        } => {
            JsEvent::RunGeneratedEntrypoint {
                entrypoint_id,
                action_index,
            }
        }
        IntermediateUiEvent::HandleViewEvent {
            widget_id,
            event_name,
            event_arguments,
        } => {
            let event_arguments = event_arguments
                .into_iter()
                .map(|arg| {
                    match arg {
                        UiPropertyValue::String(value) => JsUiPropertyValue::String { value },
                        UiPropertyValue::Number(value) => JsUiPropertyValue::Number { value },
                        UiPropertyValue::Bool(value) => JsUiPropertyValue::Bool { value },
                        UiPropertyValue::Undefined => JsUiPropertyValue::Undefined,
                        UiPropertyValue::Array(_) | UiPropertyValue::Bytes(_) | UiPropertyValue::Object(_) => {
                            todo!()
                        }
                    }
                })
                .collect();

            JsEvent::ViewEvent {
                widget_id,
                event_name,
                event_arguments,
            }
        }
        IntermediateUiEvent::HandleKeyboardEvent {
            entrypoint_id,
            origin,
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        } => {
            JsEvent::KeyboardEvent {
                entrypoint_id: entrypoint_id.to_string(),
                origin: match origin {
                    KeyboardEventOrigin::MainView => JsKeyboardEventOrigin::MainView,
                    KeyboardEventOrigin::PluginView => JsKeyboardEventOrigin::PluginView,
                },
                key: key.to_value(),
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            }
        }
        IntermediateUiEvent::OpenInlineView { text } => JsEvent::OpenInlineView { text },
        IntermediateUiEvent::RefreshSearchIndex => JsEvent::RefreshSearchIndex,
    }
}

#[derive(Clone)]
pub struct BackendForPluginRuntimeApiImpl {
    icon_cache: IconCache,
    repository: DataDbRepository,
    search_index: SearchIndex,
    clipboard: Clipboard,
    frontend_api: FrontendApiProxy,
    plugin_uuid: String,
    plugin_id: PluginId,
    plugin_name: String,
    permissions: PluginRuntimePermissions,
}

impl BackendForPluginRuntimeApiImpl {
    fn new(
        icon_cache: IconCache,
        repository: DataDbRepository,
        search_index: SearchIndex,
        clipboard: Clipboard,
        frontend_api: FrontendApiProxy,
        plugin_uuid: String,
        plugin_id: PluginId,
        plugin_name: String,
        permissions: PluginRuntimePermissions,
    ) -> Self {
        Self {
            icon_cache,
            repository,
            search_index,
            clipboard,
            frontend_api,
            plugin_uuid,
            plugin_id,
            plugin_name,
            permissions,
        }
    }
}

impl BackendForPluginRuntimeApi for BackendForPluginRuntimeApiImpl {
    async fn reload_search_index(
        &self,
        generated_entrypoints: Vec<JsGeneratedSearchItem>,
        refresh_search_list: bool,
    ) -> RequestResult<()> {
        let DbReadPlugin { name, .. } = self
            .repository
            .get_plugin_by_id(&self.plugin_id.to_string())
            .await
            .context("error when getting plugin by id")?;

        let entrypoints = self
            .repository
            .get_entrypoints_by_plugin_id(&self.plugin_id.to_string())
            .await
            .context("error when getting entrypoints by plugin id")?;

        let frecency_map = self
            .repository
            .get_frecency_for_plugin(&self.plugin_id.to_string())
            .await
            .context("error when getting frecency for plugin")?;

        let mut shortcuts = HashMap::new();

        for DbReadPluginEntrypoint { id, .. } in &entrypoints {
            let entrypoint_shortcuts = self
                .repository
                .action_shortcuts(&self.plugin_id.to_string(), id)
                .await?;
            shortcuts.insert(id.clone(), entrypoint_shortcuts);
        }

        let generator_names: HashMap<_, _> = entrypoints
            .iter()
            .filter(|entrypoint| {
                matches!(
                    db_entrypoint_from_str(&entrypoint.entrypoint_type),
                    DbPluginEntrypointType::EntrypointGenerator
                )
            })
            .map(|entrypoint| (entrypoint.id.clone(), entrypoint.name.clone()))
            .collect();

        let mut generated_search_items = generated_entrypoints
            .into_iter()
            .map(|item| {
                let entrypoint_icon = match item.entrypoint_icon {
                    None => None,
                    Some(data) => Some(bytes::Bytes::from(data)),
                };

                let entrypoint_frecency = frecency_map.get(&item.entrypoint_id).cloned().unwrap_or(0.0);

                let shortcuts = shortcuts.get(&item.generator_entrypoint_id);

                let entrypoint_actions = item
                    .entrypoint_actions
                    .iter()
                    .map(|action| {
                        let shortcut = match (shortcuts, &action.id) {
                            (Some(shortcuts), Some(id)) => shortcuts.get(id).cloned(),
                            _ => None,
                        };

                        SearchIndexItemAction {
                            id: action.id.clone(),
                            label: action.label.clone(),
                            action_type: match action.action_type {
                                JsGeneratedSearchItemActionType::View => SearchIndexItemActionActionType::View,
                                JsGeneratedSearchItemActionType::Command => SearchIndexItemActionActionType::Command,
                            },
                            shortcut,
                        }
                    })
                    .collect();

                let entrypoint_accessories = item
                    .entrypoint_accessories
                    .into_iter()
                    .map(|accessory| {
                        match accessory {
                            JsGeneratedSearchItemAccessory::TextAccessory { text, icon, tooltip } => {
                                SearchResultAccessory::TextAccessory { text, icon, tooltip }
                            }
                            JsGeneratedSearchItemAccessory::IconAccessory { icon, tooltip } => {
                                SearchResultAccessory::IconAccessory { icon, tooltip }
                            }
                        }
                    })
                    .collect();

                let entrypoint_generator = generator_names.get(&item.generator_entrypoint_id).map(|name| {
                    (
                        EntrypointId::from_string(item.generator_entrypoint_id),
                        name.to_string(),
                    )
                });

                Ok(SearchIndexItem {
                    entrypoint_type: SearchResultEntrypointType::Generated,
                    entrypoint_id: EntrypointId::from_string(item.entrypoint_id),
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_icon,
                    entrypoint_frecency,
                    entrypoint_actions,
                    entrypoint_accessories,
                    entrypoint_generator,
                })
            })
            .collect::<RequestResult<Vec<_>>>()?;

        let mut icon_asset_data = HashMap::new();

        for entrypoint in &entrypoints {
            if let Some(path_to_asset) = &entrypoint.icon_path {
                let result = self
                    .repository
                    .get_asset_data(&self.plugin_id.to_string(), path_to_asset)
                    .await;

                if let Ok(data) = result {
                    icon_asset_data.insert((entrypoint.id.clone(), path_to_asset.clone()), data);
                }
            }
        }

        let mut builtin_search_items = entrypoints
            .into_iter()
            .filter(|entrypoint| entrypoint.enabled)
            .map(|entrypoint| {
                let entrypoint_type = db_entrypoint_from_str(&entrypoint.entrypoint_type);
                let entrypoint_id = entrypoint.id.to_string();

                let entrypoint_frecency = frecency_map.get(&entrypoint_id).cloned().unwrap_or(0.0);

                let entrypoint_icon = match entrypoint.icon_path {
                    None => None,
                    Some(path_to_asset) => {
                        match icon_asset_data.get(&(entrypoint.id, path_to_asset)) {
                            None => None,
                            Some(data) => Some(bytes::Bytes::copy_from_slice(data)),
                        }
                    }
                };

                let entrypoint_id = EntrypointId::from_string(entrypoint_id);

                match &entrypoint_type {
                    DbPluginEntrypointType::Command => {
                        Ok(Some(SearchIndexItem {
                            entrypoint_type: SearchResultEntrypointType::Command,
                            entrypoint_name: entrypoint.name,
                            entrypoint_generator: None,
                            entrypoint_id,
                            entrypoint_icon,
                            entrypoint_frecency,
                            entrypoint_actions: vec![],
                            entrypoint_accessories: vec![],
                        }))
                    }
                    DbPluginEntrypointType::View => {
                        Ok(Some(SearchIndexItem {
                            entrypoint_type: SearchResultEntrypointType::View,
                            entrypoint_name: entrypoint.name,
                            entrypoint_generator: None,
                            entrypoint_id,
                            entrypoint_icon,
                            entrypoint_frecency,
                            entrypoint_actions: vec![],
                            entrypoint_accessories: vec![],
                        }))
                    }
                    DbPluginEntrypointType::EntrypointGenerator | DbPluginEntrypointType::InlineView => Ok(None),
                }
            })
            .collect::<RequestResult<Vec<_>>>()?
            .into_iter()
            .flat_map(|item| item)
            .collect::<Vec<_>>();

        generated_search_items.append(&mut builtin_search_items);

        self.search_index
            .save_for_plugin(
                self.plugin_id.clone(),
                name,
                generated_search_items,
                refresh_search_list,
            )
            .await
            .context("error when updating search index")?;

        Ok(())
    }

    async fn get_asset_data(&self, path: String) -> RequestResult<Vec<u8>> {
        let data = self
            .repository
            .get_asset_data(&self.plugin_id.to_string(), &path)
            .await?;

        Ok(data)
    }

    async fn get_entrypoint_generator_entrypoint_ids(&self) -> RequestResult<Vec<String>> {
        let result = self
            .repository
            .get_entrypoints_by_plugin_id(&self.plugin_id.to_string())
            .await?
            .into_iter()
            .filter(|entrypoint| entrypoint.enabled)
            .filter(|entrypoint| {
                matches!(
                    db_entrypoint_from_str(&entrypoint.entrypoint_type),
                    DbPluginEntrypointType::EntrypointGenerator
                )
            })
            .map(|entrypoint| entrypoint.id)
            .collect::<Vec<_>>();

        Ok(result)
    }

    async fn get_plugin_preferences(&self) -> RequestResult<HashMap<String, JsPreferenceUserData>> {
        let DbReadPlugin {
            preferences,
            preferences_user_data,
            ..
        } = self.repository.get_plugin_by_id(&self.plugin_id.to_string()).await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    }

    async fn get_entrypoint_preferences(
        &self,
        entrypoint_id: EntrypointId,
    ) -> RequestResult<HashMap<String, JsPreferenceUserData>> {
        let DbReadPluginEntrypoint {
            preferences,
            preferences_user_data,
            ..
        } = self
            .repository
            .get_entrypoint_by_id(&self.plugin_id.to_string(), &entrypoint_id.to_string())
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    }

    async fn plugin_preferences_required(&self) -> RequestResult<bool> {
        let DbReadPlugin {
            preferences,
            preferences_user_data,
            ..
        } = self.repository.get_plugin_by_id(&self.plugin_id.to_string()).await?;

        Ok(any_preferences_missing_value(preferences, preferences_user_data))
    }

    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> RequestResult<bool> {
        let DbReadPluginEntrypoint {
            preferences,
            preferences_user_data,
            ..
        } = self
            .repository
            .get_entrypoint_by_id(&self.plugin_id.to_string(), &entrypoint_id.to_string())
            .await?;

        Ok(any_preferences_missing_value(preferences, preferences_user_data))
    }

    async fn clipboard_read(&self) -> RequestResult<JsClipboardData> {
        let allow = self.permissions.clipboard.contains(&PluginPermissionsClipboard::Read);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'read' permission for clipboard").into());
        }

        tracing::debug!("Reading from clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.read().map_err(Into::into)
    }

    async fn clipboard_read_text(&self) -> RequestResult<Option<String>> {
        let allow = self.permissions.clipboard.contains(&PluginPermissionsClipboard::Read);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'read' permission for clipboard").into());
        }

        tracing::debug!("Reading text from clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.read_text().map_err(Into::into)
    }

    async fn clipboard_write(&self, data: JsClipboardData) -> RequestResult<()> {
        let allow = self.permissions.clipboard.contains(&PluginPermissionsClipboard::Write);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'write' permission for clipboard").into());
        }

        tracing::debug!("Writing to clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.write(data).map_err(Into::into)
    }

    async fn clipboard_write_text(&self, data: String) -> RequestResult<()> {
        let allow = self.permissions.clipboard.contains(&PluginPermissionsClipboard::Write);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'write' permission for clipboard").into());
        }

        tracing::debug!("Writing text to clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.write_text(data).map_err(Into::into)
    }

    async fn clipboard_clear(&self) -> RequestResult<()> {
        let allow = self.permissions.clipboard.contains(&PluginPermissionsClipboard::Clear);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'clear' permission for clipboard").into());
        }

        tracing::debug!("Clearing clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.clear().map_err(Into::into)
    }

    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> RequestResult<()> {
        self.frontend_api
            .update_loading_bar(self.plugin_id.clone(), entrypoint_id, show)
            .await?;

        Ok(())
    }

    async fn ui_show_hud(&self, display: String) -> RequestResult<()> {
        self.frontend_api.show_hud(display).await?;

        Ok(())
    }

    async fn ui_hide_window(&self) -> RequestResult<()> {
        self.frontend_api.hide_window().await?;

        Ok(())
    }

    async fn ui_get_action_id_for_shortcut(
        &self,
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> RequestResult<Option<String>> {
        let result = self
            .repository
            .get_action_id_for_shortcut(
                &self.plugin_id.to_string(),
                &entrypoint_id.to_string(),
                PhysicalKey::from_value(key),
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            )
            .await?;

        Ok(result)
    }

    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: JsUiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> RequestResult<()> {
        let data = BinaryDataGatherer::run_gatherer(&self, &container).await?;

        let render_location = match render_location {
            JsUiRenderLocation::InlineView => UiRenderLocation::InlineView,
            JsUiRenderLocation::View => UiRenderLocation::View,
        };

        self.frontend_api
            .replace_view(
                self.plugin_id.clone(),
                self.plugin_name.clone(),
                entrypoint_id,
                entrypoint_name,
                render_location,
                top_level_view,
                container,
                data,
            )
            .await?;

        Ok(())
    }

    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: JsUiRenderLocation,
    ) -> RequestResult<()> {
        let render_location = match render_location {
            JsUiRenderLocation::InlineView => UiRenderLocation::InlineView,
            JsUiRenderLocation::View => UiRenderLocation::View,
        };

        self.frontend_api
            .show_plugin_error_view(self.plugin_id.clone(), entrypoint_id, render_location)
            .await?;

        Ok(())
    }

    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> RequestResult<()> {
        self.frontend_api
            .show_preference_required_view(
                self.plugin_id.clone(),
                entrypoint_id,
                plugin_preferences_required,
                entrypoint_preferences_required,
            )
            .await?;

        Ok(())
    }

    async fn ui_clear_inline_view(&self) -> RequestResult<()> {
        self.frontend_api.clear_inline_view(self.plugin_id.clone()).await?;

        Ok(())
    }
}

fn preferences_to_js(
    preferences: HashMap<String, DbPluginPreference>,
    mut preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
) -> HashMap<String, JsPreferenceUserData> {
    preferences
        .into_iter()
        .map(|(name, preference)| {
            let user_data = match preferences_user_data.remove(&name) {
                None => {
                    match preference {
                        DbPluginPreference::Number { default, .. } => {
                            JsPreferenceUserData::Number(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreference::String { default, .. } => {
                            JsPreferenceUserData::String(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreference::Enum { default, .. } => {
                            JsPreferenceUserData::String(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreference::Bool { default, .. } => {
                            JsPreferenceUserData::Bool(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreference::ListOfStrings { default, .. } => {
                            JsPreferenceUserData::ListOfStrings(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreference::ListOfNumbers { default, .. } => {
                            JsPreferenceUserData::ListOfNumbers(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreference::ListOfEnums { default, .. } => {
                            JsPreferenceUserData::ListOfStrings(
                                default.expect("at this point preference should always have value"),
                            )
                        }
                    }
                }
                Some(user_data) => {
                    match user_data {
                        DbPluginPreferenceUserData::Number { value } => {
                            JsPreferenceUserData::Number(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreferenceUserData::String { value } => {
                            JsPreferenceUserData::String(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreferenceUserData::Enum { value } => {
                            JsPreferenceUserData::String(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreferenceUserData::Bool { value } => {
                            JsPreferenceUserData::Bool(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreferenceUserData::ListOfStrings { value } => {
                            JsPreferenceUserData::ListOfStrings(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreferenceUserData::ListOfNumbers { value } => {
                            JsPreferenceUserData::ListOfNumbers(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                        DbPluginPreferenceUserData::ListOfEnums { value } => {
                            JsPreferenceUserData::ListOfStrings(
                                value.expect("at this point preference should always have value"),
                            )
                        }
                    }
                }
            };

            (name, user_data)
        })
        .collect()
}

fn any_preferences_missing_value(
    preferences: HashMap<String, DbPluginPreference>,
    preferences_user_data: HashMap<String, DbPluginPreferenceUserData>,
) -> bool {
    for (name, preference) in preferences {
        match preferences_user_data.get(&name) {
            None => {
                let no_default = match preference {
                    DbPluginPreference::Number { default, .. } => default.is_none(),
                    DbPluginPreference::String { default, .. } => default.is_none(),
                    DbPluginPreference::Enum { default, .. } => default.is_none(),
                    DbPluginPreference::Bool { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfStrings { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfNumbers { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfEnums { default, .. } => default.is_none(),
                };

                if no_default {
                    return true;
                }
            }
            Some(preference) => {
                let no_value = match preference {
                    DbPluginPreferenceUserData::Number { value } => value.is_none(),
                    DbPluginPreferenceUserData::String { value } => value.is_none(),
                    DbPluginPreferenceUserData::Enum { value } => value.is_none(),
                    DbPluginPreferenceUserData::Bool { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfStrings { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfNumbers { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfEnums { value } => value.is_none(),
                };

                if no_value {
                    return true;
                }
            }
        }
    }

    false
}
