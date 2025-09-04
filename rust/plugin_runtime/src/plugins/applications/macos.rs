use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;
use anyhow::anyhow;
use cacao::core_graphics::base::CGFloat;
use cacao::filesystem::FileManager;
use cacao::filesystem::SearchPathDirectory;
use cacao::filesystem::SearchPathDomainMask;
use deno_core::OpState;
use deno_core::ToJsBuffer;
use deno_core::op2;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApi;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;
use gauntlet_common_plugin_runtime::model::JsMacosApplicationEvent;
use objc2::AnyThread;
use objc2_app_kit::NSBitmapImageRep;
use objc2_app_kit::NSCompositeCopy;
use objc2_app_kit::NSDeviceRGBColorSpace;
use objc2_app_kit::NSGraphicsContext;
use objc2_app_kit::NSImage;
use objc2_app_kit::NSPNGFileType;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSDictionary;
use objc2_foundation::NSInteger;
use objc2_foundation::NSPoint;
use objc2_foundation::NSRect;
use objc2_foundation::NSSize;
use objc2_foundation::NSString;
use objc2_foundation::NSZeroRect;
use regex::Regex;
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

use crate::deno::GauntletJsError;
use crate::plugins::applications::ApplicationContext;
use crate::plugins::applications::DesktopApplication;
use crate::plugins::applications::DesktopEnvironment;
use crate::plugins::applications::DesktopPathAction;
use crate::plugins::applications::DesktopSettings13AndPostData;
use crate::plugins::applications::DesktopSettingsPre13Data;

pub fn macos_major_version() -> u8 {
    let system_version: SystemVersion = plist::from_file("/System/Library/CoreServices/SystemVersion.plist")
        .expect("SystemVersion.plist doesn't follow expected format");

    let regex = Regex::new(r"^(?<major>\d+).\d+(.\d+)?$").expect("This regex cannot be invalid");

    let captures = regex
        .captures(&system_version.product_version)
        .expect("SystemVersion.plist ProductVersion doesn't match expected format");

    let major_version: u8 = captures["major"]
        .parse()
        .expect("SystemVersion.plist ProductVersion major doesn't match expected format");

    tracing::debug!("macOS version: {}", major_version);

    major_version
}

pub fn macos_system_applications() -> Vec<PathBuf> {
    let finder_application = vec![PathBuf::from("/System/Library/CoreServices/Finder.app")];
    let finder_applications = get_applications_in_dir(PathBuf::from(
        "/System/Library/CoreServices/Finder.app/Contents/Applications",
    ));

    let core_services_applications =
        get_applications_in_dir(PathBuf::from("/System/Library/CoreServices/Applications"));

    let all_applications = [finder_application, finder_applications, core_services_applications];

    let all_applications: Vec<_> = all_applications.into_iter().flatten().collect();

    all_applications
}

pub fn macos_application_dirs() -> Vec<PathBuf> {
    let file_manager = FileManager::default();

    let user_applications_dir = get_path(
        &file_manager,
        SearchPathDirectory::Applications,
        SearchPathDomainMask::User,
    );
    let local_applications_dir = get_path(
        &file_manager,
        SearchPathDirectory::Applications,
        SearchPathDomainMask::Local,
    );
    let system_applications_dir = get_path(
        &file_manager,
        SearchPathDirectory::Applications,
        SearchPathDomainMask::Domain,
    );

    let all_applications = [user_applications_dir, local_applications_dir, system_applications_dir];

    let all_applications: Vec<_> = all_applications.into_iter().flatten().collect();

    all_applications
}

pub fn macos_app_from_arbitrary_path(path: PathBuf, lang: Option<String>) -> Option<DesktopPathAction> {
    let path = path
        .ancestors()
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .find_map(|path| {
            let Some(extension) = path.extension() else {
                return None;
            };

            let Some("app") = extension.to_str() else {
                return None;
            };

            Some(path)
        });

    let Some(path) = path else {
        return None;
    };

    macos_app_from_path(path, lang)
}

fn get_bundle_name(app_path: &Path) -> String {
    let info_path = app_path.join("Contents").join("Info.plist");

    let info: Option<Info> = plist::from_file(info_path).ok();

    let fallback_name = app_path
        .file_stem()
        .expect(&format!("invalid path: {:?}", app_path))
        .to_str()
        .expect("non-uft8 paths are not supported")
        .to_string();

    let mut bundle_name = info
        .as_ref()
        .and_then(|info| info.bundle_display_name.clone().or_else(|| info.bundle_name.clone()))
        .unwrap_or(fallback_name.clone());

    if bundle_name.is_empty() {
        bundle_name = fallback_name;
    }

    bundle_name
}

