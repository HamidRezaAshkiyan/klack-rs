export type KeyZone =
  | "alpha"
  | "space"
  | "enter"
  | "backspace"
  | "modifier"
  | "function"
  | "numpad";

export interface ZoneSoundSpec {
  sprite: string;
  pitch_scale: number;
  gain_scale: number;
}

export interface SwitchZoneMapping {
  alpha: ZoneSoundSpec;
  space: ZoneSoundSpec;
  enter: ZoneSoundSpec;
  backspace: ZoneSoundSpec;
  modifier: ZoneSoundSpec;
  function: ZoneSoundSpec;
  numpad: ZoneSoundSpec;
}

export interface SwitchPack {
  id: string;
  name: string;
  brand: string;
  feel: string;
  category: string;
  description: string;
  accent_color: string;
  is_custom: boolean;
  zones: SwitchZoneMapping;
}

export interface SpriteInfo {
  id: string;
  label: string;
  category: string;
  sample_pairs: number;
}

export interface TonePadSettings {
  x: number;
  y: number;
}

export interface SpatialSettings {
  enabled: boolean;
  width: number;
  itd_enabled: boolean;
}

export interface KeyCategorySettings {
  play_modifiers: boolean;
  play_function_keys: boolean;
  play_numpad: boolean;
  play_key_up: boolean;
  key_up_volume: number;
}

export interface MouseSettings {
  enabled: boolean;
  volume: number;
  sound_id: string;
  middle_sound_id: string;
  play_release: boolean;
}

export interface ScrollSettings {
  enabled: boolean;
  volume: number;
  sound_id: string;
  min_interval_ms: number;
}

export interface SleepTriggerSettings {
  manual_sleep: boolean;
  on_gaming_fullscreen: boolean;
  on_screen_sharing: boolean;
  on_focus_dnd: boolean;
  on_media_playing: boolean;
  excluded_apps: string[];
}

export interface SleepStatus {
  is_sleeping: boolean;
  reason: string | null;
  active_app: string | null;
}

export interface AppConfig {
  enabled: boolean;
  volume: number;
  active_switch_id: string;
  output_device: string | null;
  pitch_randomization: number;
  tone_pad: TonePadSettings;
  spatial: SpatialSettings;
  key_categories: KeyCategorySettings;
  mouse: MouseSettings;
  scroll: ScrollSettings;
  sleep_triggers: SleepTriggerSettings;
  toggle_shortcut: string;
  launch_at_login: boolean;
  hide_tray_icon: boolean;
  notify_on_switch_change: boolean;
  custom_packs: SwitchPack[];
}

export interface TypingStats {
  today_date: string;
  keystrokes_today: number;
  keystrokes_all_time: number;
  mouse_clicks_today: number;
  mouse_clicks_all_time: number;
  scroll_ticks_today: number;
  scroll_ticks_all_time: number;
  current_wpm: number;
  peak_wpm: number;
  zone_counts: Record<string, number>;
  switch_usage: Record<string, number>;
}

export interface AudioDeviceInfo {
  name: string;
  is_default: boolean;
}

export interface PermissionStatus {
  platform: string;
  accessibility_granted: boolean;
  input_hook_active: boolean;
  note: string;
}

export interface BootstrapPayload {
  config: AppConfig;
  switch_packs: SwitchPack[];
  sprites: SpriteInfo[];
  audio_devices: AudioDeviceInfo[];
  sleep_status: SleepStatus;
  stats: TypingStats;
  permissions: PermissionStatus;
}

export type SettingsTabId =
  | "switches"
  | "audio"
  | "mouse"
  | "sleep"
  | "stats"
  | "general";
