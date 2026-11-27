use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyZone {
    Alpha,
    Space,
    Enter,
    Backspace,
    Modifier,
    Function,
    Numpad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyEdge {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButtonKind {
    Left,
    Right,
    Middle,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneSoundSpec {
    pub sprite: String,
    #[serde(default = "default_one")]
    pub pitch_scale: f32,
    #[serde(default = "default_one")]
    pub gain_scale: f32,
}

fn default_one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchZoneMapping {
    pub alpha: ZoneSoundSpec,
    pub space: ZoneSoundSpec,
    pub enter: ZoneSoundSpec,
    pub backspace: ZoneSoundSpec,
    pub modifier: ZoneSoundSpec,
    pub function: ZoneSoundSpec,
    pub numpad: ZoneSoundSpec,
}

impl SwitchZoneMapping {
    pub fn for_zone(&self, zone: KeyZone) -> &ZoneSoundSpec {
        match zone {
            KeyZone::Alpha => &self.alpha,
            KeyZone::Space => &self.space,
            KeyZone::Enter => &self.enter,
            KeyZone::Backspace => &self.backspace,
            KeyZone::Modifier => &self.modifier,
            KeyZone::Function => &self.function,
            KeyZone::Numpad => &self.numpad,
        }
    }
}

fn default_category() -> String {
    "Custom".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchPack {
    pub id: String,
    pub name: String,
    pub brand: String,
    pub feel: String,
    #[serde(default = "default_category")]
    pub category: String,
    pub description: String,
    pub accent_color: String,
    pub is_custom: bool,
    pub zones: SwitchZoneMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteInfo {
    pub id: String,
    pub label: String,
    pub category: String,
    pub sample_pairs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonePadSettings {
    /// -1.0 (Warm/Thocky) .. +1.0 (Crisp/Clacky)
    pub x: f32,
    /// -1.0 (Soft/Damped) .. +1.0 (Punchy/Resonant)
    pub y: f32,
}

impl Default for TonePadSettings {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialSettings {
    pub enabled: bool,
    /// 0.0 .. 1.0 stereo spread multiplier
    pub width: f32,
    /// Enable subtle interaural time delay (ITD) for headphone realism
    pub itd_enabled: bool,
}

impl Default for SpatialSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            width: 0.85,
            itd_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseSettings {
    pub enabled: bool,
    pub volume: f32,
    pub sound_id: String,
    pub middle_sound_id: String,
    pub play_release: bool,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.75,
            sound_id: "mx-master-3".to_string(),
            middle_sound_id: "rival-3-middle".to_string(),
            play_release: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollSettings {
    pub enabled: bool,
    pub volume: f32,
    pub sound_id: String,
    /// Minimum milliseconds between scroll tick sounds (rate limiting)
    pub min_interval_ms: u64,
}

impl Default for ScrollSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.55,
            sound_id: "hi75-knob".to_string(),
            min_interval_ms: 42,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepTriggerSettings {
    pub manual_sleep: bool,
    pub on_gaming_fullscreen: bool,
    pub on_screen_sharing: bool,
    pub on_focus_dnd: bool,
    pub on_media_playing: bool,
    pub excluded_apps: Vec<String>,
}

impl Default for SleepTriggerSettings {
    fn default() -> Self {
        Self {
            manual_sleep: false,
            on_gaming_fullscreen: true,
            on_screen_sharing: true,
            on_focus_dnd: false,
            on_media_playing: false,
            excluded_apps: vec![
                "zoom.us".to_string(),
                "Discord".to_string(),
                "OBS".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SleepStatus {
    pub is_sleeping: bool,
    pub reason: Option<String>,
    pub active_app: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCategorySettings {
    pub play_modifiers: bool,
    pub play_function_keys: bool,
    pub play_numpad: bool,
    pub play_key_up: bool,
    /// Relative volume of key-up release sound (0.0 .. 1.0)
    pub key_up_volume: f32,
}

impl Default for KeyCategorySettings {
    fn default() -> Self {
        Self {
            play_modifiers: true,
            play_function_keys: true,
            play_numpad: true,
            play_key_up: true,
            key_up_volume: 0.72,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled: bool,
    pub volume: f32,
    pub active_switch_id: String,
    pub output_device: Option<String>,
    /// Pitch randomization percentage (0.0 .. 0.10, e.g. 0.035 = ±3.5%)
    pub pitch_randomization: f32,
    pub tone_pad: TonePadSettings,
    pub spatial: SpatialSettings,
    pub key_categories: KeyCategorySettings,
    pub mouse: MouseSettings,
    pub scroll: ScrollSettings,
    pub sleep_triggers: SleepTriggerSettings,
    pub toggle_shortcut: String,
    pub launch_at_login: bool,
    pub hide_tray_icon: bool,
    pub notify_on_switch_change: bool,
    pub custom_packs: Vec<SwitchPack>,
}

impl Default for AppConfig {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let default_shortcut = "Alt+Super+K".to_string();
        #[cfg(not(target_os = "macos"))]
        let default_shortcut = "Ctrl+Alt+K".to_string();

        Self {
            enabled: true,
            volume: 0.85,
            active_switch_id: "leobog-hi75".to_string(),
            output_device: None,
            pitch_randomization: 0.035,
            tone_pad: TonePadSettings::default(),
            spatial: SpatialSettings::default(),
            key_categories: KeyCategorySettings::default(),
            mouse: MouseSettings::default(),
            scroll: ScrollSettings::default(),
            sleep_triggers: SleepTriggerSettings::default(),
            toggle_shortcut: default_shortcut,
            launch_at_login: false,
            hide_tray_icon: false,
            notify_on_switch_change: true,
            custom_packs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStats {
    pub today_date: String,
    pub keystrokes_today: u64,
    pub keystrokes_all_time: u64,
    pub mouse_clicks_today: u64,
    pub mouse_clicks_all_time: u64,
    pub scroll_ticks_today: u64,
    pub scroll_ticks_all_time: u64,
    pub current_wpm: u32,
    pub peak_wpm: u32,
    pub zone_counts: HashMap<String, u64>,
    pub switch_usage: HashMap<String, u64>,
}

impl Default for TypingStats {
    fn default() -> Self {
        Self {
            today_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            keystrokes_today: 0,
            keystrokes_all_time: 0,
            mouse_clicks_today: 0,
            mouse_clicks_all_time: 0,
            scroll_ticks_today: 0,
            scroll_ticks_all_time: 0,
            current_wpm: 0,
            peak_wpm: 0,
            zone_counts: HashMap::new(),
            switch_usage: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub platform: String,
    pub accessibility_granted: bool,
    pub input_hook_active: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPayload {
    pub config: AppConfig,
    pub switch_packs: Vec<SwitchPack>,
    pub sprites: Vec<SpriteInfo>,
    pub audio_devices: Vec<AudioDeviceInfo>,
    pub sleep_status: SleepStatus,
    pub stats: TypingStats,
    pub permissions: PermissionStatus,
}
