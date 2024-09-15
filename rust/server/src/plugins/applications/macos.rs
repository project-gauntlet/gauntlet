use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use cacao::filesystem::{FileManager, SearchPathDirectory, SearchPathDomainMask};
use cacao::url::Url;
use deno_runtime::deno_http::compressible::is_content_compressible;
use plist::Dictionary;
use serde::Deserialize;
use crate::plugins::applications::{DesktopEntry, resize_icon};

pub fn get_apps() -> Vec<DesktopEntry> {
    let file_manager = FileManager::default();

    let finder_application = vec![PathBuf::from("/System/Library/CoreServices/Finder.app")];
    let finder_applications = get_applications_in_dir(PathBuf::from("/System/Library/CoreServices/Finder.app/Contents/Applications"));

    let core_services_applications = get_applications_in_dir(PathBuf::from("/System/Library/CoreServices/Applications"));

    // these are covered by recursion
    // let user_admin_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::AdminApplications, SearchPathDomainMask::User);
    // let local_admin_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::AdminApplications, SearchPathDomainMask::Local);
    // let system_admin_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::AdminApplications, SearchPathDomainMask::Domain);

    let user_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::Applications, SearchPathDomainMask::User);
    let local_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::Applications, SearchPathDomainMask::Local);
    let system_applications_dir = get_applications_with_kind(&file_manager, SearchPathDirectory::Applications, SearchPathDomainMask::Domain);

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
    ].concat();

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

            DesktopEntry {
                name,
                icon: get_application_icon(&path, &info),
                command: vec!["open".to_string(), path.to_string_lossy().to_string()],
            }
        })
        .collect::<Vec<_>>();

    all_applications
}

fn get_applications_with_kind(file_manager: &FileManager, directory: SearchPathDirectory, mask: SearchPathDomainMask) -> Vec<PathBuf> {
    match file_manager.get_directory(directory.clone(), mask.clone()) {
        Ok(url) => {
            let applications_dir = url.to_file_path()
                .expect("returned application url is not a file path");

            tracing::debug!("reading {:?} {:?} directory: {:?}", directory, mask, &applications_dir);

            get_applications_in_dir(applications_dir)
        }
        Err(err) => {
            tracing::error!("error reading {:?} {:?} directory: {:?}", directory, mask, err);

            vec![]
        }
    }
}

fn get_applications_in_dir(path: PathBuf) -> Vec<PathBuf> {
    match path.read_dir() {
        Ok(read_dir) => {
            read_dir
                .collect::<std::io::Result<Vec<_>>>()
                .unwrap_or_default()
                .into_iter()
                .map(|entry| entry.path())
                .flat_map(|entry_path| {
                    if entry_path.is_dir() {
                        if entry_path.extension() == Some(OsStr::new("app")) {
                            vec![entry_path]
                        } else {
                            get_applications_in_dir(entry_path)
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

fn get_application_icon(app_path: &Path, info: &Option<Info>) -> Option<Vec<u8>> {
    if let Some(info) = info {
        info.bundle_icon_name
            .clone()
            .or(info.bundle_icon_file.clone())
            .map(|icon| {
                match PathBuf::from(&icon).extension() {
                    None => format!("{}.icns", icon),
                    Some(_) => icon
                }
            })
            .and_then(|icon_filename| {
                let icon_path = app_path
                    .join("Contents")
                    .join("Resources")
                    .join(icon_filename);

                tracing::debug!("Derived icon location for app {:?}: {:?}", &info.bundle_name, &icon_path);

                get_png_from_icon_path(icon_path)
            })
    } else {
        None
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