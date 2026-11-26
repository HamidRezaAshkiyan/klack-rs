use crate::models::{KeyEdge, KeyZone, MouseButtonKind};
use parking_lot::Mutex;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Key {
        code: u32,
        zone: KeyZone,
        pan: f32,
        edge: KeyEdge,
    },
    MouseButton {
        button: MouseButtonKind,
        edge: KeyEdge,
    },
    Scroll {
        delta: f32,
    },
}

/// Filters OS key-repeat events so holding a key down only emits one `Down` event until released.
#[derive(Default)]
pub struct RepeatFilter {
    held_keys: Mutex<HashSet<u32>>,
}

impl RepeatFilter {
    pub fn should_emit(&self, code: u32, edge: KeyEdge) -> bool {
        let mut held = self.held_keys.lock();
        match edge {
            KeyEdge::Down => held.insert(code),
            KeyEdge::Up => {
                held.remove(&code);
                true
            }
        }
    }

    pub fn clear(&self) {
        self.held_keys.lock().clear();
    }
}

/// Maps macOS virtual keycodes (Carbon/HIToolbox `CGKeyCode`) to `(KeyZone, spatial_pan)`.
/// `spatial_pan` ranges from `-0.75` (far left) to `+0.75` (far right / numpad).
pub fn map_macos_keycode(code: u16) -> (KeyZone, f32) {
    match code {
        // Spacebar
        49 => (KeyZone::Space, 0.0),
        // Return / Enter
        36 | 76 => (KeyZone::Enter, 0.52),
        // Backspace / Forward Delete
        51 | 117 => (KeyZone::Backspace, 0.58),
        // Modifiers (Cmd, Shift, Caps, Option, Control, Fn)
        54 | 55 => (KeyZone::Modifier, -0.22),
        56 | 57 | 58 | 59 => (KeyZone::Modifier, -0.68),
        60 | 61 | 62 => (KeyZone::Modifier, 0.52),
        63 => (KeyZone::Modifier, -0.72),
        // Function keys F1-F12
        122 => (KeyZone::Function, -0.68),
        120 => (KeyZone::Function, -0.58),
        99 => (KeyZone::Function, -0.48),
        118 => (KeyZone::Function, -0.36),
        96 => (KeyZone::Function, -0.22),
        97 => (KeyZone::Function, -0.10),
        98 => (KeyZone::Function, 0.05),
        100 => (KeyZone::Function, 0.18),
        101 => (KeyZone::Function, 0.30),
        109 => (KeyZone::Function, 0.42),
        103 => (KeyZone::Function, 0.54),
        111 => (KeyZone::Function, 0.65),
        // Numpad cluster
        65 | 67 | 69 | 71 | 75 | 78 | 81..=89 | 91 | 92 => (KeyZone::Numpad, 0.72),
        // Left-side alpha/number/punctuation
        53 | 50 | 48 => (KeyZone::Alpha, -0.72), // Esc, `, Tab
        18 | 12 | 0 | 6 => (KeyZone::Alpha, -0.62), // 1, Q, A, Z
        19 | 13 | 1 | 7 => (KeyZone::Alpha, -0.50), // 2, W, S, X
        20 | 14 | 2 | 8 => (KeyZone::Alpha, -0.38), // 3, E, D, C
        21 | 15 | 3 | 9 => (KeyZone::Alpha, -0.24), // 4, R, F, V
        23 | 17 | 5 | 11 => (KeyZone::Alpha, -0.12), // 5, T, G, B
        // Center-right
        22 | 16 | 4 | 45 => (KeyZone::Alpha, 0.04), // 6, Y, H, N
        26 | 32 | 38 | 46 => (KeyZone::Alpha, 0.16), // 7, U, J, M
        28 | 34 | 40 | 43 => (KeyZone::Alpha, 0.28), // 8, I, K, ,
        25 | 31 | 37 | 47 => (KeyZone::Alpha, 0.40), // 9, O, L, .
        29 | 35 | 41 | 44 => (KeyZone::Alpha, 0.52), // 0, P, ;, /
        27 | 24 | 33 | 30 | 42 | 39 => (KeyZone::Alpha, 0.62), // -, =, [, ], \, '
        // Arrows
        123..=126 => (KeyZone::Alpha, 0.66),
        _ => (KeyZone::Alpha, 0.0),
    }
}

