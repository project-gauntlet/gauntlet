use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use futures::Sink;
use futures::SinkExt;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::Code;
use global_hotkey::hotkey::HotKey;
use global_hotkey::hotkey::Modifiers;
use tokio::runtime::Handle;

use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::data_db_repository::DbSettingsGlobalEntrypointShortcutData;
use crate::plugins::data_db_repository::DbSettingsGlobalShortcutData;
use crate::plugins::data_db_repository::DbSettingsShortcut;

#[derive(Clone)]
pub struct GlobalShortcutSettings {
    repository: DataDbRepository,
    state: Arc<Mutex<GlobalShortcutSettingsState>>,
}

struct GlobalShortcutSettingsState {
    current_global_hotkey: Option<HotKey>,
    current_entrypoint_global_hotkeys: HashMap<(PluginId, EntrypointId), HotKey>,
}

impl GlobalShortcutSettings {
    pub fn new(db_repository: DataDbRepository) -> anyhow::Result<GlobalShortcutSettings> {
        Ok(Self {
            repository: db_repository,
            state: Arc::new(Mutex::new(GlobalShortcutSettingsState {
                current_global_hotkey: None,
                current_entrypoint_global_hotkeys: HashMap::new(),
            })),
        })
    }

    pub fn setup(&self, global_hotkey_manager: &GlobalHotKeyManager) -> anyhow::Result<()> {
        let global_shortcut = self.global_shortcut()?.map(|(shortcut, _)| shortcut);
        let global_entrypoint_shortcuts = self
            .global_entrypoint_shortcuts()?
            .into_iter()
            .map(|((plugin_id, entrypoint_id), (shortcut, _))| ((plugin_id, entrypoint_id), shortcut))
            .collect();

        let (hotkeys, errors) = self.setup_shortcuts(global_hotkey_manager, global_shortcut);
        self.set_global_shortcut_error(errors)?;

        let (entrypoint_hotkeys, entrypoint_errors) =
            self.setup_entrypoint_shortcuts(global_hotkey_manager, global_entrypoint_shortcuts);
        for ((plugin_id, entrypoint_id), error) in entrypoint_errors {
            self.set_global_entrypoint_shortcut_error(plugin_id, entrypoint_id, error)?;
        }

        let mut state = self.state.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        state.current_global_hotkey = hotkeys;
        state.current_entrypoint_global_hotkeys = entrypoint_hotkeys;

        Ok(())
    }

    pub fn global_shortcut(&self) -> anyhow::Result<Option<(PhysicalShortcut, Option<String>)>> {
        let settings = self.repository.get_settings()?;

        let data = settings.global_shortcut.map(|data| {
            let shortcut = PhysicalShortcut {
                physical_key: PhysicalKey::from_value(data.shortcut.physical_key),
                modifier_shift: data.shortcut.modifier_shift,
                modifier_control: data.shortcut.modifier_control,
                modifier_alt: data.shortcut.modifier_alt,
                modifier_meta: data.shortcut.modifier_meta,
            };

            (shortcut, data.error)
        });

        Ok(data)
    }

    pub fn set_global_shortcut(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        shortcut: Option<PhysicalShortcut>,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        let (hotkey, err) =
            self.assign_global_shortcut(global_hotkey_manager, state.current_global_hotkey, shortcut.clone());

        state.current_global_hotkey = hotkey;

        match &err {
            Ok(()) => {
                tracing::info!("Successfully registered new global shortcut: {:?}", shortcut);
            }
            Err(err) => {
                tracing::error!("Unable to register new global shortcut {:?}: {:?}", shortcut, err);
            }
        }

        let db_err = err.as_ref().map_err(|err| format!("{:#}", err)).err();

        self.repository.mutate_settings(|mut settings| {
            settings.global_shortcut = shortcut.map(|shortcut| {
                DbSettingsGlobalShortcutData {
                    shortcut: DbSettingsShortcut {
                        physical_key: shortcut.physical_key.to_value(),
                        modifier_shift: shortcut.modifier_shift,
                        modifier_control: shortcut.modifier_control,
                        modifier_alt: shortcut.modifier_alt,
                        modifier_meta: shortcut.modifier_meta,
                    },
                    error: db_err,
                }
            });

            Ok(settings)
        })?;

        err.map_err(Into::into)
    }

