use crate::state::AppState;
use std::sync::Arc;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub const TRAY_ID: &str = "klack-tray";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayVisualState {
    Active,
    Muted,
    Sleeping,
}

fn load_tray_image(state: TrayVisualState) -> Option<Image<'static>> {
    let bytes: &[u8] = match state {
        TrayVisualState::Active => include_bytes!("../icons/tray-active.png"),
        TrayVisualState::Muted => include_bytes!("../icons/tray-muted.png"),
        TrayVisualState::Sleeping => include_bytes!("../icons/tray-sleeping.png"),
    };
    Image::from_bytes(bytes).ok()
}

pub fn update_tray_visual(app: &AppHandle, state: &AppState) {
    let cfg = state.config.read();
    let sleep = state.sleep_status.read();

    let visual = if !cfg.enabled {
        TrayVisualState::Muted
    } else if sleep.is_sleeping {
        TrayVisualState::Sleeping
    } else {
        TrayVisualState::Active
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(!cfg.hide_tray_icon);
        if let Some(img) = load_tray_image(visual) {
            let _ = tray.set_icon(Some(img));
        }
        let tooltip = match visual {
            TrayVisualState::Active => format!("Klack-rs — Active ({})", cfg.active_switch_id),
            TrayVisualState::Muted => "Klack-rs — Muted".to_string(),
            TrayVisualState::Sleeping => format!(
                "Klack-rs — Sleeping ({})",
                sleep.reason.as_deref().unwrap_or("Trigger active")
            ),
        };
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

pub fn setup_tray(app: &AppHandle, state: Arc<AppState>) -> tauri::Result<()> {
    let enabled = state.config.read().enabled;
    let toggle_item = CheckMenuItem::with_id(app, "toggle", "Sound Enabled", true, enabled, None::<&str>)?;
    let show_popover = MenuItem::with_id(app, "popover", "Quick Controls...", true, None::<&str>)?;
    let show_settings = MenuItem::with_id(app, "settings", "Klack Settings...", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Klack-rs", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&toggle_item, &sep, &show_popover, &show_settings, &quit_item],
    )?;

    let initial_icon = load_tray_image(if enabled {
        TrayVisualState::Active
    } else {
        TrayVisualState::Muted
    });

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Klack-rs");

    if let Some(icon) = initial_icon {
        builder = builder.icon(icon);
    }

    let state_for_menu = Arc::clone(&state);
    let toggle_handle = toggle_item.clone();

    builder
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                let next_enabled = {
                    let mut cfg = state_for_menu.config.write();
                    cfg.enabled = !cfg.enabled;
                    let _ = state_for_menu.storage.save_config(&cfg);
                    cfg.enabled
                };
                let _ = toggle_handle.set_checked(next_enabled);
                update_tray_visual(app, &state_for_menu);
                let _ = app.emit("enabled-changed", next_enabled);
            }
            "popover" => {
                open_main_window(app, "popover");
            }
            "settings" => {
                open_main_window(app, "settings");
            }
            "quit" => {
                let snap = state_for_menu.stats.snapshot();
                let _ = state_for_menu.storage.save_stats(&snap);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_main_window(app);
            }
        })
        .build(app)?;

    update_tray_visual(app, &state);
    Ok(())
}

pub fn open_main_window(app: &AppHandle, mode: &str) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
        let _ = app.emit("ui-mode-changed", mode);
    }
}

pub fn toggle_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.unminimize();
            let _ = win.set_focus();
        }
    }
}
