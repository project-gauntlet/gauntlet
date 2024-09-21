use deno_runtime::permissions::{Descriptor, EnvDescriptor, NetDescriptor, Permissions, PermissionsContainer, ReadDescriptor, RunDescriptor, SysDescriptor, UnaryPermission, WriteDescriptor};
use std::collections::HashSet;
use std::hash::Hash;
use std::path::PathBuf;
use std::str::FromStr;

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

pub fn permissions_to_deno(permissions: &PluginPermissions) -> PermissionsContainer {
    PermissionsContainer::new(Permissions {
        read: path_permission(&permissions.filesystem.read, ReadDescriptor),
        write: path_permission(&permissions.filesystem.write, WriteDescriptor),
        net: net_permission(&permissions.network),
        env: env_permission(&permissions.environment),
        sys: sys_permission(&permissions.system),
        run: run_permission(&permissions.exec),
        ffi: Permissions::new_ffi(&None, &None, false).expect("new_ffi should always succeed"),
        hrtime: Permissions::new_hrtime(true, false),
    })
}

fn path_permission<T: Descriptor + Hash>(
    paths: &[String],
    to_permission: fn(PathBuf) -> T
) -> UnaryPermission<T> {
    let granted = paths
        .into_iter()
        .map(|path| to_permission(PathBuf::from(path)))
        .collect();

    UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    }
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

fn run_permission(permissions: &PluginPermissionsExec) -> UnaryPermission<RunDescriptor> {
    let granted_executable = permissions.executable
        .iter()
        .map(|path| RunDescriptor::Path(PathBuf::from(path)))
        .collect::<Vec<_>>();

    let granted_command = permissions.command
        .iter()
        .map(|cmd| RunDescriptor::Name(cmd.to_owned()))
        .collect::<HashSet<_>>();

    let mut granted = HashSet::new();
    granted.extend(granted_executable);
    granted.extend(granted_command);

    UnaryPermission {
        prompt: false,
        granted_global: false,
        flag_denied_global: false,
        granted_list: granted,
        ..Default::default()
    }
}