    fn set_global_shortcut_error(&self, error: Option<String>) -> anyhow::Result<()> {
        self.repository.mutate_settings(|mut settings| {
            if let Some(data) = &mut settings.global_shortcut {
                data.error = error
            }
            Ok(settings)
        })?;

        Ok(())
    }

    pub fn global_entrypoint_shortcuts(
        &self,
    ) -> anyhow::Result<HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>> {
        let settings = self.repository.get_settings()?;
        let data: HashMap<_, _> = settings
            .global_entrypoint_shortcuts
            .unwrap_or_default()
            .into_iter()
            .map(|data| {
                let shortcut = data.shortcut.shortcut;
                let error = data.shortcut.error;

                let shortcut = PhysicalShortcut {
                    physical_key: PhysicalKey::from_value(shortcut.physical_key),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                };

                (
                    (
                        PluginId::from_string(data.plugin_id),
                        EntrypointId::from_string(data.entrypoint_id),
                    ),
                    (shortcut, error),
                )
            })
            .collect();

        Ok(data)
    }

    pub fn set_global_entrypoint_shortcut(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut: Option<PhysicalShortcut>,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        let current_global_hotkey = state
            .current_entrypoint_global_hotkeys
            .get(&(plugin_id.clone(), entrypoint_id.clone()))
            .cloned();

        let (hotkey, err) = self.assign_global_shortcut(global_hotkey_manager, current_global_hotkey, shortcut.clone());

        if let Some(hotkey) = hotkey {
            state
                .current_entrypoint_global_hotkeys
                .insert((plugin_id.clone(), entrypoint_id.clone()), hotkey);
        } else {
            state
                .current_entrypoint_global_hotkeys
                .remove(&(plugin_id.clone(), entrypoint_id.clone()));
        };

        match &err {
            Ok(()) => {
                tracing::info!(
                    "Successfully registered new global shortcut for plugin '{:?}' and entrypoint '{:?}' : {:?}",
                    plugin_id,
                    entrypoint_id,
                    shortcut
                );
            }
            Err(err) => {
                tracing::info!(
                    "Unable to register new global shortcut for plugin '{:?}' and entrypoint '{:?}' - {:?}: {:?}",
                    plugin_id,
                    entrypoint_id,
                    shortcut,
                    err
                );
            }
        }

        let db_err = err.as_ref().map_err(|err| format!("{:#}", err)).err();

        self.repository.mutate_settings(|mut settings| {
            let mut shortcuts: HashMap<_, _> = settings
                .global_entrypoint_shortcuts
                .unwrap_or_default()
                .into_iter()
                .map(|data| {
                    (
                        (
                            PluginId::from_string(data.plugin_id),
                            EntrypointId::from_string(data.entrypoint_id),
                        ),
                        data.shortcut,
                    )
                })
                .collect();

            match shortcut {
                None => {
                    shortcuts.remove(&(plugin_id, entrypoint_id));
                }
                Some(shortcut) => {
                    shortcuts.insert(
                        (plugin_id, entrypoint_id),
                        DbSettingsGlobalShortcutData {
                            shortcut: DbSettingsShortcut {
                                physical_key: shortcut.physical_key.to_value(),
                                modifier_shift: shortcut.modifier_shift,
                                modifier_control: shortcut.modifier_control,
                                modifier_alt: shortcut.modifier_alt,
                                modifier_meta: shortcut.modifier_meta,
                            },
                            error: db_err,
                        },
                    );
                }
            }

            let global_entrypoint_shortcuts = shortcuts
                .into_iter()
                .map(|((plugin_id, entrypoint_id), data)| {
                    DbSettingsGlobalEntrypointShortcutData {
                        plugin_id: plugin_id.to_string(),
                        entrypoint_id: entrypoint_id.to_string(),
                        shortcut: data,
                    }
                })
                .collect();

            settings.global_entrypoint_shortcuts = Some(global_entrypoint_shortcuts);

            Ok(settings)
        })?;

        Ok(())
    }

