use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use cacao::filesystem::{FileManager, SearchPathDirectory, SearchPathDomainMask};
use cacao::url::Url;
use objc2::ClassType;
use objc2_app_kit::{NSBitmapImageRep, NSCalibratedWhiteColorSpace, NSCompositeCopy, NSDeviceRGBColorSpace, NSGraphicsContext, NSImage, NSPNGFileType, NSWorkspace};
use objc2_foundation::{CGFloat, CGPoint, CGRect, NSDictionary, NSInteger, NSPoint, NSRect, NSSize, NSString, NSZeroRect};
use plist::Dictionary;
use regex::Regex;
use serde::Deserialize;

use crate::plugins::applications::{DesktopEntry, resize_icon};

pub fn get_apps() -> Vec<DesktopEntry> {
    let file_manager = FileManager::default();

    let all_items = [
        get_applications(&file_manager),
        get_settings(&file_manager),
    ];

    all_items
        .into_iter()
        .flatten()
        .collect()
}

fn get_applications(file_manager: &FileManager) -> Vec<DesktopEntry> {

    let finder_application = vec![PathBuf::from("/System/Library/CoreServices/Finder.app")];
    let finder_applications = get_applications_in_dir(PathBuf::from("/System/Library/CoreServices/Finder.app/Contents/Applications"));

    let core_services_applications = get_applications_in_dir(PathBuf::from("/System/Library/CoreServices/Applications"));

    // these are covered by recursion on SearchPathDirectory::Applications
    // let user_admin_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::AdminApplications, SearchPathDomainMask::User);
    // let local_admin_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::AdminApplications, SearchPathDomainMask::Local);
    // let system_admin_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::AdminApplications, SearchPathDomainMask::Domain);

    let user_applications_dir = get_applications_with_kind(file_manager, SearchPathDirectory::Applications, SearchPathDomainMask::User);
    let local_applications_dir = get_applications_with_kind(file_manager, SearchPathDirectory::Applications, SearchPathDomainMask::Local);
    let system_applications_dir = get_applications_with_kind(file_manager, SearchPathDirectory::Applications, SearchPathDomainMask::Domain);

    let all_applications = [
        finder_application,
        finder_applications,
        core_services_applications,
        // user_admin_applications_dir,
        // local_admin_applications_dir,
        // system_admin_applications_dir,
        user_applications_dir,
        local_applications_dir,
        system_applications_dir
    ];

    let all_applications: Vec<_> = all_applications
        .into_iter()
        .flatten()
        .collect();

    tracing::debug!("Found following macOS applications: {:?}", all_applications);

    let all_applications = all_applications
        .into_iter()
        .map(|path| {
            let name = path.file_stem()
                .expect(&format!("invalid path: {:?}", path))
                .to_string_lossy()
                .to_string();

            let info_path = path.join("Contents").join("Info.plist");

            let info: Option<Info> = plist::from_file(info_path)
                .ok();

            let name = info.as_ref()
                .and_then(|info| info.bundle_display_name.clone().or_else(|| info.bundle_name.clone()))
                .unwrap_or(name);

            let icon = get_application_icon(&path)
                .inspect_err(|err| tracing::error!("error while reading application icon for {:?}: {:?}", path, err))
                .ok();

            DesktopEntry {
                name,
                icon,
                command: vec!["open".to_string(), path.to_string_lossy().to_string()],
            }
        })
        .collect::<Vec<_>>();

    all_applications
}

