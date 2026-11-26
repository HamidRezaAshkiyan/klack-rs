use crate::input_hook::{map_macos_keycode, InputEvent};
use crate::models::{KeyEdge, MouseButtonKind, PermissionStatus};
use crate::platform::{InputCallback, OsContextSnapshot};
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn is_accessibility_granted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn platform_permission_status(hook_active: bool) -> PermissionStatus {
    let granted = is_accessibility_granted();
    PermissionStatus {
        platform: "macos".to_string(),
        accessibility_granted: granted,
        input_hook_active: hook_active,
        note: if granted {
            "Accessibility permission granted. Global keyboard, mouse, and scroll capture active."
                .to_string()
        } else {
            "Grant Accessibility access in System Settings → Privacy & Security → Accessibility so Klack-rs can hear global keystrokes.".to_string()
        },
    }
}

pub fn platform_open_accessibility_settings() -> Result<(), String> {
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn start_platform_hook(callback: InputCallback, hook_active: Arc<AtomicBool>) {
    thread::Builder::new()
        .name("klack-macos-event-tap".to_string())
        .spawn(move || loop {
            if !is_accessibility_granted() {
                hook_active.store(false, Ordering::Relaxed);
                thread::sleep(Duration::from_secs(2));
                continue;
            }

            let cb = Arc::clone(&callback);
            let last_flags = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let last_flags_cb = Arc::clone(&last_flags);

            let events = vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseUp,
                CGEventType::RightMouseDown,
                CGEventType::RightMouseUp,
                CGEventType::OtherMouseDown,
                CGEventType::OtherMouseUp,
                CGEventType::ScrollWheel,
            ];

            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                events,
                move |_proxy, event_type, event| {
                    match event_type {
                        CGEventType::KeyDown => {
                            let code = event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                as u16;
                            let (zone, pan) = map_macos_keycode(code);
                            cb(InputEvent::Key {
                                code: code as u32,
                                zone,
                                pan,
                                edge: KeyEdge::Down,
                            });
                        }
                        CGEventType::KeyUp => {
                            let code = event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                as u16;
                            let (zone, pan) = map_macos_keycode(code);
                            cb(InputEvent::Key {
                                code: code as u32,
                                zone,
                                pan,
                                edge: KeyEdge::Up,
                            });
                        }
                        CGEventType::FlagsChanged => {
                            let code = event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                as u16;
                            let flags = event.get_flags().bits();
                            let prev = last_flags_cb.swap(flags, Ordering::Relaxed);
                            let edge = if flags > prev {
                                KeyEdge::Down
                            } else {
                                KeyEdge::Up
                            };
                            let (zone, pan) = map_macos_keycode(code);
                            cb(InputEvent::Key {
                                code: code as u32,
                                zone,
                                pan,
                                edge,
                            });
                        }
                        CGEventType::LeftMouseDown => cb(InputEvent::MouseButton {
                            button: MouseButtonKind::Left,
                            edge: KeyEdge::Down,
                        }),
                        CGEventType::LeftMouseUp => cb(InputEvent::MouseButton {
                            button: MouseButtonKind::Left,
                            edge: KeyEdge::Up,
                        }),
                        CGEventType::RightMouseDown => cb(InputEvent::MouseButton {
                            button: MouseButtonKind::Right,
                            edge: KeyEdge::Down,
                        }),
                        CGEventType::RightMouseUp => cb(InputEvent::MouseButton {
                            button: MouseButtonKind::Right,
                            edge: KeyEdge::Up,
                        }),
                        CGEventType::OtherMouseDown => cb(InputEvent::MouseButton {
                            button: MouseButtonKind::Middle,
                            edge: KeyEdge::Down,
                        }),
                        CGEventType::OtherMouseUp => cb(InputEvent::MouseButton {
                            button: MouseButtonKind::Middle,
                            edge: KeyEdge::Up,
                        }),
                        CGEventType::ScrollWheel => {
                            let delta = event.get_integer_value_field(
                                EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1,
                            ) as f32;
                            if delta.abs() > 0.0 {
                                cb(InputEvent::Scroll { delta });
                            }
                        }
                        _ => {}
                    }
                    None
                },
            );

            match tap {
                Ok(event_tap) => {
                    if let Ok(loop_source) = event_tap.mach_port.create_runloop_source(0) {
                        unsafe {
                            let current = CFRunLoop::get_current();
                            current.add_source(&loop_source, kCFRunLoopCommonModes);
                            event_tap.enable();
                            hook_active.store(true, Ordering::Relaxed);
                            tracing::info!("macOS CGEventTap active");
                            CFRunLoop::run_current();
                        }
                    }
                    hook_active.store(false, Ordering::Relaxed);
                }
                Err(_) => {
                    hook_active.store(false, Ordering::Relaxed);
                }
            }
            thread::sleep(Duration::from_secs(2));
        })
        .ok();
}

pub fn platform_sample_os_context() -> OsContextSnapshot {
    let mut ctx = OsContextSnapshot::default();

    // Lightweight frontmost app check via lsappinfo (faster and avoids AppleEvents permission prompts)
    if let Ok(out) = Command::new("lsappinfo").args(["front"]).output() {
        if let Ok(asn) = String::from_utf8(out.stdout) {
            let asn = asn.trim();
            if !asn.is_empty() {
                if let Ok(info) = Command::new("lsappinfo")
                    .args(["info", "-only", "name", asn])
                    .output()
                {
                    if let Ok(s) = String::from_utf8(info.stdout) {
                        if let Some(name) = s.split('"').nth(3) {
                            ctx.active_app = Some(name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Check known screen-sharing / conferencing processes
    if let Some(app) = &ctx.active_app {
        let lower = app.to_lowercase();
        if lower.contains("zoom")
            || lower.contains("obs")
            || lower.contains("loom")
            || lower.contains("discord")
            || lower.contains("teams")
            || lower.contains("facetime")
        {
            ctx.is_screen_sharing = true;
        }
    }

    // Check macOS Focus / Do Not Disturb assertion file (read-only, zero permissions needed)
    if let Ok(home) = std::env::var("HOME") {
        let assertions = format!("{home}/Library/DoNotDisturb/DB/Assertions.json");
        if let Ok(raw) = std::fs::read_to_string(assertions) {
            if raw.contains("storeAssertionRecords") && raw.contains("assertionDetailsModeIdentifier")
            {
                ctx.is_focus_dnd = true;
            }
        }
    }

    ctx
}