    fn set_global_entrypoint_shortcut_error(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        error: Option<String>,
    ) -> anyhow::Result<()> {
        self.repository.mutate_settings(|mut settings| {
            let mut shortcuts: HashMap<_, _> = settings
                .global_entrypoint_shortcuts
                .unwrap_or_default()
                .into_iter()
                .map(|data| {
                    (
                        (
                            PluginId::from_string(data.plugin_id),
                            EntrypointId::from_string(data.entrypoint_id),
                        ),
                        data.shortcut,
                    )
                })
                .collect();

            if let Some(data) = shortcuts.get_mut(&(plugin_id, entrypoint_id)) {
                data.error = error;
            };

            let global_entrypoint_shortcuts = shortcuts
                .into_iter()
                .map(|((plugin_id, entrypoint_id), data)| {
                    DbSettingsGlobalEntrypointShortcutData {
                        plugin_id: plugin_id.to_string(),
                        entrypoint_id: entrypoint_id.to_string(),
                        shortcut: data,
                    }
                })
                .collect();

            settings.global_entrypoint_shortcuts = Some(global_entrypoint_shortcuts);

            Ok(settings)
        })?;

        Ok(())
    }

    fn assign_global_shortcut(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        current_hotkey: Option<HotKey>,
        new_shortcut: Option<PhysicalShortcut>,
    ) -> (Option<HotKey>, anyhow::Result<()>) {
        if let Some(current_hotkey) = current_hotkey {
            if let Err(err) = global_hotkey_manager.unregister(current_hotkey.clone()) {
                tracing::warn!(
                    "error occurred when unregistering global shortcut {:?}: {:?}",
                    current_hotkey,
                    err
                )
            }
        }

        if let Some(new_shortcut) = new_shortcut {
            let hotkey = convert_physical_shortcut_to_hotkey(new_shortcut);
            match global_hotkey_manager.register(hotkey) {
                Ok(()) => (Some(hotkey), Ok(())),
                Err(err) => (None, Err(anyhow!(err))),
            }
        } else {
            (None, Ok(()))
        }
    }

    fn setup_shortcuts(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        global_shortcut: Option<PhysicalShortcut>,
    ) -> (Option<HotKey>, Option<String>) {
        let (current_global_hotkey, result) = self.assign_global_shortcut(global_hotkey_manager, None, global_shortcut);

        let result = result.map_err(|err| format!("{:#}", err)).err();

        (current_global_hotkey, result)
    }

    fn setup_entrypoint_shortcuts(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        global_entrypoint_shortcuts: HashMap<(PluginId, EntrypointId), PhysicalShortcut>,
    ) -> (
        HashMap<(PluginId, EntrypointId), HotKey>,
        HashMap<(PluginId, EntrypointId), Option<String>>,
    ) {
        let mut results = HashMap::new();
        let mut current_entrypoint_global_hotkeys = HashMap::new();

        for ((plugin_id, entrypoint_id), shortcut) in global_entrypoint_shortcuts {
            let (global_hotkey, result) = self.assign_global_shortcut(global_hotkey_manager, None, Some(shortcut));

            if let Some(global_hotkey) = global_hotkey {
                current_entrypoint_global_hotkeys.insert((plugin_id.clone(), entrypoint_id.clone()), global_hotkey);
            }

            let result = result.map_err(|err| format!("{:#}", err)).err();
            results.insert((plugin_id, entrypoint_id), result);
        }

        (current_entrypoint_global_hotkeys, results)
    }

    pub fn handle_global_shortcut_event(
        &self,
        GlobalShortcutPressedEvent(id): GlobalShortcutPressedEvent,
    ) -> anyhow::Result<GlobalShortcutAction> {
        let state = self.state.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        if let Some(hotkey) = state.current_global_hotkey {
            if hotkey.id == id {
                return Ok(GlobalShortcutAction::ToggleWindow);
            }
        }

        let ids = state
            .current_entrypoint_global_hotkeys
            .iter()
            .find(|(_, hotkey)| hotkey.id == id)
            .map(|(ids, _)| ids);

        if let Some((plugin_id, entrypoint_id)) = ids {
            return Ok(GlobalShortcutAction::RunEntrypoint {
                plugin_id: plugin_id.clone(),
                entrypoint_id: entrypoint_id.clone(),
            });
        };

        Ok(GlobalShortcutAction::Noop)
    }
}

pub enum GlobalShortcutAction {
    ToggleWindow,
    RunEntrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    Noop,
}

