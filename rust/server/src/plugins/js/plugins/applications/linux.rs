use std::collections::{HashMap, HashSet};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::plugins::js::plugins::applications::{resize_icon, DesktopApplication, DesktopPathAction};
use freedesktop_entry_parser::parse_entry;
use freedesktop_icons::lookup;
use image::imageops::FilterType;
use image::ImageFormat;
use walkdir::WalkDir;

pub fn linux_application_dirs(home_dir: PathBuf) -> Vec<PathBuf> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(val) => {
            PathBuf::from(val)
        },
        None => {
            home_dir
                .join(".local")
                .join("share")
        }
    };

    let mut extra_data_dirs = match env::var_os("XDG_DATA_DIRS") {
        Some(val) => {
            env::split_paths(&val).map(PathBuf::from).collect()
        },
        None => {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
                PathBuf::from("/var/lib/flatpak/exports/share"),
            ]
        }
    };

    let flatpak = data_home.to_path_buf()
        .join("flatpak")
        .join("exports")
        .join("share");

    let mut res = Vec::new();
    res.push(data_home);
    res.push(flatpak);
    res.append(&mut extra_data_dirs);

    res.into_iter()
        .map(|d| d.join("applications"))
        .collect()
}

pub fn linux_app_from_path(home_dir: PathBuf, path: PathBuf) -> Option<DesktopPathAction> {
    let app_directories = linux_application_dirs(home_dir);

    let relative_to_app_dir = app_directories
        .into_iter()
        .find_map(|app_path| path.strip_prefix(app_path).ok());

    let Some(relative_to_app_dir) = relative_to_app_dir else {
        return None;
    };

    let Some(relative_to_app_dir) = relative_to_app_dir.to_str() else {
        return None;
    };

    let Some(extension) = path.extension() else {
        return None;
    };

    let Some("desktop") = extension.to_str() else {
        return None;
    };

    let desktop_file_id = relative_to_app_dir.replace("/", "-");

    if !path.exists() {
        tracing::debug!("Removing application at: {:?}", path);
        Some(DesktopPathAction::Remove { id: desktop_file_id })
    } else {
        // follows symlinks needed for flatpak
        let Ok(metadata) = fs::metadata(&path) else {
            return None;
        };

        if !metadata.is_file() {
            return None;
        }

        if let Some(entry) = create_app_entry(&path) {
            tracing::debug!("Adding application at: {:?}", path);

            Some(DesktopPathAction::Add {
                id: desktop_file_id,
                data: entry
            })
        } else {
            None
        }
    }
}

fn create_app_entry(desktop_file_path: &Path) -> Option<DesktopApplication> {
    let entry = parse_entry(desktop_file_path)
        .inspect_err(|err| tracing::warn!("error parsing .desktop file at path {:?}: {:?}", desktop_file_path, err))
        .ok()?;

    let entry = entry.section("Desktop Entry");

    let name = entry.attr("Name")?;
    let icon = entry.attr("Icon").map(|s| s.to_string());
    let no_display = entry.attr("NoDisplay").map(|val| val == "true").unwrap_or(false);
    let hidden = entry.attr("Hidden").map(|val| val == "true").unwrap_or(false);
    // TODO NotShowIn, OnlyShowIn https://wiki.archlinux.org/title/desktop_entries

    if no_display || hidden {
        return None
    }

    let icon = icon
        .map(|icon| {
            let icon_path = PathBuf::from(&icon);
            if icon_path.is_absolute() {
                Some(icon_path)
            } else {
                lookup(&icon)
                    .with_size(48)
                    .find()
            }
        })
        .flatten()
        .inspect(|path| tracing::debug!("icon path: {:?}", path))
        .map(|path| {
            match path.extension() {
                None => Err(anyhow::anyhow!("unknown format")),
                Some(extension) => {
                    match extension.to_str() {
                        Some("png") => {
                            let data = std::fs::read(path)?;

                            resize_icon(data)
                        },
                        Some("svg") => {
                            let data = std::fs::read(path)?;

                            let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())?;

                            let pixmap_size = tree.size().to_int_size();
                            let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

                            resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

                            let data = pixmap.encode_png()?;

                            let data = resize_icon(data)?;

                            Ok(data)
                        },
                        Some("xpm") => Err(anyhow::anyhow!("xpm format")),
                        _ => Err(anyhow::anyhow!("unsupported by spec format {:?}", extension)),
                    }
                }
            }
        })
        .map(|res| {
            res
                .inspect_err(|err| tracing::warn!("error processing icon of {:?}: {:?}", desktop_file_path, err))
                .ok()
        })
        .flatten();

    Some(DesktopApplication {
        name: name.to_string(),
        icon,
    })
}