fn get_localized_name(path: &Path, preferred_language: &str) -> Option<String> {
    let localized_info: Option<InfoPlist> = plist::from_file(path).ok();

    if let Some(localized_info) = localized_info {
        // get language info. first try to use display name, if not available use name
        localized_info
            .languages
            .get(preferred_language)
            .and_then(|localized_info| {
                localized_info
                    .bundle_display_name
                    .clone()
                    .or_else(|| localized_info.bundle_name.clone())
            })
    } else {
        eprintln!("Error: Could not load plist from path '{}'.", path.display());
        None
    }
}

pub fn macos_app_from_path(path: &Path, lang: Option<String>) -> Option<DesktopPathAction> {
    if !path.is_dir() {
        return None;
    }

    let name = lang
        .and_then(|l| {
            let info_plist_path = path.join("Contents/Resources/InfoPlist.loctable");
            if info_plist_path.is_file() {
                get_localized_name(info_plist_path.as_path(), &l)
            } else {
                None
            }
        })
        .unwrap_or(get_bundle_name(path));

    let icon = get_application_icon(&path)
        .inspect_err(|err| tracing::error!("error while reading application icon for {:?}: {:?}", path, err))
        .ok();

    let path = path.to_str().expect("non-uft8 paths are not supported").to_string();

    Some(DesktopPathAction::Add {
        id: path.clone(),
        data: DesktopApplication { name, path, icon },
    })
}

pub fn macos_settings_pre_13() -> Vec<DesktopSettingsPre13Data> {
    let file_manager = FileManager::default();

    let user_pref_panes_dir =
        get_pref_panes_with_kind(&file_manager, SearchPathDirectory::Library, SearchPathDomainMask::User);
    let local_pref_panes_dir =
        get_pref_panes_with_kind(&file_manager, SearchPathDirectory::Library, SearchPathDomainMask::Local);
    let system_pref_panes_dir = get_pref_panes_with_kind(
        &file_manager,
        SearchPathDirectory::Library,
        SearchPathDomainMask::Domain,
    );

    let all_settings = [user_pref_panes_dir, local_pref_panes_dir, system_pref_panes_dir];

    let all_settings: Vec<_> = all_settings.into_iter().flatten().collect();

    tracing::debug!("Found following macOS settings: {:?}", all_settings);

    let all_settings = all_settings
        .into_iter()
        .map(|path| {
            let name = path
                .file_stem() // TODO is there a proper way got get the name?
                .expect(&format!("invalid path: {:?}", &path))
                .to_string_lossy()
                .to_string();

            DesktopSettingsPre13Data {
                name,
                path: path.to_str().expect("non-uft8 paths are not supported").to_string(),
                icon: None,
            }
        })
        .collect();

    all_settings
}

pub fn macos_settings_13_and_post(lang: Option<String>) -> Vec<DesktopSettings13AndPostData> {
    let sidebar: Vec<SidebarSection> =
        plist::from_file("/System/Applications/System Settings.app/Contents/Resources/Sidebar.plist")
            .expect("Sidebar.plist doesn't follow expected format");

    let preferences_ids: Vec<_> = sidebar
        .into_iter()
        .flat_map(|section| {
            match section {
                SidebarSection::Content { content } => content,
                SidebarSection::Title { .. } => vec![],
            }
        })
        .collect();

    tracing::debug!("Found following macOS setting preference ids: {:?}", &preferences_ids);

    let extensions: HashMap<_, _> = get_extensions_in_dir(PathBuf::from("/System/Library/ExtensionKit/Extensions"))
        .into_iter()
        .filter_map(|path| {
            fn read_plist(path: &Path, lang: &Option<String>) -> anyhow::Result<(String, (String, PathBuf))> {
                let mut name = path
                    .file_stem()
                    .expect(&format!("invalid path: {:?}", path))
                    .to_string_lossy()
                    .to_string();

                let localized_info_path = path.join("Contents/Resources/InfoPlist.loctable");
                if !localized_info_path.is_file() {
                    return Ok((name.clone(), (name, path.to_path_buf())));
                }

                if let Some(lang) = lang {
                    name = get_localized_name(localized_info_path.as_path(), lang).unwrap_or(name);
                } else {
                    name = get_localized_name(localized_info_path.as_path(), "en").unwrap_or(name);
                }

                let info_path = path.join("Contents").join("Info.plist");

                let info = plist::from_file::<_, Info>(info_path.as_path()).context(format!(
                    "Unexpected Info.plist for System Extensions: {}",
                    &info_path.display()
                ))?;

                Ok((info.bundle_id, (name, path.to_path_buf())))
            }

            read_plist(&path, &lang)
                .inspect_err(|err| {
                    tracing::error!("error while reading system extension Info.plist {:?}: {:?}", path, err)
                })
                .ok()
        })
        .collect();

    tracing::debug!("Found following macOS setting extensions: {:?}", &extensions);

    preferences_ids
        .into_iter()
        .filter_map(|preferences_id| {
            match extensions.get(&preferences_id) {
                None => {
                    // todo some settings panel items return none here
                    tracing::debug!("Unknown preference id found: {}", &preferences_id);

                    None
                }
                Some((name, path)) => {
                    let icon = get_application_icon(&path)
                        .inspect_err(|err| {
                            tracing::error!("error while reading application icon for {:?}: {:?}", path, err)
                        })
                        .ok();

                    Some(DesktopSettings13AndPostData {
                        name: name.to_string(),
                        preferences_id,
                        icon,
                    })
                }
            }
        })
        .collect()
}