/// Maps Windows Virtual-Key (`VK_*`) codes to `(KeyZone, spatial_pan)`.
pub fn map_windows_vk(vk: u32) -> (KeyZone, f32) {
    match vk {
        0x20 => (KeyZone::Space, 0.0),
        0x0D => (KeyZone::Enter, 0.52),
        0x08 | 0x2E => (KeyZone::Backspace, 0.58),
        0x10 | 0xA0 | 0x11 | 0xA2 | 0x12 | 0xA4 | 0x5B | 0x14 => (KeyZone::Modifier, -0.65),
        0xA1 | 0xA3 | 0xA5 | 0x5C => (KeyZone::Modifier, 0.52),
        0x70..=0x7B => {
            let idx = (vk - 0x70) as f32;
            (KeyZone::Function, -0.65 + (idx / 11.0) * 1.3)
        }
        0x60..=0x6F => (KeyZone::Numpad, 0.72),
        // Left keys: Q A Z 1 W S X 2 E D C 3 R F V 4 T G B 5
        0x1B | 0x09 | 0xC0 => (KeyZone::Alpha, -0.72),
        0x31 | 0x51 | 0x41 | 0x5A => (KeyZone::Alpha, -0.62),
        0x32 | 0x57 | 0x53 | 0x58 => (KeyZone::Alpha, -0.50),
        0x33 | 0x45 | 0x44 | 0x43 => (KeyZone::Alpha, -0.38),
        0x34 | 0x52 | 0x46 | 0x56 => (KeyZone::Alpha, -0.24),
        0x35 | 0x54 | 0x47 | 0x42 => (KeyZone::Alpha, -0.12),
        // Right keys: Y H N 6 U J M 7 I K , 8 O L . 9 P ; / 0
        0x36 | 0x59 | 0x48 | 0x4E => (KeyZone::Alpha, 0.04),
        0x37 | 0x55 | 0x4A | 0x4D => (KeyZone::Alpha, 0.16),
        0x38 | 0x49 | 0x4B | 0xBC => (KeyZone::Alpha, 0.28),
        0x39 | 0x4F | 0x4C | 0xBE => (KeyZone::Alpha, 0.40),
        0x30 | 0x50 | 0xBA | 0xBF => (KeyZone::Alpha, 0.52),
        0xBD | 0xBB | 0xDB | 0xDD | 0xDC | 0xDE => (KeyZone::Alpha, 0.62),
        0x25..=0x28 => (KeyZone::Alpha, 0.66),
        _ => (KeyZone::Alpha, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_filter_deduplicates_held_keydowns() {
        let filter = RepeatFilter::default();
        assert!(filter.should_emit(12, KeyEdge::Down));
        assert!(!filter.should_emit(12, KeyEdge::Down));
        assert!(!filter.should_emit(12, KeyEdge::Down));
        assert!(filter.should_emit(12, KeyEdge::Up));
        assert!(filter.should_emit(12, KeyEdge::Down));
    }

    #[test]
    fn spatial_pan_maps_left_and_right_hands_accurately() {
        let (z_q, pan_q) = map_macos_keycode(12); // Q
        let (z_p, pan_p) = map_macos_keycode(35); // P
        let (z_sp, pan_sp) = map_macos_keycode(49); // Space
        assert_eq!(z_q, KeyZone::Alpha);
        assert_eq!(z_p, KeyZone::Alpha);
        assert_eq!(z_sp, KeyZone::Space);
        assert!(pan_q < -0.5);
        assert!(pan_p > 0.4);
        assert_eq!(pan_sp, 0.0);
    }
}
