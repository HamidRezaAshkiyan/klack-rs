use crate::audio_engine::AudioEngine;
use crate::models::{
    AppConfig, AudioDeviceInfo, BootstrapPayload, PermissionStatus, SleepStatus, SwitchPack,
    TypingStats,
};
use crate::platform;
use crate::sound_bank::sprite_catalog;
use crate::state::AppState;
use crate::tray::update_tray_visual;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn get_bootstrap(state: State<'_, Arc<AppState>>) -> BootstrapPayload {
    BootstrapPayload {
        config: state.config.read().clone(),
        switch_packs: state.all_switch_packs(),
        sprites: sprite_catalog(),
        audio_devices: AudioEngine::list_output_devices(),
        sleep_status: state.sleep_status.read().clone(),
        stats: state.stats.snapshot(),
        permissions: platform::get_permission_status(state.hook_active.load(Ordering::Relaxed)),
    }
}

#[tauri::command]
pub fn update_config(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    new_config: AppConfig,
) -> Result<AppConfig, String> {
    let prev = state.config.read().clone();

    // Update volume immediately
    if (prev.volume - new_config.volume).abs() > f32::EPSILON {
        state.audio.set_volume(new_config.volume);
    }

    // Switch audio output device if changed
    if prev.output_device != new_config.output_device {
        if let Ok(sr) = state
            .audio
            .switch_output_device(new_config.output_device.as_deref())
        {
            state.rebuild_sound_bank_for_rate(sr);
        }
    }

    // Launch at login
    if prev.launch_at_login != new_config.launch_at_login {
        let autostart = app.autolaunch();
        if new_config.launch_at_login {
            let _ = autostart.enable();
        } else {
            let _ = autostart.disable();
        }
    }

    // Global shortcut re-registration if changed
    if prev.toggle_shortcut != new_config.toggle_shortcut {
        let _ = app.global_shortcut().unregister_all();
        if !new_config.toggle_shortcut.trim().is_empty() {
            let _ = register_toggle_shortcut(&app, &new_config.toggle_shortcut);
        }
    }

    let switch_changed = prev.active_switch_id != new_config.active_switch_id;

    {
        let mut cfg = state.config.write();
        *cfg = new_config.clone();
        state.storage.save_config(&cfg)?;
    }

    state.preload_active_assets();
    update_tray_visual(&app, &state);

    if switch_changed && new_config.notify_on_switch_change {
        let pack = state.active_switch_pack();
        let _ = app
            .notification()
            .builder()
            .title("Klack-rs Switch Changed")
            .body(format!("{} — {}", pack.name, pack.feel))
            .show();
    }

    let _ = app.emit("config-updated", &new_config);
    Ok(new_config)
}

#[tauri::command]
pub fn toggle_enabled(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let next = {
        let mut cfg = state.config.write();
        cfg.enabled = !cfg.enabled;
        state.storage.save_config(&cfg)?;
        cfg.enabled
    };
    update_tray_visual(&app, &state);
    let _ = app.emit("enabled-changed", next);
    Ok(next)
}

#[tauri::command]
pub fn preview_switch(state: State<'_, Arc<AppState>>, pack_id: String) {
    state.preview_switch_pack(&pack_id);
}

#[tauri::command]
pub fn preview_sprite(
    state: State<'_, Arc<AppState>>,
    sprite_id: String,
    volume: Option<f32>,
) {
    state.preview_sprite(&sprite_id, volume);
}

#[tauri::command]
pub fn save_custom_pack(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    mut pack: SwitchPack,
) -> Result<Vec<SwitchPack>, String> {
    pack.is_custom = true;
    {
        let mut cfg = state.config.write();
        if let Some(existing) = cfg.custom_packs.iter_mut().find(|p| p.id == pack.id) {
            *existing = pack.clone();
        } else {
            cfg.custom_packs.push(pack.clone());
        }
        cfg.active_switch_id = pack.id.clone();
        state.storage.save_config(&cfg)?;
    }
    state.preload_active_assets();
    update_tray_visual(&app, &state);
    Ok(state.all_switch_packs())
}

#[tauri::command]
pub fn delete_custom_pack(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    pack_id: String,
) -> Result<Vec<SwitchPack>, String> {
    {
        let mut cfg = state.config.write();
        cfg.custom_packs.retain(|p| p.id != pack_id);
        if cfg.active_switch_id == pack_id {
            cfg.active_switch_id = "leobog-hi75".to_string();
        }
        state.storage.save_config(&cfg)?;
    }
    state.preload_active_assets();
    update_tray_visual(&app, &state);
    Ok(state.all_switch_packs())
}

#[tauri::command]
pub fn list_audio_devices() -> Vec<AudioDeviceInfo> {
    AudioEngine::list_output_devices()
}

#[tauri::command]
pub fn get_stats(state: State<'_, Arc<AppState>>) -> TypingStats {
    state.stats.snapshot()
}

#[tauri::command]
pub fn reset_stats(state: State<'_, Arc<AppState>>) -> Result<TypingStats, String> {
    let fresh = state.stats.reset();
    state.storage.save_stats(&fresh)?;
    Ok(fresh)
}

#[tauri::command]
pub fn get_sleep_status(state: State<'_, Arc<AppState>>) -> SleepStatus {
    state.sleep_status.read().clone()
}

#[tauri::command]
pub fn get_permissions(state: State<'_, Arc<AppState>>) -> PermissionStatus {
    platform::get_permission_status(state.hook_active.load(Ordering::Relaxed))
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    platform::open_accessibility_settings()
}

pub fn register_toggle_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    use tauri::Manager;
    use tauri_plugin_global_shortcut::ShortcutState;

    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut_str, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(state) = app_handle.try_state::<Arc<AppState>>() {
                    let next = {
                        let mut cfg = state.config.write();
                        cfg.enabled = !cfg.enabled;
                        let _ = state.storage.save_config(&cfg);
                        cfg.enabled
                    };
                    update_tray_visual(&app_handle, &state);
                    let _ = app_handle.emit("enabled-changed", next);
                }
            }
        })
        .map_err(|e| e.to_string())
}