fn get_pref_panes_with_kind(
    file_manager: &FileManager,
    directory: SearchPathDirectory,
    mask: SearchPathDomainMask,
) -> Vec<PathBuf> {
    get_items_with_kind(file_manager, directory, mask, Some("PreferencePanes"), |dir| {
        get_pref_panes_in_dir(dir)
    })
}

#[allow(unused)]
fn get_applications_with_kind(
    file_manager: &FileManager,
    directory: SearchPathDirectory,
    mask: SearchPathDomainMask,
) -> Vec<PathBuf> {
    get_items_with_kind(file_manager, directory, mask, None, |dir| get_applications_in_dir(dir))
}

fn get_items_with_kind<F>(
    file_manager: &FileManager,
    directory: SearchPathDirectory,
    mask: SearchPathDomainMask,
    suffix: Option<&'static str>,
    read_fn: F,
) -> Vec<PathBuf>
where
    F: Fn(PathBuf) -> Vec<PathBuf>,
{
    match file_manager.get_directory(directory.clone(), mask.clone()) {
        Ok(url) => {
            let applications_dir = url.to_file_path().expect("returned application url is not a file path");

            let applications_dir = match suffix {
                Some(suffix) => applications_dir.join(suffix),
                None => applications_dir,
            };

            tracing::debug!("reading {:?} {:?} directory: {:?}", directory, mask, &applications_dir);

            read_fn(applications_dir)
        }
        Err(err) => {
            tracing::error!("error reading {:?} {:?} directory: {:?}", directory, mask, err);

            vec![]
        }
    }
}

fn get_path(file_manager: &FileManager, directory: SearchPathDirectory, mask: SearchPathDomainMask) -> Option<PathBuf> {
    match file_manager.get_directory(directory.clone(), mask.clone()) {
        Ok(url) => {
            let applications_dir = url.to_file_path().expect("returned application url is not a file path");

            Some(applications_dir)
        }
        Err(err) => {
            tracing::error!("error reading {:?} {:?} directory: {:?}", directory, mask, err);

            None
        }
    }
}

fn get_pref_panes_in_dir(path: PathBuf) -> Vec<PathBuf> {
    get_items_in_dir(path, "prefPane")
}

fn get_applications_in_dir(path: PathBuf) -> Vec<PathBuf> {
    get_items_in_dir(path, "app")
}

fn get_extensions_in_dir(path: PathBuf) -> Vec<PathBuf> {
    get_items_in_dir(path, "appex")
}

fn get_items_in_dir(path: PathBuf, extension: &str) -> Vec<PathBuf> {
    match path.read_dir() {
        Ok(read_dir) => {
            read_dir
                .collect::<std::io::Result<Vec<_>>>()
                .unwrap_or_default()
                .into_iter()
                .map(|entry| entry.path())
                .flat_map(|entry_path| {
                    if entry_path.is_dir() {
                        if entry_path.extension() == Some(OsStr::new(extension)) {
                            vec![entry_path]
                        } else {
                            get_items_in_dir(entry_path, extension)
                        }
                    } else {
                        vec![]
                    }
                })
                .collect::<Vec<_>>()
        }
        Err(_) => vec![],
    }
}

