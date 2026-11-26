use crate::input_hook::InputEvent;
use crate::models::PermissionStatus;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[derive(Debug, Clone, Default)]
pub struct OsContextSnapshot {
    pub active_app: Option<String>,
    pub is_fullscreen: bool,
    pub is_focus_dnd: bool,
    pub is_screen_sharing: bool,
    pub is_media_playing: bool,
}

pub type InputCallback = Arc<dyn Fn(InputEvent) + Send + Sync + 'static>;

pub fn start_global_input_listener(callback: InputCallback, hook_active: Arc<AtomicBool>) {
    start_platform_hook(callback, hook_active);
}

pub fn get_permission_status(hook_active: bool) -> PermissionStatus {
    platform_permission_status(hook_active)
}

pub fn open_accessibility_settings() -> Result<(), String> {
    platform_open_accessibility_settings()
}

pub fn sample_os_context() -> OsContextSnapshot {
    platform_sample_os_context()
}
