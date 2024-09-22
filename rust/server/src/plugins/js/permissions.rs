use deno_runtime::permissions::{Descriptor, EnvDescriptor, NetDescriptor, Permissions, PermissionsContainer, ReadDescriptor, RunDescriptor, SysDescriptor, UnaryPermission, WriteDescriptor};
use std::collections::HashSet;
use std::hash::Hash;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::anyhow;
use typed_path::Utf8TypedPath;
use common::dirs::Dirs;
use common::model::PluginId;
use crate::plugins::loader::VARIABLE_PATTERN;

pub struct PluginPermissions {
    pub environment: Vec<String>,
    pub network: Vec<String>,
    pub filesystem: PluginPermissionsFileSystem,
    pub exec: PluginPermissionsExec,
    pub system: Vec<String>,
    pub clipboard: Vec<PluginPermissionsClipboard>,
    pub main_search_bar: Vec<PluginPermissionsMainSearchBar>,
}

pub struct PluginPermissionsFileSystem {
    pub read: Vec<String>,
    pub write: Vec<String>,
}

pub struct PluginPermissionsExec {
    pub command: Vec<String>,
    pub executable: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PluginPermissionsClipboard {
    Read,
    Write,
    Clear
}

#[derive(Clone, Debug)]
pub enum PluginPermissionsMainSearchBar {
    Read,
}

pub fn permissions_to_deno(permissions: &PluginPermissions, dirs: &Dirs, plugin_uuid: &str) -> anyhow::Result<PermissionsContainer> {
    Ok(PermissionsContainer::new(Permissions {
        read: path_permission(&permissions.filesystem.read, ReadDescriptor, dirs, plugin_uuid)?,
        write: path_permission(&permissions.filesystem.write, WriteDescriptor, dirs, plugin_uuid)?,
        net: net_permission(&permissions.network),
        env: env_permission(&permissions.environment),
        sys: sys_permission(&permissions.system),
        run: run_permission(&permissions.exec, dirs, plugin_uuid)?,
        ffi: Permissions::new_ffi(&None, &None, false).expect("new_ffi should always succeed"),
        hrtime: Permissions::new_hrtime(true, false),
    }))
}

fn path_permission<T: Descriptor + Hash>(
    paths: &[String],
    to_permission: fn(PathBuf) -> T,
    dirs: &Dirs,
    plugin_uuid: &str
) -> anyhow::Result<UnaryPermission<T>> {
    let granted = paths
        .into_iter()
        .map(|path| {
            augment_path(path, dirs, plugin_uuid)
                .map(|path| path.map(|path| to_permission(path)))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(std::convert::identity)
        .collect::<HashSet<_>>();

    Ok(UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    })
}

fn net_permission(domain_and_ports: &[String]) -> UnaryPermission<NetDescriptor> {
    let granted = domain_and_ports
        .into_iter()
        .map(|domain_and_port| {
            NetDescriptor::from_str(&domain_and_port)
                .expect("should be validated when loading")
        })
        .collect();

    UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    }
}

fn env_permission(envs: &[String]) -> UnaryPermission<EnvDescriptor> {
    let granted = envs
        .into_iter()
        .map(|env| EnvDescriptor::new(env))
        .collect();

    UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    }
}

fn sys_permission(system: &[String]) -> UnaryPermission<SysDescriptor> {
    let granted = system
        .into_iter()
        .map(|system| SysDescriptor(system.to_owned()))
        .collect();

    UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    }
}

fn run_permission(permissions: &PluginPermissionsExec, dirs: &Dirs, plugin_uuid: &str) -> anyhow::Result<UnaryPermission<RunDescriptor>> {
    let granted_executable = permissions.executable
        .iter()
        .map(|path| {
            augment_path(path, dirs, plugin_uuid)
                .map(|path| path.map(|path| RunDescriptor::Path(path)))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(std::convert::identity)
        .collect::<HashSet<_>>();

    let granted_command = permissions.command
        .iter()
        .map(|cmd| RunDescriptor::Name(cmd.to_owned()))
        .collect::<HashSet<_>>();

    let mut granted = HashSet::new();
    granted.extend(granted_executable);
    granted.extend(granted_command);

    Ok(UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    })
}

fn augment_path(path: &String, dirs: &Dirs, plugin_uuid: &str) -> anyhow::Result<Option<PathBuf>> {
    if let Some(matches) = VARIABLE_PATTERN.captures(path) {
        let namespace = &matches["namespace"];
        let name = &matches["name"];

        let replacement = match (namespace, name) {
            ("macos", "user-home") => {
                if cfg!(target_os = "macos") {
                    Some(dirs.home_dir())
                } else {
                    None
                }
            },
            ("linux", "user-home") => {
                if cfg!(target_os = "linux") {
                    Some(dirs.home_dir())
                } else {
                    None
                }
            },
            ("windows", "user-home") => {
                if cfg!(windows) {
                    Some(dirs.home_dir())
                } else {
                    None
                }
            },
            ("common", "plugin-data") => Some(dirs.plugin_data(plugin_uuid)?),
            ("common", "plugin-cache") => Some(dirs.plugin_cache(plugin_uuid)?),
            (_, _) => {
                Err(anyhow!("Trying to load plugin with unknown variable in path in manifest permissions: {}", path))?
            }
        };

        match replacement {
            None => Ok(None),
            Some(replacement) => {
                let replacement = replacement.to_str()
                    .expect("non-utf8 file paths are not supported");

                Ok(Some(PathBuf::from(VARIABLE_PATTERN.replace(path, replacement).to_string())))
            }
        }
    } else {
        match Utf8TypedPath::derive(&path) {
            Utf8TypedPath::Unix(_) => {
                if cfg!(unix) {
                    Ok(Some(PathBuf::from(path)))
                } else {
                    Ok(None)
                }
            }
            Utf8TypedPath::Windows(_) => {
                if cfg!(windows) {
                    Ok(Some(PathBuf::from(path)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}