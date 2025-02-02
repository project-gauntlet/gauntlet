use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use deno_core::op2;
use deno_core::OpState;
use freedesktop_entry_parser::parse_entry;
use freedesktop_icons::lookup;
use image::imageops::FilterType;
use image::ImageFormat;
use tokio::sync::mpsc::Sender;
use tokio::task::spawn_blocking;
use walkdir::WalkDir;

use crate::plugin_data::PluginData;
use crate::plugins::applications::linux;
use crate::plugins::applications::resize_icon;
use crate::plugins::applications::spawn_detached;
use crate::plugins::applications::DesktopApplication;
use crate::plugins::applications::DesktopPathAction;

mod wayland;
mod x11;

deno_core::extension!(
    gauntlet_internal_linux,
    ops = [
        // plugins applications
        linux_app_from_path,
        linux_application_dirs,
        linux_open_application,
        x11::linux_x11_focus_window,
        x11::application_x11_pending_event,
        wayland::linux_wayland_focus_window,
        wayland::application_wayland_pending_event,
    ],
    esm_entry_point = "ext:gauntlet/internal-linux/bootstrap.js",
    esm = [
        "ext:gauntlet/internal-linux/bootstrap.js" = "../../js/bridge_build/dist/bridge-internal-linux-bootstrap.js",
        "ext:gauntlet/internal-linux.js" = "../../js/core/dist/internal-linux.js",
    ]
);

pub enum LinuxDesktopEnvironment {
    X11(x11::X11DesktopEnvironment),
    Wayland(wayland::WaylandDesktopEnvironment),
}

impl LinuxDesktopEnvironment {
    pub fn new() -> anyhow::Result<Self> {
        let wayland = std::env::var("WAYLAND_DISPLAY")
            .or_else(|_| std::env::var("WAYLAND_SOCKET"))
            .is_ok();

        if wayland {
            Ok(LinuxDesktopEnvironment::Wayland(
                wayland::WaylandDesktopEnvironment::new()?,
            ))
        } else {
            Ok(LinuxDesktopEnvironment::X11(x11::X11DesktopEnvironment::new()))
        }
    }

    pub fn is_wayland(&self) -> bool {
        matches!(self, LinuxDesktopEnvironment::Wayland(_))
    }
}

#[op2(async)]
#[serde]
async fn linux_app_from_path(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> anyhow::Result<Option<DesktopPathAction>> {
    let home_dir = {
        let state = state.borrow();

        let home_dir = state.borrow::<PluginData>().home_dir();

        home_dir
    };

    Ok(spawn_blocking(|| linux_app_from_path_async(home_dir, PathBuf::from(path))).await?)
}

#[op2]
#[serde]
fn linux_application_dirs(state: Rc<RefCell<OpState>>) -> Vec<String> {
    let home_dir = {
        let state = state.borrow();

        let home_dir = state.borrow::<PluginData>().home_dir();

        home_dir
    };

    linux_application_dirs_inner(home_dir)
        .into_iter()
        .map(|path| path.to_str().expect("non-utf8 paths are not supported").to_string())
        .collect()
}

#[op2(fast)]
fn linux_open_application(#[string] desktop_file_id: String) -> anyhow::Result<()> {
    spawn_detached("gtk-launch", &[desktop_file_id])?;

    Ok(())
}

fn linux_application_dirs_inner(home_dir: PathBuf) -> Vec<PathBuf> {
    let data_home = match std::env::var_os("XDG_DATA_HOME") {
        Some(val) => PathBuf::from(val),
        None => home_dir.join(".local").join("share"),
    };

    let mut extra_data_dirs = match std::env::var_os("XDG_DATA_DIRS") {
        Some(val) => std::env::split_paths(&val).map(PathBuf::from).collect(),
        None => {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
                PathBuf::from("/var/lib/flatpak/exports/share"),
            ]
        }
    };

    let flatpak = data_home.to_path_buf().join("flatpak").join("exports").join("share");

    let mut res = Vec::new();
    res.push(data_home);
    res.push(flatpak);
    res.append(&mut extra_data_dirs);

    res.into_iter().map(|d| d.join("applications")).collect()
}

fn linux_app_from_path_async(home_dir: PathBuf, path: PathBuf) -> Option<DesktopPathAction> {
    let app_directories = linux_application_dirs_inner(home_dir);

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

    let desktop_file_name = relative_to_app_dir
        .strip_suffix(".desktop")
        .unwrap_or(&relative_to_app_dir)
        .to_string();

    let desktop_app_id = desktop_file_name.replace("/", "-");

    if !path.exists() {
        tracing::debug!("Removing application at: {:?}", path);
        Some(DesktopPathAction::Remove { id: desktop_app_id })
    } else {
        // follows symlinks needed for flatpak
        let Ok(metadata) = std::fs::metadata(&path) else {
            return None;
        };

        if !metadata.is_file() {
            return None;
        }

        if let Some(entry) = create_app_entry(&path) {
            tracing::debug!("Adding application at: {:?}", path);

            Some(DesktopPathAction::Add {
                id: desktop_app_id,
                data: entry,
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

    let desktop_file_path_str = desktop_file_path
        .to_str()
        .expect("non-utf8 paths are not supported")
        .to_string();

    let entry = entry.section("Desktop Entry");

    let name = entry.attr("Name")?;
    let icon = entry.attr("Icon").map(|s| s.to_string());
    let no_display = entry.attr("NoDisplay").map(|val| val == "true").unwrap_or(false);
    let hidden = entry.attr("Hidden").map(|val| val == "true").unwrap_or(false);
    let startup_wm_class = entry.attr("StartupWMClass").map(|s| s.to_string());
    // TODO NotShowIn, OnlyShowIn https://wiki.archlinux.org/title/desktop_entries

    if no_display || hidden {
        return None;
    }

    let icon = icon
        .map(|icon| {
            let icon_path = PathBuf::from(&icon);
            if icon_path.is_absolute() {
                Some(icon_path)
            } else {
                lookup(&icon).with_size(48).find()
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
                        }
                        Some("svg") => {
                            let data = std::fs::read(path)?;

                            let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())?;

                            let pixmap_size = tree.size().to_int_size();
                            let mut pixmap =
                                resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

                            resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

                            let data = pixmap.encode_png()?;

                            let data = resize_icon(data)?;

                            Ok(data)
                        }
                        Some("xpm") => Err(anyhow::anyhow!("xpm format")),
                        _ => Err(anyhow::anyhow!("unsupported by spec format {:?}", extension)),
                    }
                }
            }
        })
        .map(|res| {
            res.inspect_err(|err| tracing::warn!("error processing icon of {:?}: {:?}", desktop_file_path, err))
                .ok()
        })
        .flatten();

    Some(DesktopApplication {
        name: name.to_string(),
        desktop_file_path: desktop_file_path_str,
        icon,
        startup_wm_class,
    })
}
