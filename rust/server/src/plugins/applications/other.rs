use crate::plugins::applications::DesktopEntry;

#[cfg(not(target_os = "linux"))]
pub fn get_apps() -> Vec<DesktopEntry> {
    vec![]
}