// from https://stackoverflow.com/a/38442746 and https://stackoverflow.com/a/29162536
unsafe fn resize_ns_image(source_image: &NSImage, width: NSInteger, height: NSInteger) -> Option<Vec<u8>> {
    unsafe {
        let new_size = NSSize::new(width as CGFloat, height as CGFloat);

        let bitmap_image_rep = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut::<*mut _>(),
            width,
            height,
            8,
            4,
            true,
            false,
            NSDeviceRGBColorSpace,
            0,
            0,
        )?;

        bitmap_image_rep.setSize(new_size);

        NSGraphicsContext::saveGraphicsState_class();

        let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&bitmap_image_rep)
            .expect("should be present because just saved");

        NSGraphicsContext::setCurrentContext(Some(&context));

        let rect = NSRect::new(NSPoint::new(0.0, 0.0), new_size);
        source_image.drawInRect_fromRect_operation_fraction(rect, NSZeroRect, NSCompositeCopy, 1.0);

        NSGraphicsContext::restoreGraphicsState_class();

        // TODO i guess this doesn't work for 2x image

        let data = bitmap_image_rep.representationUsingType_properties(NSPNGFileType, &NSDictionary::dictionary())?;

        Some(data.to_vec())
    }
}

fn get_application_icon(app_path: &Path) -> anyhow::Result<ToJsBuffer> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();

        let app_path = app_path
            .to_str()
            .context(format!("Application path is not a utf-8 string: {:?}", &app_path))?;

        let app_path = NSString::from_str(app_path);

        let image = workspace.iconForFile(&app_path);

        let bytes = resize_ns_image(&image, 40, 40).ok_or(anyhow!("Unable to resize the image"))?;

        Ok(bytes.into())
    }
}

#[derive(Deserialize)]
struct Info {
    #[serde(rename = "CFBundleIdentifier")]
    bundle_id: String,

    #[serde(rename = "CFBundleDisplayName")]
    bundle_display_name: Option<String>,
    #[serde(rename = "CFBundleName")]
    bundle_name: Option<String>,

    // macOS only icon fields
    #[serde(rename = "CFBundleIconFile")]
    #[allow(unused)]
    bundle_icon_file: Option<String>,
    #[serde(rename = "CFBundleIconName")]
    #[allow(unused)]
    bundle_icon_name: Option<String>,
}

#[derive(Deserialize)]
struct LocalizedInfo {
    #[serde(rename = "CFBundleDisplayName")]
    bundle_display_name: Option<String>,
    #[serde(rename = "CFBundleName")]
    bundle_name: Option<String>,
}

#[derive(Deserialize)]
struct InfoPlist {
    #[serde(flatten)]
    languages: HashMap<String, LocalizedInfo>,
}

#[derive(Deserialize)]
struct SystemVersion {
    #[serde(rename = "ProductVersion")]
    product_version: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SidebarSection {
    Content {
        content: Vec<String>,
    },
    Title {
        #[allow(unused)]
        title: String,
    },
}

pub struct MacosDesktopEnvironment {
    event_receiver: Rc<RefCell<Receiver<JsMacosApplicationEvent>>>,
    sender: Sender<JsMacosApplicationEvent>,
}

impl MacosDesktopEnvironment {
    pub fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = channel(10);

        Ok(Self {
            event_receiver: Rc::new(RefCell::new(receiver)),
            sender,
        })
    }
}

#[op2(async)]
pub async fn application_macos_receive_event(
    state: Rc<RefCell<OpState>>,
    #[serde] event: JsMacosApplicationEvent,
) -> Result<(), GauntletJsError> {
    tracing::debug!("Received macos application event from main app {:?}", event);

    let state = state.borrow();

    let context = state.borrow::<ApplicationContext>();

    match &context.desktop {
        DesktopEnvironment::Macos(env) => {
            env.sender
                .send(event)
                .await
                .context("Failed to send event back to plugin runtime")?;
        }
        _ => Err(anyhow!("Calling application_macos_receive_event on non-macos platform"))?,
    }

    Ok(())
}

#[op2(async)]
#[serde]
pub async fn application_macos_pending_event(
    state: Rc<RefCell<OpState>>,
) -> Result<JsMacosApplicationEvent, GauntletJsError> {
    let receiver = {
        let state = state.borrow();

        let context = state.borrow::<ApplicationContext>();

        match &context.desktop {
            DesktopEnvironment::Macos(env) => env.event_receiver.clone(),
            _ => Err(anyhow!("Calling application_macos_pending_event on non-macos platform"))?,
        }
    };

    let mut receiver = receiver.borrow_mut();
    let event = receiver
        .recv()
        .await
        .ok_or_else(|| anyhow!("plugin event stream was suddenly closed"))?;

    tracing::trace!("Received macos application event {:?}", event);

    Ok(event)
}

#[op2(async)]
pub async fn macos_focus_window(
    state: Rc<RefCell<OpState>>,
    #[string] window_uuid: String,
) -> Result<(), GauntletJsError> {
    let api = {
        let state = state.borrow();

        let api = state.borrow::<BackendForPluginRuntimeApiProxy>().clone();

        api
    };

    api.window_tracking_macos_focus_window(window_uuid)
        .await
        .map_err(Into::into)
}
