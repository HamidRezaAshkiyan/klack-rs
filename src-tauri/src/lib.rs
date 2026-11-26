pub mod audio_engine;
pub mod commands;
pub mod input_hook;
pub mod models;
pub mod platform;
pub mod sleep_monitor;
pub mod sound_bank;
pub mod state;
pub mod stats;
pub mod storage;
pub mod tray;

use crate::sleep_monitor::evaluate_sleep_status;
use crate::state::AppState;
use crate::tray::{setup_tray, update_tray_visual};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

fn resolve_sounds_dir(app: &tauri::App) -> PathBuf {
    if let Ok(res_dir) = app.path().resource_dir() {
        let bundled = res_dir.join("sounds");
        if bundled.exists() {
            return bundled;
        }
    }
    // Dev fallback relative to src-tauri/
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/sounds");
    if dev_path.exists() {
        return dev_path;
    }
    PathBuf::from("resources/sounds")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "klack_rs=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from(".klack-rs"));
            let sounds_dir = resolve_sounds_dir(app);

            let state = AppState::new(data_dir, sounds_dir);
            app.manage(Arc::clone(&state));

            // System tray / menubar
            if let Err(err) = setup_tray(app.handle(), Arc::clone(&state)) {
                tracing::warn!("tray setup failed: {}", err);
            }

            // Global toggle shortcut
            let shortcut = state.config.read().toggle_shortcut.clone();
            if !shortcut.trim().is_empty() {
                if let Err(err) = commands::register_toggle_shortcut(app.handle(), &shortcut) {
                    tracing::warn!("failed to register shortcut '{}': {}", shortcut, err);
                }
            }

            // Global OS input hook
            let state_for_hook = Arc::clone(&state);
            let hook_active = Arc::clone(&state.hook_active);
            platform::start_global_input_listener(
                Arc::new(move |event| {
                    state_for_hook.handle_input_event(event);
                }),
                hook_active,
            );

            // Background Sleep Trigger & Stats persistence thread
            let state_for_bg = Arc::clone(&state);
            let handle_for_bg = app.handle().clone();
            std::thread::Builder::new()
                .name("klack-sleep-and-stats".to_string())
                .spawn(move || {
                    let mut ticks = 0u64;
                    loop {
                        std::thread::sleep(Duration::from_secs(2));
                        ticks += 1;

                        let triggers = state_for_bg.config.read().sleep_triggers.clone();
                        let ctx = platform::sample_os_context();
                        let next_sleep = evaluate_sleep_status(&triggers, &ctx);

                        let changed = {
                            let mut cur = state_for_bg.sleep_status.write();
                            if *cur != next_sleep {
                                *cur = next_sleep.clone();
                                true
                            } else {
                                false
                            }
                        };

                        if changed {
                            update_tray_visual(&handle_for_bg, &state_for_bg);
                            let _ = handle_for_bg.emit("sleep-status-changed", &next_sleep);
                        }

                        // Persist stats every 20 seconds and emit live snapshot every 2 seconds
                        let snap = state_for_bg.stats.snapshot();
                        let _ = handle_for_bg.emit("stats-updated", &snap);
                        if ticks % 10 == 0 {
                            let _ = state_for_bg.storage.save_stats(&snap);
                        }
                    }
                })
                .ok();

            Ok(())
        })
        .on_window_event(|window, event| {
            // Keep running discreetly in the menubar/tray when the window is closed
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap,
            commands::update_config,
            commands::toggle_enabled,
            commands::preview_switch,
            commands::preview_sprite,
            commands::save_custom_pack,
            commands::delete_custom_pack,
            commands::list_audio_devices,
            commands::get_stats,
            commands::reset_stats,
            commands::get_sleep_status,
            commands::get_permissions,
            commands::open_accessibility_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Klack-rs");
}
