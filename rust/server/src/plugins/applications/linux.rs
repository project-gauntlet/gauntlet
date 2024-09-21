use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::env;
use std::path::{PathBuf};

use freedesktop_entry_parser::parse_entry;
use freedesktop_icons::lookup;
use image::ImageFormat;
use image::imageops::FilterType;
use serde::Serialize;
use walkdir::WalkDir;
use common::dirs::Dirs;
use crate::plugins::applications::{DesktopEntry, resize_icon};

fn find_application_dirs() -> Option<Vec<PathBuf>> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(val) => {
            PathBuf::from(val)
        },
        None => {
            let dirs = Dirs::new();

            dirs.home_dir()
                .join(".local")
                .join("share")
        }
    };
    let extra_data_dirs = match env::var_os("XDG_DATA_DIRS") {
        Some(val) => {
            env::split_paths(&val).map(PathBuf::from).collect()
        },
        None => {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share")
            ]
        }
    };

    let mut res = Vec::new();
    res.push(data_home.join("applications"));
    for dir in extra_data_dirs {
        res.push(dir.join("applications"));
    }
    Some(res)
}

pub fn get_apps() -> Vec<DesktopEntry> {
    let app_dirs = find_application_dirs()
        .unwrap_or_default()
        .into_iter()
        .filter(|dir| dir.exists())
        .collect::<Vec<_>>();

    let mut result: HashMap<String, DesktopEntry> = HashMap::new();

    for app_dir in app_dirs {
        let found_desktop_entries = WalkDir::new(app_dir.clone())
            .into_iter()
            .filter_map(|dir_entry| dir_entry.ok())
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .filter_map(|path| {
                let path = path.path();

                tracing::debug!("path: {:?}", path);

                match path.extension() {
                    None => None,
                    Some(extension) => {
                        match extension.to_str() {
                            Some("desktop") => {

                                let desktop_id = path.strip_prefix(&app_dir)
                                    .ok()?
                                    .to_str()?
                                    .to_owned();

                                let entry = create_app_entry(path.to_path_buf())?;

                                Some((desktop_id, entry))
                            },
                            _ => None,
                        }
                    }
                }
            })
            .collect::<HashMap<_, _>>();

        for (path, desktop_entry) in found_desktop_entries {
            if let Vacant(entry) = result.entry(path) {
                entry.insert(desktop_entry);
            }
        }
    }

    result.into_values().collect()
}

fn create_app_entry(path: PathBuf) -> Option<DesktopEntry> {
    let desktop_filename = path.file_name()
        .expect("desktop file doesn't have filename")
        .to_os_string()
        .into_string()
        .ok()?;

    let entry = parse_entry(&path)
        .inspect_err(|err| tracing::warn!("error parsing .desktop file at path {:?}: {:?}", &path, err))
        .ok()?;

    let entry = entry.section("Desktop Entry");

    let name = entry.attr("Name")?;
    let icon = entry.attr("Icon").map(|s| s.to_string());
    let no_display = entry.attr("NoDisplay").map(|val| val == "true").unwrap_or(false);
    let hidden = entry.attr("Hidden").map(|val| val == "true").unwrap_or(false);
    // TODO NotShowIn, OnlyShowIn https://wiki.archlinux.org/title/desktop_entries
    // TODO DBusActivatable
    if no_display || hidden {
        return None
    }

    let command = vec!["gtk-launch".to_string(), desktop_filename];

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
                .inspect_err(|err| tracing::warn!("error processing icon {:?}: {:?}", &path, err))
                .ok()
        })
        .flatten();

    Some(DesktopEntry {
        name: name.to_string(),
        icon,
        command,
    })
}