fn get_settings(file_manager: &FileManager) -> Vec<DesktopEntry> {
    let system_version: SystemVersion = plist::from_file("/System/Library/CoreServices/SystemVersion.plist")
        .expect("SystemVersion.plist doesn't follow expected format");

    let regex = Regex::new(r"^(?<major>\d+).\d+(.\d+)?$")
        .expect("This regex cannot be invalid");

    let captures = regex.captures(&system_version.product_version)
        .expect("SystemVersion.plist ProductVersion doesn't match expected format");

    let major_version: u8 = captures["major"]
        .parse()
        .expect("SystemVersion.plist ProductVersion major doesn't match expected format");

    tracing::debug!("Indexing settings for macOS version: {}", major_version);

    if major_version >= 13 {  // Ventura and higher
        let sidebar: Vec<SidebarSection> = plist::from_file("/System/Applications/System Settings.app/Contents/Resources/Sidebar.plist")
            .expect("Sidebar.plist doesn't follow expected format");

        let preferences_ids: Vec<_> = sidebar.into_iter()
            .flat_map(|section| match section {
                SidebarSection::Content { content } => content,
                SidebarSection::Title { .. } => vec![]
            })
            .collect();

        tracing::debug!("Found following macOS setting preference ids: {:?}", &preferences_ids);

        let extensions: HashMap<_, _> = get_extensions_in_dir(PathBuf::from("/System/Library/ExtensionKit/Extensions"))
            .into_iter()
            .filter_map(|path| {
                fn read_plist(path: &Path) -> anyhow::Result<(String, (String, PathBuf))> {
                    let name = path.file_stem()
                        .expect(&format!("invalid path: {:?}", path))
                        .to_string_lossy()
                        .to_string();

                    let info_path = path.join("Contents").join("Info.plist");

                    let info = plist::from_file::<_, Info>(info_path.as_path())
                        .context(format!("Unexpected Info.plist for System Extensions: {}", &info_path.display()))?;

                    let name = info.bundle_display_name
                        .clone()
                        .or_else(|| info.bundle_name.clone())
                        .unwrap_or(name);

                    Ok((info.bundle_id, (name, path.to_path_buf())))
                }

                read_plist(&path)
                    .inspect_err(|err| tracing::error!("error while reading system extension Info.plist {:?}: {:?}", path, err))
                    .ok()
            })
            .collect();

        tracing::debug!("Found following macOS setting extensions: {:?}", &extensions);

        preferences_ids.into_iter()
            .filter_map(|preferences_id| {
                match extensions.get(&preferences_id) {
                    None => {
                        // todo some settings panel items return none here
                        tracing::debug!("Unknown preference id found: {}", &preferences_id);

                        None
                    }
                    Some((name, path)) => {
                        let icon = get_application_icon(&path)
                            .inspect_err(|err| tracing::error!("error while reading application icon for {:?}: {:?}", path, err))
                            .ok();

                        Some(
                            DesktopEntry {
                                name: name.to_string(),
                                icon,
                                command: vec![
                                    "open".to_string(),
                                    format!("x-apple.systempreferences:{}", preferences_id)
                                ],
                            }
                        )
                    }
                }
            })
            .collect()
    } else {
        let user_pref_panes_dir = get_pref_panes_with_kind(file_manager, SearchPathDirectory::Library, SearchPathDomainMask::User);
        let local_pref_panes_dir = get_pref_panes_with_kind(file_manager, SearchPathDirectory::Library, SearchPathDomainMask::Local);
        let system_pref_panes_dir = get_pref_panes_with_kind(file_manager, SearchPathDirectory::Library, SearchPathDomainMask::Domain);

        let all_settings = [
            user_pref_panes_dir,
            local_pref_panes_dir,
            system_pref_panes_dir,
        ];

        let all_settings: Vec<_> = all_settings
            .into_iter()
            .flatten()
            .collect();

        tracing::debug!("Found following macOS settings: {:?}", all_settings);

        let all_settings = all_settings.into_iter()
            .map(|path| {
                let name = path.file_stem() // TODO is there a proper way?
                    .expect(&format!("invalid path: {:?}", path))
                    .to_string_lossy()
                    .to_string();

                DesktopEntry {
                    name,
                    icon: None,
                    command: vec![
                        "open".to_string(),
                        "-b".to_string(),
                        "com.apple.systempreferences".to_string(),
                        path.to_string_lossy().to_string()
                    ],
                }
            })
            .collect();

        all_settings
    }
}

fn get_pref_panes_with_kind(file_manager: &FileManager, directory: SearchPathDirectory, mask: SearchPathDomainMask) -> Vec<PathBuf> {
    get_items_with_kind(file_manager, directory, mask, Some("PreferencePanes"), |dir| get_pref_panes_in_dir(dir))
}

fn get_applications_with_kind(file_manager: &FileManager, directory: SearchPathDirectory, mask: SearchPathDomainMask) -> Vec<PathBuf> {
    get_items_with_kind(file_manager, directory, mask, None, |dir| get_applications_in_dir(dir))
}

fn get_items_with_kind<F>(
    file_manager: &FileManager,
    directory: SearchPathDirectory,
    mask: SearchPathDomainMask,
    suffix: Option<&'static str>,
    read_fn: F
) -> Vec<PathBuf> where F: Fn(PathBuf) -> Vec<PathBuf>
{
    match file_manager.get_directory(directory.clone(), mask.clone()) {
        Ok(url) => {
            let applications_dir = url.to_file_path()
                .expect("returned application url is not a file path");

            let applications_dir = match suffix {
                Some(suffix) => applications_dir.join(suffix),
                None => applications_dir
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
        Err(_) => vec![]
    }
}

// from https://stackoverflow.com/a/38442746 and https://stackoverflow.com/a/29162536
unsafe fn resize_ns_image(source_image: &NSImage, width: NSInteger, height: NSInteger) -> Option<Vec<u8>> {
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

    Some(data.bytes().to_vec())
}

fn get_application_icon(app_path: &Path) -> anyhow::Result<Vec<u8>> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();

        let app_path = app_path.to_str()
            .context(format!("Application path is not a utf-8 string: {:?}", &app_path))?;

        let app_path = NSString::from_str(app_path);

        let image = workspace.iconForFile(&app_path);

        let bytes = resize_ns_image(&image, 40, 40)
            .ok_or(anyhow!("Unable to resize the image"))?;

        Ok(bytes)
    }
}

fn get_png_from_icon_path(icon_path: PathBuf) -> Option<Vec<u8>> {
    let icon_file = std::fs::File::open(icon_path.clone())
        .inspect_err(|err| tracing::debug!("error while opening icns {:?}: {:?}", icon_path, err))
        .ok()?;

    let icon_family = icns::IconFamily::read(icon_file)
        .inspect_err(|err| tracing::debug!("error while reading icns {:?}: {:?}", icon_path, err))
        .ok()?;

    let bytes = icon_family.available_icons()
        .into_iter()
        .max_by_key(|icon_type| icon_type.screen_width())
        .and_then(|icon_type| {
            icon_family.get_icon_with_type(icon_type)
                .inspect_err(|err| tracing::debug!("error while extracting image from icns {:?}: {:?}", icon_path, err))
                .ok()
        })
        .and_then(|image| {
            let mut buffer = vec![];

            let result = image.write_png(&mut buffer);

            match result {
                Ok(_) => {
                    resize_icon(buffer)
                        .inspect_err(|err| tracing::debug!("error while resizing image {:?}: {:?}", icon_path, err))
                        .ok()
                },
                Err(_) => None,
            }
        });

    bytes
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
    bundle_icon_file: Option<String>,
    #[serde(rename = "CFBundleIconName")]
    bundle_icon_name: Option<String>,
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
        content: Vec<String>
    },
    Title {
        title: String
    }
}