#[derive(Debug, Clone)]
pub struct GlobalShortcutPressedEvent(u32);

pub fn register_global_shortcut_listener<
    S: Sink<GlobalShortcutPressedEvent> + Unpin + Send + Sync + Debug + Clone + 'static,
>(
    pressed_events: S,
) where
    <S as Sink<GlobalShortcutPressedEvent>>::Error: Debug,
{
    let handle = Handle::current();

    global_hotkey::GlobalHotKeyEvent::set_event_handler(Some(move |e: global_hotkey::GlobalHotKeyEvent| {
        let mut pressed_events = pressed_events.clone();

        if let global_hotkey::HotKeyState::Pressed = e.state() {
            handle.spawn(async move {
                if let Err(err) = pressed_events.send(GlobalShortcutPressedEvent(e.id)).await {
                    tracing::warn!(target = "rpc", "error occurred when receiving shortcut event {:?}", err)
                }
            });
        }
    }));
}

fn convert_physical_shortcut_to_hotkey(shortcut: PhysicalShortcut) -> HotKey {
    let modifiers: Modifiers = {
        let mut modifiers = Modifiers::empty();

        if shortcut.modifier_alt {
            modifiers.insert(Modifiers::ALT);
        }

        if shortcut.modifier_control {
            modifiers.insert(Modifiers::CONTROL);
        }

        if shortcut.modifier_meta {
            modifiers.insert(Modifiers::META);
        }

        if shortcut.modifier_shift {
            modifiers.insert(Modifiers::SHIFT);
        }

        modifiers
    };

    let code = match shortcut.physical_key {
        PhysicalKey::Backquote => Code::Backquote,
        PhysicalKey::Backslash => Code::Backslash,
        PhysicalKey::BracketLeft => Code::BracketLeft,
        PhysicalKey::BracketRight => Code::BracketRight,
        PhysicalKey::Comma => Code::Comma,
        PhysicalKey::Digit1 => Code::Digit1,
        PhysicalKey::Digit2 => Code::Digit2,
        PhysicalKey::Digit3 => Code::Digit3,
        PhysicalKey::Digit4 => Code::Digit4,
        PhysicalKey::Digit5 => Code::Digit5,
        PhysicalKey::Digit6 => Code::Digit6,
        PhysicalKey::Digit7 => Code::Digit7,
        PhysicalKey::Digit8 => Code::Digit8,
        PhysicalKey::Digit9 => Code::Digit9,
        PhysicalKey::Digit0 => Code::Digit0,
        PhysicalKey::Equal => Code::Equal,
        PhysicalKey::IntlBackslash => Code::IntlBackslash,
        PhysicalKey::IntlRo => Code::IntlRo,
        PhysicalKey::IntlYen => Code::IntlYen,
        PhysicalKey::KeyA => Code::KeyA,
        PhysicalKey::KeyB => Code::KeyB,
        PhysicalKey::KeyC => Code::KeyC,
        PhysicalKey::KeyD => Code::KeyD,
        PhysicalKey::KeyE => Code::KeyE,
        PhysicalKey::KeyF => Code::KeyF,
        PhysicalKey::KeyG => Code::KeyG,
        PhysicalKey::KeyH => Code::KeyH,
        PhysicalKey::KeyI => Code::KeyI,
        PhysicalKey::KeyJ => Code::KeyJ,
        PhysicalKey::KeyK => Code::KeyK,
        PhysicalKey::KeyL => Code::KeyL,
        PhysicalKey::KeyM => Code::KeyM,
        PhysicalKey::KeyN => Code::KeyN,
        PhysicalKey::KeyO => Code::KeyO,
        PhysicalKey::KeyP => Code::KeyP,
        PhysicalKey::KeyQ => Code::KeyQ,
        PhysicalKey::KeyR => Code::KeyR,
        PhysicalKey::KeyS => Code::KeyS,
        PhysicalKey::KeyT => Code::KeyT,
        PhysicalKey::KeyU => Code::KeyU,
        PhysicalKey::KeyV => Code::KeyV,
        PhysicalKey::KeyW => Code::KeyW,
        PhysicalKey::KeyX => Code::KeyX,
        PhysicalKey::KeyY => Code::KeyY,
        PhysicalKey::KeyZ => Code::KeyZ,
        PhysicalKey::Minus => Code::Minus,
        PhysicalKey::Period => Code::Period,
        PhysicalKey::Quote => Code::Quote,
        PhysicalKey::Semicolon => Code::Semicolon,
        PhysicalKey::Slash => Code::Slash,
        PhysicalKey::Backspace => Code::Backspace,
        PhysicalKey::CapsLock => Code::CapsLock,
        PhysicalKey::ContextMenu => Code::ContextMenu,
        PhysicalKey::Enter => Code::Enter,
        PhysicalKey::Space => Code::Space,
        PhysicalKey::Tab => Code::Tab,
        PhysicalKey::Convert => Code::Convert,
        PhysicalKey::KanaMode => Code::KanaMode,
        PhysicalKey::Lang1 => Code::Lang1,
        PhysicalKey::Lang2 => Code::Lang2,
        PhysicalKey::Lang3 => Code::Lang3,
        PhysicalKey::Lang4 => Code::Lang4,
        PhysicalKey::Lang5 => Code::Lang5,
        PhysicalKey::NonConvert => Code::NonConvert,
        PhysicalKey::Delete => Code::Delete,
        PhysicalKey::End => Code::End,
        PhysicalKey::Help => Code::Help,
        PhysicalKey::Home => Code::Home,
        PhysicalKey::Insert => Code::Insert,
        PhysicalKey::PageDown => Code::PageDown,
        PhysicalKey::PageUp => Code::PageUp,
        PhysicalKey::ArrowDown => Code::ArrowDown,
        PhysicalKey::ArrowLeft => Code::ArrowLeft,
        PhysicalKey::ArrowRight => Code::ArrowRight,
        PhysicalKey::ArrowUp => Code::ArrowUp,
        PhysicalKey::NumLock => Code::NumLock,
        PhysicalKey::Numpad0 => Code::Numpad0,
        PhysicalKey::Numpad1 => Code::Numpad1,
        PhysicalKey::Numpad2 => Code::Numpad2,
        PhysicalKey::Numpad3 => Code::Numpad3,
        PhysicalKey::Numpad4 => Code::Numpad4,
        PhysicalKey::Numpad5 => Code::Numpad5,
        PhysicalKey::Numpad6 => Code::Numpad6,
        PhysicalKey::Numpad7 => Code::Numpad7,
        PhysicalKey::Numpad8 => Code::Numpad8,
        PhysicalKey::Numpad9 => Code::Numpad9,
        PhysicalKey::NumpadAdd => Code::NumpadAdd,
        PhysicalKey::NumpadBackspace => Code::NumpadBackspace,
        PhysicalKey::NumpadClear => Code::NumpadClear,
        PhysicalKey::NumpadClearEntry => Code::NumpadClearEntry,
        PhysicalKey::NumpadComma => Code::NumpadComma,
        PhysicalKey::NumpadDecimal => Code::NumpadDecimal,
        PhysicalKey::NumpadDivide => Code::NumpadDivide,
        PhysicalKey::NumpadEnter => Code::NumpadEnter,
        PhysicalKey::NumpadEqual => Code::NumpadEqual,
        PhysicalKey::NumpadHash => Code::NumpadHash,
        PhysicalKey::NumpadMemoryAdd => Code::NumpadMemoryAdd,
        PhysicalKey::NumpadMemoryClear => Code::NumpadMemoryClear,
        PhysicalKey::NumpadMemoryRecall => Code::NumpadMemoryRecall,
        PhysicalKey::NumpadMemoryStore => Code::NumpadMemoryStore,
        PhysicalKey::NumpadMemorySubtract => Code::NumpadMemorySubtract,
        PhysicalKey::NumpadMultiply => Code::NumpadMultiply,
        PhysicalKey::NumpadParenLeft => Code::NumpadParenLeft,
        PhysicalKey::NumpadParenRight => Code::NumpadParenRight,
        PhysicalKey::NumpadStar => Code::NumpadStar,
        PhysicalKey::NumpadSubtract => Code::NumpadSubtract,
        PhysicalKey::Escape => Code::Escape,
        PhysicalKey::Fn => Code::Fn,
        PhysicalKey::FnLock => Code::FnLock,
        PhysicalKey::PrintScreen => Code::PrintScreen,
        PhysicalKey::ScrollLock => Code::ScrollLock,
        PhysicalKey::Pause => Code::Pause,
        PhysicalKey::BrowserBack => Code::BrowserBack,
        PhysicalKey::BrowserFavorites => Code::BrowserFavorites,
        PhysicalKey::BrowserForward => Code::BrowserForward,
        PhysicalKey::BrowserHome => Code::BrowserHome,
        PhysicalKey::BrowserRefresh => Code::BrowserRefresh,
        PhysicalKey::BrowserSearch => Code::BrowserSearch,
        PhysicalKey::BrowserStop => Code::BrowserStop,
        PhysicalKey::Eject => Code::Eject,
        PhysicalKey::LaunchApp1 => Code::LaunchApp1,
        PhysicalKey::LaunchApp2 => Code::LaunchApp2,
        PhysicalKey::LaunchMail => Code::LaunchMail,
        PhysicalKey::MediaPlayPause => Code::MediaPlayPause,
        PhysicalKey::MediaSelect => Code::MediaSelect,
        PhysicalKey::MediaStop => Code::MediaStop,
        PhysicalKey::MediaTrackNext => Code::MediaTrackNext,
        PhysicalKey::MediaTrackPrevious => Code::MediaTrackPrevious,
        PhysicalKey::Power => Code::Power,
        PhysicalKey::Sleep => Code::Sleep,
        PhysicalKey::AudioVolumeDown => Code::AudioVolumeDown,
        PhysicalKey::AudioVolumeMute => Code::AudioVolumeMute,
        PhysicalKey::AudioVolumeUp => Code::AudioVolumeUp,
        PhysicalKey::WakeUp => Code::WakeUp,
        PhysicalKey::Abort => Code::Abort,
        PhysicalKey::Resume => Code::Resume,
        PhysicalKey::Suspend => Code::Suspend,
        PhysicalKey::Again => Code::Again,
        PhysicalKey::Copy => Code::Copy,
        PhysicalKey::Cut => Code::Cut,
        PhysicalKey::Find => Code::Find,
        PhysicalKey::Open => Code::Open,
        PhysicalKey::Paste => Code::Paste,
        PhysicalKey::Props => Code::Props,
        PhysicalKey::Select => Code::Select,
        PhysicalKey::Undo => Code::Undo,
        PhysicalKey::Hiragana => Code::Hiragana,
        PhysicalKey::Katakana => Code::Katakana,
        PhysicalKey::F1 => Code::F1,
        PhysicalKey::F2 => Code::F2,
        PhysicalKey::F3 => Code::F3,
        PhysicalKey::F4 => Code::F4,
        PhysicalKey::F5 => Code::F5,
        PhysicalKey::F6 => Code::F6,
        PhysicalKey::F7 => Code::F7,
        PhysicalKey::F8 => Code::F8,
        PhysicalKey::F9 => Code::F9,
        PhysicalKey::F10 => Code::F10,
        PhysicalKey::F11 => Code::F11,
        PhysicalKey::F12 => Code::F12,
        PhysicalKey::F13 => Code::F13,
        PhysicalKey::F14 => Code::F14,
        PhysicalKey::F15 => Code::F15,
        PhysicalKey::F16 => Code::F16,
        PhysicalKey::F17 => Code::F17,
        PhysicalKey::F18 => Code::F18,
        PhysicalKey::F19 => Code::F19,
        PhysicalKey::F20 => Code::F20,
        PhysicalKey::F21 => Code::F21,
        PhysicalKey::F22 => Code::F22,
        PhysicalKey::F23 => Code::F23,
        PhysicalKey::F24 => Code::F24,
        PhysicalKey::F25 => Code::F25,
        PhysicalKey::F26 => Code::F26,
        PhysicalKey::F27 => Code::F27,
        PhysicalKey::F28 => Code::F28,
        PhysicalKey::F29 => Code::F29,
        PhysicalKey::F30 => Code::F30,
        PhysicalKey::F31 => Code::F31,
        PhysicalKey::F32 => Code::F32,
        PhysicalKey::F33 => Code::F33,
        PhysicalKey::F34 => Code::F34,
        PhysicalKey::F35 => Code::F35,
    };

    HotKey::new(Some(modifiers), code)
}
