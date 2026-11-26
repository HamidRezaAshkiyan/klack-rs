use crate::input_hook::InputEvent;
use crate::models::{KeyEdge, KeyZone, MouseButtonKind, PermissionStatus};
use crate::platform::{InputCallback, OsContextSnapshot};
use rdev::{listen, Button, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn platform_permission_status(hook_active: bool) -> PermissionStatus {
    PermissionStatus {
        platform: "linux".to_string(),
        accessibility_granted: true,
        input_hook_active: hook_active,
        note: "Linux global input hook active via X11/rdev.".to_string(),
    }
}

pub fn platform_open_accessibility_settings() -> Result<(), String> {
    Ok(())
}

fn map_rdev_key(key: Key) -> (u32, KeyZone, f32) {
    let code = format!("{:?}", key)
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let (zone, pan) = match key {
        Key::Space => (KeyZone::Space, 0.0),
        Key::Return | Key::KpReturn => (KeyZone::Enter, 0.52),
        Key::Backspace | Key::Delete => (KeyZone::Backspace, 0.58),
        Key::ShiftLeft | Key::ControlLeft | Key::Alt | Key::MetaLeft | Key::CapsLock => {
            (KeyZone::Modifier, -0.65)
        }
        Key::ShiftRight | Key::ControlRight | Key::AltGr | Key::MetaRight => {
            (KeyZone::Modifier, 0.52)
        }
        Key::F1 | Key::F2 | Key::F3 | Key::F4 => (KeyZone::Function, -0.5),
        Key::F5 | Key::F6 | Key::F7 | Key::F8 => (KeyZone::Function, 0.0),
        Key::F9 | Key::F10 | Key::F11 | Key::F12 => (KeyZone::Function, 0.5),
        Key::Kp0
        | Key::Kp1
        | Key::Kp2
        | Key::Kp3
        | Key::Kp4
        | Key::Kp5
        | Key::Kp6
        | Key::Kp7
        | Key::Kp8
        | Key::Kp9 => (KeyZone::Numpad, 0.72),
        Key::KeyQ | Key::KeyA | Key::KeyZ | Key::Num1 => (KeyZone::Alpha, -0.62),
        Key::KeyW | Key::KeyS | Key::KeyX | Key::Num2 => (KeyZone::Alpha, -0.50),
        Key::KeyE | Key::KeyD | Key::KeyC | Key::Num3 => (KeyZone::Alpha, -0.38),
        Key::KeyR | Key::KeyF | Key::KeyV | Key::Num4 => (KeyZone::Alpha, -0.24),
        Key::KeyT | Key::KeyG | Key::KeyB | Key::Num5 => (KeyZone::Alpha, -0.12),
        Key::KeyY | Key::KeyH | Key::KeyN | Key::Num6 => (KeyZone::Alpha, 0.04),
        Key::KeyU | Key::KeyJ | Key::KeyM | Key::Num7 => (KeyZone::Alpha, 0.16),
        Key::KeyI | Key::KeyK | Key::Comma | Key::Num8 => (KeyZone::Alpha, 0.28),
        Key::KeyO | Key::KeyL | Key::Dot | Key::Num9 => (KeyZone::Alpha, 0.40),
        Key::KeyP | Key::SemiColon | Key::Slash | Key::Num0 => (KeyZone::Alpha, 0.52),
        _ => (KeyZone::Alpha, 0.0),
    };
    (code, zone, pan)
}

pub fn start_platform_hook(callback: InputCallback, hook_active: Arc<AtomicBool>) {
    thread::Builder::new()
        .name("klack-linux-rdev".to_string())
        .spawn(move || {
            hook_active.store(true, Ordering::Relaxed);
            let _ = listen(move |event| match event.event_type {
                EventType::KeyPress(k) => {
                    let (code, zone, pan) = map_rdev_key(k);
                    callback(InputEvent::Key {
                        code,
                        zone,
                        pan,
                        edge: KeyEdge::Down,
                    });
                }
                EventType::KeyRelease(k) => {
                    let (code, zone, pan) = map_rdev_key(k);
                    callback(InputEvent::Key {
                        code,
                        zone,
                        pan,
                        edge: KeyEdge::Up,
                    });
                }
                EventType::ButtonPress(b) => {
                    let button = match b {
                        Button::Left => MouseButtonKind::Left,
                        Button::Right => MouseButtonKind::Right,
                        Button::Middle => MouseButtonKind::Middle,
                        _ => MouseButtonKind::Other,
                    };
                    callback(InputEvent::MouseButton {
                        button,
                        edge: KeyEdge::Down,
                    });
                }
                EventType::ButtonRelease(b) => {
                    let button = match b {
                        Button::Left => MouseButtonKind::Left,
                        Button::Right => MouseButtonKind::Right,
                        Button::Middle => MouseButtonKind::Middle,
                        _ => MouseButtonKind::Other,
                    };
                    callback(InputEvent::MouseButton {
                        button,
                        edge: KeyEdge::Up,
                    });
                }
                EventType::Wheel { delta_y, .. } if delta_y != 0 => {
                    callback(InputEvent::Scroll {
                        delta: delta_y as f32,
                    });
                }
                _ => {}
            });
            hook_active.store(false, Ordering::Relaxed);
        })
        .ok();
}

pub fn platform_sample_os_context() -> OsContextSnapshot {
    OsContextSnapshot::default()
}
