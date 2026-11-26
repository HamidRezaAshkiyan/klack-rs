use crate::input_hook::{map_windows_vk, InputEvent};
use crate::models::{KeyEdge, MouseButtonKind, PermissionStatus};
use crate::platform::{InputCallback, OsContextSnapshot};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetMessageW, GetWindowTextW, SetWindowsHookExW,
    UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static GLOBAL_CB: std::sync::OnceLock<InputCallback> = std::sync::OnceLock::new();

pub fn platform_permission_status(hook_active: bool) -> PermissionStatus {
    PermissionStatus {
        platform: "windows".to_string(),
        accessibility_granted: true,
        input_hook_active: hook_active,
        note: "Windows low-level keyboard and mouse hooks active (no extra permissions required)."
            .to_string(),
    }
}

pub fn platform_open_accessibility_settings() -> Result<(), String> {
    Ok(())
}

unsafe extern "system" fn kb_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        if let Some(cb) = GLOBAL_CB.get() {
            let info = *(lparam as *const KBDLLHOOKSTRUCT);
            let msg = wparam as u32;
            let edge = match msg {
                WM_KEYDOWN | WM_SYSKEYDOWN => Some(KeyEdge::Down),
                WM_KEYUP | WM_SYSKEYUP => Some(KeyEdge::Up),
                _ => None,
            };
            if let Some(edge) = edge {
                let (zone, pan) = map_windows_vk(info.vkCode);
                cb(InputEvent::Key {
                    code: info.vkCode,
                    zone,
                    pan,
                    edge,
                });
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        if let Some(cb) = GLOBAL_CB.get() {
            let msg = wparam as u32;
            match msg {
                WM_LBUTTONDOWN => cb(InputEvent::MouseButton {
                    button: MouseButtonKind::Left,
                    edge: KeyEdge::Down,
                }),
                WM_LBUTTONUP => cb(InputEvent::MouseButton {
                    button: MouseButtonKind::Left,
                    edge: KeyEdge::Up,
                }),
                WM_RBUTTONDOWN => cb(InputEvent::MouseButton {
                    button: MouseButtonKind::Right,
                    edge: KeyEdge::Down,
                }),
                WM_RBUTTONUP => cb(InputEvent::MouseButton {
                    button: MouseButtonKind::Right,
                    edge: KeyEdge::Up,
                }),
                WM_MBUTTONDOWN => cb(InputEvent::MouseButton {
                    button: MouseButtonKind::Middle,
                    edge: KeyEdge::Down,
                }),
                WM_MBUTTONUP => cb(InputEvent::MouseButton {
                    button: MouseButtonKind::Middle,
                    edge: KeyEdge::Up,
                }),
                WM_MOUSEWHEEL => {
                    let info = *(lparam as *const MSLLHOOKSTRUCT);
                    let delta = ((info.mouseData >> 16) as i16) as f32 / 120.0;
                    if delta.abs() > 0.0 {
                        cb(InputEvent::Scroll { delta });
                    }
                }
                _ => {}
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

pub fn start_platform_hook(callback: InputCallback, hook_active: Arc<AtomicBool>) {
    let _ = GLOBAL_CB.set(callback);
    thread::Builder::new()
        .name("klack-win-hooks".to_string())
        .spawn(move || unsafe {
            let kb = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kb_proc), std::ptr::null_mut(), 0);
            let ms = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), std::ptr::null_mut(), 0);
            if !kb.is_null() {
                hook_active.store(true, Ordering::Relaxed);
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
                UnhookWindowsHookEx(kb);
            }
            if !ms.is_null() {
                UnhookWindowsHookEx(ms);
            }
            hook_active.store(false, Ordering::Relaxed);
        })
        .ok();
}

pub fn platform_sample_os_context() -> OsContextSnapshot {
    let mut ctx = OsContextSnapshot::default();
    unsafe {
        let hwnd = GetForegroundWindow();
        if !hwnd.is_null() {
            let mut buf = [0u16; 256];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                let lower = title.to_lowercase();
                if lower.contains("zoom")
                    || lower.contains("discord")
                    || lower.contains("obs")
                    || lower.contains("teams")
                {
                    ctx.is_screen_sharing = true;
                }
                ctx.active_app = Some(title);
            }
        }
    }
    ctx
}
